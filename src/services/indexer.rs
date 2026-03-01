//! Workspace indexing service.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result, anyhow, bail};
use log::{trace, warn};
use notify::{Event, RecursiveMode, Watcher};
use regex::RegexBuilder;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::AllQuery;
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term, doc};

use crate::configuration::default_workspace_index_root;
use crate::errors::AlfredError;
use crate::path_encoding::{
	render_component, render_relative_path, resolve_workspace_relative_path,
};
use crate::redaction::Redactor;
use crate::services::workspace_files::{
	IndexIgnoreMatcher, read_indexable_utf8_text, read_text_file_range,
	visit_utf8_lines_best_effort,
};
use crate::workspace_boundary::SYMLINK_JUNCTION_ESCAPE_MESSAGE;

const DOC_KIND_FILE: &str = "file";
const DOC_KIND_DIRECTORY: &str = "directory";
const INDEX_SUBDIRECTORY: &str = "tantivy";

/// Maximum on-disk file size (bytes) we will fully load into memory for indexing.
///
/// Larger files are still tracked (path + mtime) but their content is not cached in the
/// in-memory snapshot. Query operations may fall back to streaming reads from disk.
const MAX_INDEXED_TEXT_BYTES: u64 = 4 * 1024 * 1024;

fn default_user_ignore_path(workspace_root: &Path) -> PathBuf {
	if let Some(config_dir) = dirs::config_dir() {
		return config_dir.join("alfred").join(".alfredignore");
	}

	workspace_root.join(".alfred").join(".alfredignore.user")
}

fn is_alfredignore_path(relative_path: &str) -> bool {
	relative_path == ".alfredignore" || relative_path.ends_with("/.alfredignore")
}

/// A single text match found by index-backed query operations.
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SearchMatch {
	pub path: String,
	pub line: usize,
	pub text: String,
}

/// Summary of a workspace index build.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexBuildStats {
	pub indexed_files: usize,
	pub indexed_directories: usize,
	pub indexed_text_files: usize,
}

#[derive(Debug, Clone, Default)]
struct WorkspaceSnapshot {
	files: Vec<IndexedFile>,
	directories: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct IndexedFile {
	path: String,
	text: Option<String>,
	modified_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy)]
struct TantivyFields {
	kind: Field,
	path: Field,
	text: Field,
}

struct TantivyEngine {
	fields: TantivyFields,
	reader: IndexReader,
	writer: Mutex<IndexWriter>,
}

impl fmt::Debug for TantivyEngine {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter
			.debug_struct("TantivyEngine")
			.field("fields", &self.fields)
			.finish_non_exhaustive()
	}
}

/// Maintains workspace index lifecycle.
#[derive(Debug, Clone)]
pub struct WorkspaceIndexer {
	workspace_root: PathBuf,
	index_storage_root: PathBuf,
	ignore: Arc<IndexIgnoreMatcher>,
	redactor: Arc<Redactor>,
	snapshot: Arc<RwLock<WorkspaceSnapshot>>,
	engine: Arc<TantivyEngine>,
	index_ready: Arc<AtomicBool>,
	watch_running: Arc<AtomicBool>,
	watch_stop: Arc<AtomicBool>,
	persist_dirty: Arc<AtomicBool>,
	persist_interval: Duration,
	watch_debounce: Duration,
}

impl WorkspaceIndexer {
	/// Creates a new indexer for the given workspace root.
	pub fn new(workspace_root: PathBuf) -> Result<Self> {
		let index_storage_root = default_workspace_index_root(&workspace_root);
		let user_ignore_path = default_user_ignore_path(&workspace_root);
		let redactor = Arc::new(Redactor::try_default()?);
		Self::new_with_options(
			workspace_root,
			index_storage_root,
			Duration::from_secs(5),
			Duration::from_millis(250),
			user_ignore_path,
			None,
			redactor,
		)
	}

	/// Creates a new indexer using explicit persistence and watch settings.
	pub fn new_with_options(
		workspace_root: PathBuf,
		index_storage_root: PathBuf,
		persist_interval: Duration,
		watch_debounce: Duration,
		user_ignore_path: PathBuf,
		workspace_ignore_path: Option<PathBuf>,
		redactor: Arc<Redactor>,
	) -> Result<Self> {
		let ignore = IndexIgnoreMatcher::new(
			workspace_root.clone(),
			user_ignore_path,
			workspace_ignore_path,
		)
		.context("failed to build workspace index ignore rules")?;

		let tantivy_path = index_storage_root.join(INDEX_SUBDIRECTORY);
		fs::create_dir_all(&tantivy_path).with_context(|| {
			format!(
				"failed to create index persistence directory at {}",
				tantivy_path.display()
			)
		})?;

		let schema = build_schema();
		let fields = resolve_schema_fields(&schema)?;
		let directory = MmapDirectory::open(&tantivy_path).with_context(|| {
			format!(
				"failed to open tantivy directory at {}",
				tantivy_path.display()
			)
		})?;
		let index = Index::open_or_create(directory, schema)
			.context("failed to open or create tantivy index")?;
		let reader = index
			.reader_builder()
			.reload_policy(ReloadPolicy::Manual)
			.try_into()
			.context("failed to initialize tantivy index reader")?;
		let writer = index
			.writer(50_000_000)
			.context("failed to initialize tantivy index writer")?;

		Ok(Self {
			workspace_root,
			index_storage_root,
			ignore: Arc::new(ignore),
			redactor,
			snapshot: Arc::new(RwLock::new(WorkspaceSnapshot::default())),
			engine: Arc::new(TantivyEngine {
				fields,
				reader,
				writer: Mutex::new(writer),
			}),
			index_ready: Arc::new(AtomicBool::new(false)),
			watch_running: Arc::new(AtomicBool::new(false)),
			watch_stop: Arc::new(AtomicBool::new(false)),
			persist_dirty: Arc::new(AtomicBool::new(false)),
			persist_interval,
			watch_debounce,
		})
	}

	/// Initializes the in-memory snapshot from persisted index when available.
	pub fn initialize(&self) -> Result<IndexBuildStats> {
		let persisted = self.load_snapshot_from_index()?;
		if persisted.indexed_files > 0 || persisted.indexed_directories > 0 {
			trace!(
				"workspace index initialization mode=persisted indexed_files={} indexed_directories={} indexed_text_files={}",
				persisted.indexed_files,
				persisted.indexed_directories,
				persisted.indexed_text_files,
			);
			return Ok(persisted);
		}

		let rebuilt = self.rebuild()?;
		trace!(
			"workspace index initialization mode=rebuild indexed_files={} indexed_directories={} indexed_text_files={}",
			rebuilt.indexed_files, rebuilt.indexed_directories, rebuilt.indexed_text_files,
		);
		Ok(rebuilt)
	}

	/// Returns the workspace root used by the indexer.
	pub fn workspace_root(&self) -> &Path {
		&self.workspace_root
	}

	/// Returns whether the index has been initialized and is ready for query tools.
	pub fn is_ready(&self) -> bool {
		self.index_ready.load(AtomicOrdering::Relaxed)
	}

	/// Rebuilds the in-memory workspace index from disk.
	pub fn rebuild(&self) -> Result<IndexBuildStats> {
		let snapshot = build_snapshot(
			&self.workspace_root,
			self.ignore.as_ref(),
			self.redactor.as_ref(),
		)?;
		let stats = stats_from_snapshot(&snapshot);
		self.write_full_snapshot(snapshot.clone())?;

		let mut state = self
			.snapshot
			.write()
			.map_err(|_| anyhow!("workspace index lock poisoned"))?;
		*state = snapshot;
		self.index_ready.store(true, AtomicOrdering::Relaxed);
		Ok(stats)
	}

	/// Returns deterministically ordered indexed file paths.
	pub fn list_files(&self) -> Vec<String> {
		self.snapshot
			.read()
			.map(|snapshot| {
				snapshot
					.files
					.iter()
					.map(|file| file.path.clone())
					.collect::<Vec<_>>()
			})
			.unwrap_or_default()
	}

	/// Returns deterministically ordered indexed directory paths.
	pub fn list_directories(&self) -> Vec<String> {
		self.snapshot
			.read()
			.map(|snapshot| snapshot.directories.clone())
			.unwrap_or_default()
	}

	/// Returns lightweight snapshot statistics without cloning file/directory lists.
	pub fn snapshot_stats(&self) -> IndexBuildStats {
		let Ok(snapshot) = self.snapshot.read() else {
			return IndexBuildStats {
				indexed_files: 0,
				indexed_directories: 0,
				indexed_text_files: 0,
			};
		};

		IndexBuildStats {
			indexed_files: snapshot.files.len(),
			indexed_directories: snapshot.directories.len(),
			indexed_text_files: snapshot
				.files
				.iter()
				.filter(|file| file.text.is_some())
				.count(),
		}
	}

	/// Performs literal search over indexed text files.
	pub fn grep_literal(&self, query: &str, case_sensitive: bool) -> Vec<SearchMatch> {
		if query.is_empty() {
			return Vec::new();
		}

		let Ok(snapshot) = self.snapshot.read() else {
			return Vec::new();
		};

		let needle = if case_sensitive {
			None
		} else {
			Some(query.to_lowercase())
		};

		let mut matches = Vec::new();
		for file in &snapshot.files {
			if let Some(text) = &file.text {
				collect_literal_matches(
					file.path.as_str(),
					text,
					query,
					needle.as_deref(),
					&mut matches,
				);
				continue;
			}

			let absolute_path =
				resolve_workspace_relative_path(self.workspace_root.as_path(), file.path.as_str());
			let absolute_path =
				match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
					self.workspace_root.as_path(),
					absolute_path.as_path(),
				) {
					Ok(Some(resolved)) => resolved,
					Ok(None) => continue,
					Err(_) => continue,
				};
			let _ = collect_literal_matches_from_disk(
				file.path.as_str(),
				absolute_path.as_path(),
				query,
				needle.as_deref(),
				&mut matches,
			);
		}

		sort_matches(&mut matches);
		matches
	}

	/// Performs regular-expression search over indexed text files.
	pub fn search_regex(&self, pattern: &str, case_sensitive: bool) -> Result<Vec<SearchMatch>> {
		if pattern.is_empty() {
			return Ok(Vec::new());
		}

		let regex = RegexBuilder::new(pattern)
			.case_insensitive(!case_sensitive)
			.build()
			.with_context(|| format!("invalid regex pattern: {pattern}"))?;

		let snapshot = self
			.snapshot
			.read()
			.map_err(|_| anyhow!("workspace index lock poisoned"))?;

		let mut matches = Vec::new();
		for file in &snapshot.files {
			if let Some(text) = &file.text {
				collect_regex_matches(file.path.as_str(), text, &regex, &mut matches);
				continue;
			}

			let absolute_path =
				resolve_workspace_relative_path(self.workspace_root.as_path(), file.path.as_str());
			let absolute_path =
				match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
					self.workspace_root.as_path(),
					absolute_path.as_path(),
				) {
					Ok(Some(resolved)) => resolved,
					Ok(None) => continue,
					Err(_) => continue,
				};
			let _ = collect_regex_matches_from_disk(
				file.path.as_str(),
				absolute_path.as_path(),
				&regex,
				&mut matches,
			);
		}

		sort_matches(&mut matches);
		Ok(matches)
	}

	/// Reads an inclusive line range from an indexed text file.
	pub fn read_range(&self, path: &str, start_line: usize, end_line: usize) -> Result<String> {
		if start_line == 0 || end_line == 0 {
			bail!("line numbers must be 1-indexed");
		}
		if end_line < start_line {
			bail!("end_line must be greater than or equal to start_line");
		}

		let normalized_path = normalize_relative(path);
		let absolute_path = resolve_workspace_relative_path(
			self.workspace_root.as_path(),
			normalized_path.as_str(),
		);
		let absolute_path =
			match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
				self.workspace_root.as_path(),
				absolute_path.as_path(),
			) {
				Ok(Some(resolved)) => resolved,
				Ok(None) => absolute_path,
				Err(AlfredError::PermissionDenied(_)) => bail!(SYMLINK_JUNCTION_ESCAPE_MESSAGE),
				Err(error) => bail!(error.to_string()),
			};
		{
			let snapshot = self
				.snapshot
				.read()
				.map_err(|_| anyhow!("workspace index lock poisoned"))?;
			if let Some(file) = snapshot
				.files
				.iter()
				.find(|file| file.path == normalized_path)
				&& let (Some(text), Some(index_modified_at)) = (&file.text, file.modified_at)
			{
				let disk_modified_at = fs::metadata(&absolute_path)
					.ok()
					.and_then(|metadata| metadata.modified().ok());
				if disk_modified_at.is_some_and(|disk| disk <= index_modified_at) {
					return slice_text_lines(
						text.as_str(),
						normalized_path.as_str(),
						start_line,
						end_line,
					);
				}
			}
		}

		read_text_file_range(
			absolute_path.as_path(),
			normalized_path.as_str(),
			start_line,
			end_line,
		)
	}

	/// Starts the background watch thread that reindexes on file saves.
	pub fn start_watch_thread(&self) -> Result<bool> {
		if self.watch_running.swap(true, AtomicOrdering::SeqCst) {
			return Ok(false);
		}

		self.watch_stop.store(false, AtomicOrdering::SeqCst);
		let watcher = self.clone();
		thread::Builder::new()
			.name("alfred-indexer-watch".to_string())
			.spawn(move || watcher.watch_loop())
			.context("failed to spawn workspace index watch thread")?;

		Ok(true)
	}

	/// Stops the background watch thread.
	pub fn stop_watch_thread(&self) {
		self.watch_stop.store(true, AtomicOrdering::SeqCst);
	}

	fn watch_loop(self) {
		let (event_tx, event_rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
		let mut watcher = match notify::recommended_watcher(move |event| {
			let _ = event_tx.send(event);
		}) {
			Ok(watcher) => watcher,
			Err(error) => {
				warn!("workspace watcher initialization failed: {error:#}");
				self.watch_running.store(false, AtomicOrdering::SeqCst);
				return;
			}
		};

		if let Err(error) = watcher.watch(self.workspace_root.as_path(), RecursiveMode::Recursive) {
			warn!("workspace watcher start failed: {error:#}");
			self.watch_running.store(false, AtomicOrdering::SeqCst);
			return;
		}

		let mut pending_paths: HashMap<String, Instant> = HashMap::new();
		let mut last_persist = Instant::now();

		while !self.watch_stop.load(AtomicOrdering::Relaxed) {
			match event_rx.recv_timeout(Duration::from_millis(100)) {
				Ok(Ok(event)) => self.enqueue_watch_event(&mut pending_paths, &event),
				Ok(Err(error)) => warn!("workspace watch error: {error:#}"),
				Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
				Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
					warn!("workspace watch channel disconnected");
					break;
				}
			}

			if let Err(error) = self.flush_pending_paths(&mut pending_paths, false) {
				warn!("workspace watch incremental update failed: {error:#}");
			}

			if last_persist.elapsed() >= self.persist_interval {
				if let Err(error) = self.persist_if_dirty() {
					warn!("workspace index periodic persist failed: {error:#}");
				}
				last_persist = Instant::now();
			}
		}

		if let Err(error) = self.flush_pending_paths(&mut pending_paths, true) {
			warn!("workspace watch final incremental update failed: {error:#}");
		}
		if let Err(error) = self.persist_if_dirty() {
			warn!("workspace index final persist failed: {error:#}");
		}

		self.watch_running.store(false, AtomicOrdering::SeqCst);
	}

	fn enqueue_watch_event(&self, pending_paths: &mut HashMap<String, Instant>, event: &Event) {
		for path in &event.paths {
			if let Some(relative) = self.to_relative_watch_path(path.as_path()) {
				pending_paths.insert(relative, Instant::now());
			}
		}
	}

	fn to_relative_watch_path(&self, path: &Path) -> Option<String> {
		let relative = path.strip_prefix(&self.workspace_root).ok()?;
		let normalized = normalize_relative(relative);
		if normalized.is_empty() {
			return None;
		}
		if is_alfredignore_path(normalized.as_str()) {
			return Some(normalized);
		}
		if self
			.ignore
			.should_ignore_relative(normalized.as_str(), false)
		{
			return None;
		}

		Some(normalized)
	}

	fn flush_pending_paths(
		&self,
		pending_paths: &mut HashMap<String, Instant>,
		force: bool,
	) -> Result<()> {
		let due_paths = pending_paths
			.iter()
			.filter_map(|(path, seen_at)| {
				if force || seen_at.elapsed() >= self.watch_debounce {
					Some(path.clone())
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		if due_paths.is_empty() {
			return Ok(());
		}

		for path in due_paths {
			pending_paths.remove(path.as_str());
			if self.refresh_relative_path(path.as_str())? {
				trace!("workspace index incrementally updated path={path}");
			}
		}

		Ok(())
	}

	fn refresh_relative_path(&self, relative_path: &str) -> Result<bool> {
		if relative_path.is_empty() {
			return Ok(false);
		}
		if is_alfredignore_path(relative_path) {
			self.rebuild()?;
			return Ok(true);
		}

		let absolute_path =
			resolve_workspace_relative_path(self.workspace_root.as_path(), relative_path);
		let resolved_path =
			match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
				self.workspace_root.as_path(),
				absolute_path.as_path(),
			) {
				Ok(Some(resolved)) => resolved,
				Ok(None) => return self.remove_path_entries(relative_path),
				Err(AlfredError::PermissionDenied(_)) => {
					let _ = self.remove_path_entries(relative_path);
					return Ok(true);
				}
				Err(error) => bail!(error.to_string()),
			};

		let metadata = fs::metadata(resolved_path.as_path())
			.with_context(|| format!("failed to stat {}", resolved_path.display()))?;
		if self
			.ignore
			.should_ignore_relative(relative_path, metadata.is_dir())
		{
			return Ok(false);
		}
		if metadata.is_file() {
			return self.upsert_file(relative_path, resolved_path.as_path());
		}
		if metadata.is_dir() {
			return self.refresh_directory_subtree(relative_path, resolved_path.as_path());
		}

		self.remove_path_entries(relative_path)
	}

	fn upsert_file(&self, relative_path: &str, absolute_path: &Path) -> Result<bool> {
		let text = read_indexable_utf8_text(absolute_path, MAX_INDEXED_TEXT_BYTES)
			.map(|text| redact_string_if_needed(self.redactor.as_ref(), text));
		let modified_at = fs::metadata(absolute_path)
			.ok()
			.and_then(|metadata| metadata.modified().ok());
		let mut changed = false;
		let added_directories = {
			let mut snapshot = self
				.snapshot
				.write()
				.map_err(|_| anyhow!("workspace index lock poisoned"))?;

			let mut updated = false;
			for file in &mut snapshot.files {
				if file.path == relative_path {
					updated = true;
					if file.text != text || file.modified_at != modified_at {
						file.text = text.clone();
						file.modified_at = modified_at;
						changed = true;
					}
					break;
				}
			}

			if !updated {
				snapshot.files.push(IndexedFile {
					path: relative_path.to_string(),
					text: text.clone(),
					modified_at,
				});
				snapshot
					.files
					.sort_by(|left, right| compare_paths(left.path.as_str(), right.path.as_str()));
				changed = true;
			}

			let added_directories = ensure_parent_directories(
				relative_path,
				self.ignore.as_ref(),
				snapshot.directories.as_mut_slice(),
			);
			if !added_directories.is_empty() {
				snapshot.directories.extend(added_directories.clone());
				snapshot
					.directories
					.sort_by(|left, right| compare_paths(left, right));
				changed = true;
			}

			added_directories
		};

		if !changed {
			return Ok(false);
		}

		let mut writer = self
			.engine
			.writer
			.lock()
			.map_err(|_| anyhow!("workspace index writer lock poisoned"))?;
		let fields = self.engine.fields;
		writer.delete_term(Term::from_field_text(fields.path, relative_path));
		for directory in &added_directories {
			writer.delete_term(Term::from_field_text(fields.path, directory.as_str()));
			add_directory_document(&mut writer, fields, directory)?;
		}
		add_file_document(&mut writer, fields, relative_path, text.as_deref())?;

		self.persist_dirty.store(true, AtomicOrdering::Relaxed);
		Ok(true)
	}

	fn refresh_directory_subtree(&self, relative_path: &str, absolute_path: &Path) -> Result<bool> {
		let mut scanned_files = Vec::new();
		let mut scanned_directories = vec![relative_path.to_string()];
		scan_directory(
			&self.workspace_root,
			absolute_path,
			self.ignore.as_ref(),
			self.redactor.as_ref(),
			&mut scanned_files,
			&mut scanned_directories,
		)?;

		scanned_files.sort_by(|left, right| compare_paths(left.path.as_str(), right.path.as_str()));
		scanned_directories.sort_by(|left, right| compare_paths(left, right));

		let (existing_files, existing_directories) = {
			let snapshot = self
				.snapshot
				.read()
				.map_err(|_| anyhow!("workspace index lock poisoned"))?;

			let files = snapshot
				.files
				.iter()
				.filter(|file| is_path_or_child(relative_path, file.path.as_str()))
				.cloned()
				.collect::<Vec<_>>();
			let directories = snapshot
				.directories
				.iter()
				.filter(|directory| is_path_or_child(relative_path, directory.as_ref()))
				.cloned()
				.collect::<Vec<_>>();
			(files, directories)
		};

		if existing_files == scanned_files && existing_directories == scanned_directories {
			return Ok(false);
		}

		{
			let mut snapshot = self
				.snapshot
				.write()
				.map_err(|_| anyhow!("workspace index lock poisoned"))?;
			snapshot
				.files
				.retain(|file| !is_path_or_child(relative_path, file.path.as_str()));
			snapshot
				.directories
				.retain(|directory| !is_path_or_child(relative_path, directory.as_ref()));
			snapshot.files.extend(scanned_files.clone());
			snapshot.directories.extend(scanned_directories.clone());
			snapshot
				.files
				.sort_by(|left, right| compare_paths(left.path.as_str(), right.path.as_str()));
			snapshot
				.directories
				.sort_by(|left, right| compare_paths(left, right));
		}

		let mut writer = self
			.engine
			.writer
			.lock()
			.map_err(|_| anyhow!("workspace index writer lock poisoned"))?;
		let fields = self.engine.fields;
		for file in &existing_files {
			writer.delete_term(Term::from_field_text(fields.path, file.path.as_str()));
		}
		for directory in &existing_directories {
			writer.delete_term(Term::from_field_text(fields.path, directory.as_str()));
		}
		for directory in &scanned_directories {
			add_directory_document(&mut writer, fields, directory)?;
		}
		for file in &scanned_files {
			add_file_document(
				&mut writer,
				fields,
				file.path.as_str(),
				file.text.as_deref(),
			)?;
		}

		self.persist_dirty.store(true, AtomicOrdering::Relaxed);
		Ok(true)
	}

	fn remove_path_entries(&self, relative_path: &str) -> Result<bool> {
		let (existing_files, existing_directories) = {
			let snapshot = self
				.snapshot
				.read()
				.map_err(|_| anyhow!("workspace index lock poisoned"))?;

			let files = snapshot
				.files
				.iter()
				.filter(|file| is_path_or_child(relative_path, file.path.as_str()))
				.map(|file| file.path.clone())
				.collect::<Vec<_>>();
			let directories = snapshot
				.directories
				.iter()
				.filter(|directory| is_path_or_child(relative_path, directory.as_ref()))
				.cloned()
				.collect::<Vec<_>>();
			(files, directories)
		};

		if existing_files.is_empty() && existing_directories.is_empty() {
			return Ok(false);
		}

		{
			let mut snapshot = self
				.snapshot
				.write()
				.map_err(|_| anyhow!("workspace index lock poisoned"))?;
			snapshot
				.files
				.retain(|file| !is_path_or_child(relative_path, file.path.as_str()));
			snapshot
				.directories
				.retain(|directory| !is_path_or_child(relative_path, directory.as_ref()));
		}

		let writer = self
			.engine
			.writer
			.lock()
			.map_err(|_| anyhow!("workspace index writer lock poisoned"))?;
		let fields = self.engine.fields;
		for file in &existing_files {
			writer.delete_term(Term::from_field_text(fields.path, file.as_str()));
		}
		for directory in &existing_directories {
			writer.delete_term(Term::from_field_text(fields.path, directory.as_str()));
		}

		self.persist_dirty.store(true, AtomicOrdering::Relaxed);
		Ok(true)
	}

	fn persist_if_dirty(&self) -> Result<()> {
		if !self.persist_dirty.load(AtomicOrdering::Relaxed) {
			return Ok(());
		}

		let mut writer = self
			.engine
			.writer
			.lock()
			.map_err(|_| anyhow!("workspace index writer lock poisoned"))?;
		writer
			.commit()
			.context("failed to commit workspace index")?;
		drop(writer);

		self.engine
			.reader
			.reload()
			.context("failed to reload workspace index reader")?;
		self.persist_dirty.store(false, AtomicOrdering::Relaxed);
		Ok(())
	}

	fn write_full_snapshot(&self, snapshot: WorkspaceSnapshot) -> Result<()> {
		let mut writer = self
			.engine
			.writer
			.lock()
			.map_err(|_| anyhow!("workspace index writer lock poisoned"))?;
		writer
			.delete_all_documents()
			.context("failed to reset workspace index documents")?;

		let fields = self.engine.fields;
		for directory in &snapshot.directories {
			add_directory_document(&mut writer, fields, directory)?;
		}
		for file in &snapshot.files {
			add_file_document(
				&mut writer,
				fields,
				file.path.as_str(),
				file.text.as_deref(),
			)?;
		}

		writer
			.commit()
			.context("failed to commit full workspace index")?;
		drop(writer);

		self.engine
			.reader
			.reload()
			.context("failed to reload workspace index reader")?;
		self.persist_dirty.store(false, AtomicOrdering::Relaxed);
		Ok(())
	}

	fn load_snapshot_from_index(&self) -> Result<IndexBuildStats> {
		self.engine
			.reader
			.reload()
			.context("failed to reload workspace index reader")?;

		let searcher = self.engine.reader.searcher();
		let doc_count = searcher
			.search(&AllQuery, &Count)
			.context("failed to count persisted index documents")?;

		if doc_count == 0 {
			let mut state = self
				.snapshot
				.write()
				.map_err(|_| anyhow!("workspace index lock poisoned"))?;
			*state = WorkspaceSnapshot::default();
			return Ok(IndexBuildStats {
				indexed_files: 0,
				indexed_directories: 0,
				indexed_text_files: 0,
			});
		}

		let mut files = Vec::new();
		let mut directories = Vec::new();
		let top_docs = searcher
			.search(&AllQuery, &TopDocs::with_limit(doc_count))
			.context("failed to read persisted index documents")?;

		for (_score, doc_address) in top_docs {
			let document = searcher
				.doc::<TantivyDocument>(doc_address)
				.context("failed to deserialize persisted index document")?;

			let Some(kind) = document
				.get_first(self.engine.fields.kind)
				.and_then(|value| value.as_str())
			else {
				continue;
			};
			let Some(path) = document
				.get_first(self.engine.fields.path)
				.and_then(|value| value.as_str())
			else {
				continue;
			};

			if kind == DOC_KIND_FILE {
				let text = document
					.get_first(self.engine.fields.text)
					.and_then(|value| value.as_str())
					.map(|value| value.to_string());
				files.push(IndexedFile {
					path: path.to_string(),
					text,
					modified_at: None,
				});
			} else if kind == DOC_KIND_DIRECTORY {
				directories.push(path.to_string());
			}
		}

		files.sort_by(|left, right| compare_paths(left.path.as_str(), right.path.as_str()));
		directories.sort_by(|left, right| compare_paths(left, right));

		let snapshot = WorkspaceSnapshot { files, directories };
		let stats = stats_from_snapshot(&snapshot);
		let mut state = self
			.snapshot
			.write()
			.map_err(|_| anyhow!("workspace index lock poisoned"))?;
		*state = snapshot;
		self.index_ready.store(true, AtomicOrdering::Relaxed);
		trace!(
			"loaded persisted workspace index root={} storage={}",
			self.workspace_root.display(),
			self.index_storage_root.display()
		);
		Ok(stats)
	}
}

fn build_snapshot(
	workspace_root: &Path,
	ignore: &IndexIgnoreMatcher,
	redactor: &Redactor,
) -> Result<WorkspaceSnapshot> {
	let mut files = Vec::new();
	let mut directories = Vec::new();
	scan_directory(
		workspace_root,
		workspace_root,
		ignore,
		redactor,
		&mut files,
		&mut directories,
	)?;

	files.sort_by(|left, right| compare_paths(left.path.as_str(), right.path.as_str()));
	directories.sort_by(|left, right| compare_paths(left, right));

	Ok(WorkspaceSnapshot { files, directories })
}

fn stats_from_snapshot(snapshot: &WorkspaceSnapshot) -> IndexBuildStats {
	IndexBuildStats {
		indexed_files: snapshot.files.len(),
		indexed_directories: snapshot.directories.len(),
		indexed_text_files: snapshot
			.files
			.iter()
			.filter(|file| file.text.is_some())
			.count(),
	}
}

fn scan_directory(
	workspace_root: &Path,
	current_dir: &Path,
	ignore: &IndexIgnoreMatcher,
	redactor: &Redactor,
	files: &mut Vec<IndexedFile>,
	directories: &mut Vec<String>,
) -> Result<()> {
	for entry in sorted_entries(current_dir)? {
		let path = entry.path();
		let relative =
			normalize_relative(path.strip_prefix(workspace_root).with_context(|| {
				format!(
					"failed to normalize path relative to workspace: {}",
					path.display()
				)
			})?);

		if relative.is_empty() {
			continue;
		}

		let file_type = entry
			.file_type()
			.with_context(|| format!("failed to inspect file type for {}", path.display()))?;

		if file_type.is_dir() {
			if ignore.should_ignore_relative(relative.as_str(), true) {
				continue;
			}

			directories.push(relative);
			scan_directory(workspace_root, &path, ignore, redactor, files, directories)?;
			continue;
		}

		if file_type.is_file() {
			if ignore.should_ignore_relative(relative.as_str(), false) {
				continue;
			}

			let modified_at = entry
				.metadata()
				.ok()
				.and_then(|metadata| metadata.modified().ok());
			let text = read_indexable_utf8_text(path.as_path(), MAX_INDEXED_TEXT_BYTES)
				.map(|text| redact_string_if_needed(redactor, text));
			files.push(IndexedFile {
				path: relative,
				text,
				modified_at,
			});
		}
	}

	Ok(())
}

fn redact_string_if_needed(redactor: &Redactor, input: String) -> String {
	let result = redactor.redact_text_with_stats(input.as_str());
	if result.stats.redacted_spans == 0 {
		return input;
	}
	result.output
}

fn sorted_entries(path: &Path) -> Result<Vec<fs::DirEntry>> {
	let mut entries = fs::read_dir(path)
		.with_context(|| format!("failed to read directory {}", path.display()))?
		.collect::<std::result::Result<Vec<_>, _>>()
		.with_context(|| format!("failed to enumerate directory {}", path.display()))?;

	entries.sort_by(|left, right| {
		let left_name = render_component(left.file_name().as_os_str());
		let right_name = render_component(right.file_name().as_os_str());
		compare_paths(left_name.text.as_str(), right_name.text.as_str())
	});

	Ok(entries)
}

fn ensure_parent_directories(
	relative_file: &str,
	ignore: &IndexIgnoreMatcher,
	directories: &mut [String],
) -> Vec<String> {
	let mut existing = directories
		.iter()
		.map(|value| value.as_str())
		.collect::<std::collections::HashSet<_>>();
	let mut added = Vec::new();

	let mut cursor = relative_file;
	while let Some((parent, _name)) = cursor.rsplit_once('/') {
		if parent.is_empty() || ignore.should_ignore_relative(parent, true) {
			break;
		}
		if !existing.contains(parent) {
			added.push(parent.to_string());
			existing.insert(parent);
		}
		cursor = parent;
	}

	added
}

fn is_path_or_child(root: &str, candidate: &str) -> bool {
	candidate == root
		|| candidate
			.strip_prefix(root)
			.is_some_and(|suffix| suffix.starts_with('/'))
}

fn collect_literal_matches(
	path: &str,
	text: &str,
	query: &str,
	query_lower: Option<&str>,
	results: &mut Vec<SearchMatch>,
) {
	for (index, line) in text.lines().enumerate() {
		let matched = if let Some(needle) = query_lower {
			line.to_lowercase().contains(needle)
		} else {
			line.contains(query)
		};

		if matched {
			results.push(SearchMatch {
				path: path.to_string(),
				line: index + 1,
				text: line.to_string(),
			});
		}
	}
}

fn collect_regex_matches(
	path: &str,
	text: &str,
	regex: &regex::Regex,
	results: &mut Vec<SearchMatch>,
) {
	for (index, line) in text.lines().enumerate() {
		if regex.is_match(line) {
			results.push(SearchMatch {
				path: path.to_string(),
				line: index + 1,
				text: line.to_string(),
			});
		}
	}
}

fn sort_matches(matches: &mut [SearchMatch]) {
	matches.sort_by(|left, right| {
		compare_paths(left.path.as_str(), right.path.as_str())
			.then(left.line.cmp(&right.line))
			.then(left.text.cmp(&right.text))
	});
}

fn compare_paths(left: &str, right: &str) -> Ordering {
	left.to_lowercase()
		.cmp(&right.to_lowercase())
		.then(left.cmp(right))
}

fn build_schema() -> Schema {
	let mut builder = Schema::builder();
	builder.add_text_field("kind", STRING | STORED);
	builder.add_text_field("path", STRING | STORED);
	builder.add_text_field("text", TEXT | STORED);
	builder.build()
}

fn resolve_schema_fields(schema: &Schema) -> Result<TantivyFields> {
	let kind = schema
		.get_field("kind")
		.context("tantivy schema missing kind field")?;
	let path = schema
		.get_field("path")
		.context("tantivy schema missing path field")?;
	let text = schema
		.get_field("text")
		.context("tantivy schema missing text field")?;

	Ok(TantivyFields { kind, path, text })
}

fn add_directory_document(
	writer: &mut IndexWriter,
	fields: TantivyFields,
	path: &str,
) -> Result<()> {
	writer.delete_term(Term::from_field_text(fields.path, path));
	writer
		.add_document(doc!(
			fields.kind => DOC_KIND_DIRECTORY,
			fields.path => path,
		))
		.context("failed to add directory document to tantivy index")?;
	Ok(())
}

fn add_file_document(
	writer: &mut IndexWriter,
	fields: TantivyFields,
	path: &str,
	text: Option<&str>,
) -> Result<()> {
	writer.delete_term(Term::from_field_text(fields.path, path));
	match text {
		Some(content) => {
			writer
				.add_document(doc!(
					fields.kind => DOC_KIND_FILE,
					fields.path => path,
					fields.text => content,
				))
				.context("failed to add file document to tantivy index")?;
		}
		None => {
			writer
				.add_document(doc!(
					fields.kind => DOC_KIND_FILE,
					fields.path => path,
				))
				.context("failed to add file document to tantivy index")?;
		}
	}

	Ok(())
}

fn normalize_relative(path: impl AsRef<Path>) -> String {
	render_relative_path(path.as_ref()).text
}

fn slice_text_lines(text: &str, path: &str, start_line: usize, end_line: usize) -> Result<String> {
	let mut output = String::new();
	let mut seen_lines = 0usize;
	for (index, line) in text.lines().enumerate() {
		let line_number = index + 1;
		seen_lines = line_number;
		if line_number < start_line {
			continue;
		}
		if line_number > end_line {
			break;
		}
		if !output.is_empty() {
			output.push('\n');
		}
		output.push_str(line);
		if line_number == end_line {
			break;
		}
	}

	if end_line > seen_lines {
		bail!("requested line range is out of bounds for file: {path}");
	}

	Ok(output)
}

fn collect_literal_matches_from_disk(
	path: &str,
	absolute_path: &Path,
	query: &str,
	query_lower: Option<&str>,
	results: &mut Vec<SearchMatch>,
) -> Result<()> {
	visit_utf8_lines_best_effort(absolute_path, |line_number, line| {
		let matched = if let Some(needle) = query_lower {
			line.to_lowercase().contains(needle)
		} else {
			line.contains(query)
		};
		if matched {
			results.push(SearchMatch {
				path: path.to_string(),
				line: line_number,
				text: line.to_string(),
			});
		}
	})
}

fn collect_regex_matches_from_disk(
	path: &str,
	absolute_path: &Path,
	regex: &regex::Regex,
	results: &mut Vec<SearchMatch>,
) -> Result<()> {
	visit_utf8_lines_best_effort(absolute_path, |line_number, line| {
		if regex.is_match(line) {
			results.push(SearchMatch {
				path: path.to_string(),
				line: line_number,
				text: line.to_string(),
			});
		}
	})
}
