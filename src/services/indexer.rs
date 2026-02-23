//! Workspace indexing service.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result, anyhow, bail};
use regex::RegexBuilder;

/// A single text match found by index-backed query operations.
#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone)]
struct IndexedFile {
	path: String,
	text: Option<String>,
}

/// Maintains workspace index lifecycle.
#[derive(Debug, Clone)]
pub struct WorkspaceIndexer {
	workspace_root: PathBuf,
	snapshot: Arc<RwLock<WorkspaceSnapshot>>,
}

impl WorkspaceIndexer {
	/// Creates a new indexer for the given workspace root.
	pub fn new(workspace_root: PathBuf) -> Self {
		Self {
			workspace_root,
			snapshot: Arc::new(RwLock::new(WorkspaceSnapshot::default())),
		}
	}

	/// Returns the workspace root used by the indexer.
	pub fn workspace_root(&self) -> &Path {
		&self.workspace_root
	}

	/// Rebuilds the in-memory workspace index from disk.
	pub fn rebuild(&self) -> Result<IndexBuildStats> {
		let snapshot = build_snapshot(&self.workspace_root)?;
		let stats = IndexBuildStats {
			indexed_files: snapshot.files.len(),
			indexed_directories: snapshot.directories.len(),
			indexed_text_files: snapshot
				.files
				.iter()
				.filter(|file| file.text.is_some())
				.count(),
		};

		let mut state = self
			.snapshot
			.write()
			.map_err(|_| anyhow!("workspace index lock poisoned"))?;
		*state = snapshot;
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
			let Some(text) = &file.text else {
				continue;
			};
			collect_literal_matches(
				file.path.as_str(),
				text,
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
			let Some(text) = &file.text else {
				continue;
			};
			collect_regex_matches(file.path.as_str(), text, &regex, &mut matches);
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
		let snapshot = self
			.snapshot
			.read()
			.map_err(|_| anyhow!("workspace index lock poisoned"))?;
		let Some(file) = snapshot
			.files
			.iter()
			.find(|file| file.path == normalized_path)
		else {
			bail!("indexed file not found: {normalized_path}");
		};
		let Some(text) = &file.text else {
			bail!("file is not available as UTF-8 text: {normalized_path}");
		};

		let lines = text.lines().collect::<Vec<_>>();
		if start_line > lines.len() || end_line > lines.len() {
			bail!("requested line range is out of bounds for file: {normalized_path}");
		}

		let slice = &lines[(start_line - 1)..end_line];
		Ok(slice.join("\n"))
	}
}

fn build_snapshot(workspace_root: &Path) -> Result<WorkspaceSnapshot> {
	let mut files = Vec::new();
	let mut directories = Vec::new();
	scan_directory(workspace_root, workspace_root, &mut files, &mut directories)?;

	files.sort_by(|left, right| compare_paths(left.path.as_str(), right.path.as_str()));
	directories.sort_by(|left, right| compare_paths(left, right));

	Ok(WorkspaceSnapshot { files, directories })
}

fn scan_directory(
	workspace_root: &Path,
	current_dir: &Path,
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
			if should_skip_directory(relative.as_str()) {
				continue;
			}

			directories.push(relative);
			scan_directory(workspace_root, &path, files, directories)?;
			continue;
		}

		if file_type.is_file() {
			let text = fs::read_to_string(&path).ok();
			files.push(IndexedFile {
				path: relative,
				text,
			});
		}
	}

	Ok(())
}

fn sorted_entries(path: &Path) -> Result<Vec<fs::DirEntry>> {
	let mut entries = fs::read_dir(path)
		.with_context(|| format!("failed to read directory {}", path.display()))?
		.collect::<std::result::Result<Vec<_>, _>>()
		.with_context(|| format!("failed to enumerate directory {}", path.display()))?;

	entries.sort_by(|left, right| {
		compare_paths(
			left.file_name().to_string_lossy().as_ref(),
			right.file_name().to_string_lossy().as_ref(),
		)
	});

	Ok(entries)
}

fn should_skip_directory(relative_path: &str) -> bool {
	relative_path == ".git"
		|| relative_path.starts_with(".git/")
		|| relative_path == ".agents"
		|| relative_path.starts_with(".agents/")
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

fn normalize_relative(path: impl AsRef<Path>) -> String {
	path.as_ref()
		.to_string_lossy()
		.replace('\\', "/")
		.trim_start_matches("./")
		.to_string()
}
