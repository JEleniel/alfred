//! Background job service.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::errors::{AlfredError, ToolError};

/// Tracks and allocates background job identifiers.
#[derive(Debug)]
pub struct JobManager {
	next_id: AtomicU64,
	bulk_jobs: Mutex<HashMap<String, FsBulkJobHandle>>,
}

impl Default for JobManager {
	fn default() -> Self {
		Self::new()
	}
}

impl JobManager {
	/// Creates a new in-memory job manager.
	pub fn new() -> Self {
		Self {
			next_id: AtomicU64::new(1),
			bulk_jobs: Mutex::new(HashMap::new()),
		}
	}

	/// Allocates the next job id as a string.
	pub fn allocate_job_id(&self) -> String {
		let id = self.next_id.fetch_add(1, Ordering::Relaxed);
		format!("job-{id}")
	}

	pub fn start_fs_bulk_background(
		&self,
		workspace_root: PathBuf,
		operations: Vec<FsBulkOperation>,
	) -> Result<FsBulkStatus, AlfredError> {
		if operations.is_empty() {
			return Err(AlfredError::InvalidArgument(
				"fs.bulk.operations must not be empty".to_string(),
			));
		}

		let operation_id = Uuid::new_v4().to_string();
		let handle = FsBulkJobHandle::new(operation_id.clone(), operations.len());
		{
			let mut guard = self
				.bulk_jobs
				.lock()
				.map_err(|_| AlfredError::Internal("bulk job manager lock poisoned".to_string()))?;
			guard.insert(operation_id.clone(), handle.clone());
		}

		let thread_handle = handle.clone();

		std::thread::spawn(move || {
			run_fs_bulk_job(workspace_root.as_path(), operations, &thread_handle);
		});

		Ok(handle.snapshot(None))
	}

	pub fn execute_fs_bulk_sync(
		&self,
		workspace_root: &Path,
		operations: Vec<FsBulkOperation>,
		dry_run: bool,
	) -> Result<FsBulkStatus, AlfredError> {
		if operations.is_empty() {
			return Err(AlfredError::InvalidArgument(
				"fs.bulk.operations must not be empty".to_string(),
			));
		}

		let operation_id = Uuid::new_v4().to_string();
		let handle = FsBulkJobHandle::new(operation_id.clone(), operations.len());
		handle.set_state(FsBulkState::Running);
		for (index, operation) in operations.into_iter().enumerate() {
			let result = execute_fs_bulk_operation(workspace_root, &operation, dry_run);
			handle.record_item(index, operation, result);
			std::thread::yield_now();
		}
		handle.finalize();
		Ok(handle.snapshot(Some(FsBulkItemsMode::All)))
	}

	pub fn fs_bulk_status(&self, operation_id: &str) -> Result<FsBulkStatus, AlfredError> {
		let handle = self.lookup_bulk_job(operation_id)?;
		Ok(handle.snapshot(Some(FsBulkItemsMode::All)))
	}

	pub fn fs_bulk_cancel(&self, operation_id: &str) -> Result<FsBulkStatus, AlfredError> {
		let handle = self.lookup_bulk_job(operation_id)?;
		handle.cancel();
		Ok(handle.snapshot(Some(FsBulkItemsMode::All)))
	}

	fn lookup_bulk_job(&self, operation_id: &str) -> Result<FsBulkJobHandle, AlfredError> {
		let guard = self
			.bulk_jobs
			.lock()
			.map_err(|_| AlfredError::Internal("bulk job manager lock poisoned".to_string()))?;
		guard.get(operation_id).cloned().ok_or_else(|| {
			AlfredError::NotFound(format!("bulk operation not found: {operation_id}"))
		})
	}
}

#[derive(Debug, Clone)]
pub enum FsBulkOperation {
	Move {
		from: String,
		to: String,
		overwrite: bool,
		create_parents: bool,
	},
	Copy {
		from: String,
		to: String,
		overwrite: bool,
		create_parents: bool,
	},
	Delete {
		path: String,
		recursive: bool,
	},
}

#[derive(Debug, Clone, Copy, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FsBulkState {
	Queued,
	Running,
	Succeeded,
	Failed,
	Canceled,
	Partial,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct FsBulkSummary {
	pub total: usize,
	pub completed: usize,
	pub failed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsBulkItem {
	pub index: usize,
	pub kind: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub from: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub to: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub path: Option<String>,
	pub succeeded: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub result: Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub error: Option<ToolError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsBulkStatus {
	pub operation_id: String,
	pub state: FsBulkState,
	pub summary: FsBulkSummary,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub items: Option<Vec<FsBulkItem>>,
}

#[derive(Debug, Clone, Copy)]
enum FsBulkItemsMode {
	All,
}

#[derive(Debug, Clone)]
struct FsBulkJobHandle {
	cancel: Arc<AtomicBool>,
	inner: Arc<Mutex<FsBulkJobInner>>,
}

#[derive(Debug)]
struct FsBulkJobInner {
	operation_id: String,
	state: FsBulkState,
	summary: FsBulkSummary,
	items: Vec<FsBulkItem>,
}

impl FsBulkJobHandle {
	fn new(operation_id: String, total: usize) -> Self {
		Self {
			cancel: Arc::new(AtomicBool::new(false)),
			inner: Arc::new(Mutex::new(FsBulkJobInner {
				operation_id,
				state: FsBulkState::Queued,
				summary: FsBulkSummary {
					total,
					completed: 0,
					failed: 0,
				},
				items: Vec::new(),
			})),
		}
	}

	fn cancel(&self) {
		self.cancel.store(true, Ordering::Relaxed);
		let mut guard = self
			.inner
			.lock()
			.unwrap_or_else(|poisoned| poisoned.into_inner());
		if matches!(guard.state, FsBulkState::Queued | FsBulkState::Running) {
			guard.state = FsBulkState::Canceled;
		}
	}

	fn is_cancel_requested(&self) -> bool {
		self.cancel.load(Ordering::Relaxed)
	}

	fn set_state(&self, state: FsBulkState) {
		let mut guard = self
			.inner
			.lock()
			.unwrap_or_else(|poisoned| poisoned.into_inner());
		guard.state = state;
	}

	fn record_item(
		&self,
		index: usize,
		operation: FsBulkOperation,
		result: Result<Value, AlfredError>,
	) {
		let item = match operation {
			FsBulkOperation::Move {
				from,
				to,
				overwrite: _,
				create_parents: _,
			} => build_item(index, "move", Some(from), Some(to), None, result),
			FsBulkOperation::Copy {
				from,
				to,
				overwrite: _,
				create_parents: _,
			} => build_item(index, "copy", Some(from), Some(to), None, result),
			FsBulkOperation::Delete { path, recursive: _ } => {
				build_item(index, "delete", None, None, Some(path), result)
			}
		};

		let mut guard = self
			.inner
			.lock()
			.unwrap_or_else(|poisoned| poisoned.into_inner());
		if item.succeeded {
			guard.summary.completed += 1;
		} else {
			guard.summary.failed += 1;
		}
		guard.items.push(item);
	}

	fn finalize(&self) {
		let mut guard = self
			.inner
			.lock()
			.unwrap_or_else(|poisoned| poisoned.into_inner());
		if guard.state == FsBulkState::Canceled {
			return;
		}
		let completed = guard.summary.completed;
		let failed = guard.summary.failed;
		guard.state = if failed == 0 {
			FsBulkState::Succeeded
		} else if completed == 0 {
			FsBulkState::Failed
		} else {
			FsBulkState::Partial
		};
	}

	fn snapshot(&self, items: Option<FsBulkItemsMode>) -> FsBulkStatus {
		let guard = self
			.inner
			.lock()
			.unwrap_or_else(|poisoned| poisoned.into_inner());
		FsBulkStatus {
			operation_id: guard.operation_id.clone(),
			state: guard.state,
			summary: guard.summary.clone(),
			items: items
				.map(|_| guard.items.clone())
				.filter(|items| !items.is_empty()),
		}
	}
}

fn build_item(
	index: usize,
	kind: &str,
	from: Option<String>,
	to: Option<String>,
	path: Option<String>,
	result: Result<Value, AlfredError>,
) -> FsBulkItem {
	match result {
		Ok(value) => FsBulkItem {
			index,
			kind: kind.to_string(),
			from,
			to,
			path,
			succeeded: true,
			result: Some(value),
			error: None,
		},
		Err(error) => FsBulkItem {
			index,
			kind: kind.to_string(),
			from,
			to,
			path,
			succeeded: false,
			result: None,
			error: Some(error.into()),
		},
	}
}

fn run_fs_bulk_job(
	workspace_root: &Path,
	operations: Vec<FsBulkOperation>,
	handle: &FsBulkJobHandle,
) {
	if handle.is_cancel_requested() {
		handle.set_state(FsBulkState::Canceled);
		return;
	}

	handle.set_state(FsBulkState::Running);
	for (index, operation) in operations.into_iter().enumerate() {
		if handle.is_cancel_requested() {
			handle.set_state(FsBulkState::Canceled);
			break;
		}

		let result = execute_fs_bulk_operation(workspace_root, &operation, false);
		handle.record_item(index, operation, result);
		std::thread::yield_now();
	}

	handle.finalize();
}

fn execute_fs_bulk_operation(
	workspace_root: &Path,
	operation: &FsBulkOperation,
	dry_run: bool,
) -> Result<Value, AlfredError> {
	match operation {
		FsBulkOperation::Move {
			from,
			to,
			overwrite,
			create_parents,
		} => execute_move(
			workspace_root,
			from,
			to,
			*overwrite,
			*create_parents,
			dry_run,
		),
		FsBulkOperation::Copy {
			from,
			to,
			overwrite,
			create_parents,
		} => execute_copy(
			workspace_root,
			from,
			to,
			*overwrite,
			*create_parents,
			dry_run,
		),
		FsBulkOperation::Delete { path, recursive } => {
			execute_delete(workspace_root, path, *recursive, dry_run)
		}
	}
}

fn normalize_workspace_path(raw_path: &str) -> Result<String, AlfredError> {
	let raw = crate::path_encoding::normalize_inbound_separators(raw_path.trim());
	if raw.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"path must not be empty".to_string(),
		));
	}
	if is_absolute_path(raw.as_str()) {
		return Err(AlfredError::InvalidArgument(format!(
			"absolute paths are not allowed: {raw_path}"
		)));
	}

	let mut segments = Vec::new();
	for segment in raw.split('/') {
		if segment.is_empty() || segment == "." {
			continue;
		}
		if segment == ".." {
			return Err(AlfredError::WorkspaceBoundaryViolation(format!(
				"path must be workspace-relative and must not contain '..': {raw_path}"
			)));
		}
		segments.push(segment);
	}

	let normalized = segments.join("/");
	if normalized.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"path must not be empty".to_string(),
		));
	}
	if !crate::configuration::is_workspace_relative_path(normalized.as_str()) {
		return Err(AlfredError::WorkspaceBoundaryViolation(format!(
			"path must be workspace-relative: {raw_path}"
		)));
	}
	Ok(normalized)
}

fn is_absolute_path(path: &str) -> bool {
	if path.starts_with('/') {
		return true;
	}
	if path.len() >= 3 {
		let bytes = path.as_bytes();
		if bytes[1] == b':' && bytes[2] == b'/' {
			return true;
		}
	}
	false
}

fn resolve_existing_path(workspace_root: &Path, relative: &str) -> Result<PathBuf, AlfredError> {
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, relative);
	match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	)? {
		Some(_) => Ok(candidate),
		None => Err(AlfredError::NotFound(format!("path not found: {relative}"))),
	}
}

fn resolve_write_target(workspace_root: &Path, relative: &str) -> Result<PathBuf, AlfredError> {
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, relative);
	crate::workspace_boundary::resolve_write_target_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	)
}

fn execute_move(
	workspace_root: &Path,
	from_raw: &str,
	to_raw: &str,
	overwrite: bool,
	create_parents: bool,
	dry_run: bool,
) -> Result<Value, AlfredError> {
	let from = normalize_workspace_path(from_raw)?;
	let to = normalize_workspace_path(to_raw)?;
	let from_path = resolve_existing_path(workspace_root, from.as_str())?;

	let metadata = std::fs::symlink_metadata(from_path.as_path())
		.map_err(|error| AlfredError::IoError(format!("failed to stat {from}: {error}")))?;
	if metadata.file_type().is_symlink() {
		return Err(AlfredError::InvalidArgument(format!(
			"move does not support symlinks: {from}"
		)));
	}

	let to_path = resolve_write_target(workspace_root, to.as_str())?;
	ensure_destination_parent(to_path.as_path(), to.as_str(), create_parents, dry_run)?;
	reconcile_destination(to_path.as_path(), to.as_str(), overwrite, dry_run)?;

	if !dry_run {
		match std::fs::rename(from_path.as_path(), to_path.as_path()) {
			Ok(()) => {}
			Err(error) => {
				if metadata.is_dir() {
					copy_dir_no_follow(from_path.as_path(), to_path.as_path())?;
					delete_dir_no_follow(from_path.as_path())?;
				} else {
					copy_file(from_path.as_path(), to_path.as_path())?;
					std::fs::remove_file(from_path.as_path()).map_err(|io_error| {
						AlfredError::IoError(format!(
							"failed to delete source file after copy {from}: {io_error}"
						))
					})?;
				}
				let _ = error; // keep deterministic fallback semantics regardless of platform error kinds.
			}
		}
	}

	Ok(serde_json::json!({"from": from, "to": to, "moved": true}))
}

fn execute_copy(
	workspace_root: &Path,
	from_raw: &str,
	to_raw: &str,
	overwrite: bool,
	create_parents: bool,
	dry_run: bool,
) -> Result<Value, AlfredError> {
	let from = normalize_workspace_path(from_raw)?;
	let to = normalize_workspace_path(to_raw)?;
	let from_path = resolve_existing_path(workspace_root, from.as_str())?;
	let metadata = std::fs::symlink_metadata(from_path.as_path())
		.map_err(|error| AlfredError::IoError(format!("failed to stat {from}: {error}")))?;
	if metadata.file_type().is_symlink() {
		return Err(AlfredError::InvalidArgument(format!(
			"copy does not support symlinks: {from}"
		)));
	}

	let to_path = resolve_write_target(workspace_root, to.as_str())?;
	ensure_destination_parent(to_path.as_path(), to.as_str(), create_parents, dry_run)?;
	reconcile_destination(to_path.as_path(), to.as_str(), overwrite, dry_run)?;

	if !dry_run {
		if metadata.is_dir() {
			std::fs::create_dir_all(to_path.as_path()).map_err(|error| {
				AlfredError::IoError(format!("failed to create directory {to}: {error}"))
			})?;
			copy_dir_no_follow(from_path.as_path(), to_path.as_path())?;
		} else {
			copy_file(from_path.as_path(), to_path.as_path())?;
		}
	}

	Ok(serde_json::json!({"from": from, "to": to, "copied": true}))
}

fn execute_delete(
	workspace_root: &Path,
	path_raw: &str,
	recursive: bool,
	dry_run: bool,
) -> Result<Value, AlfredError> {
	let path = normalize_workspace_path(path_raw)?;
	let absolute = crate::path_encoding::resolve_workspace_relative_path(workspace_root, &path);
	match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
		workspace_root,
		absolute.as_path(),
	)? {
		Some(_) => {}
		None => return Ok(serde_json::json!({"path": path, "deleted": false})),
	}

	let metadata = std::fs::symlink_metadata(absolute.as_path())
		.map_err(|error| AlfredError::IoError(format!("failed to stat {path}: {error}")))?;

	if metadata.is_dir() {
		if !recursive {
			return Err(AlfredError::InvalidArgument(format!(
				"delete requires recursive=true for directories: {path}"
			)));
		}
		if !dry_run {
			delete_dir_no_follow(absolute.as_path())?;
		}
		return Ok(serde_json::json!({"path": path, "deleted": true}));
	}

	if !dry_run {
		std::fs::remove_file(absolute.as_path()).map_err(|error| {
			AlfredError::IoError(format!("failed to delete file {path}: {error}"))
		})?;
	}

	Ok(serde_json::json!({"path": path, "deleted": true}))
}

fn ensure_destination_parent(
	to_path: &Path,
	to_rel: &str,
	create_parents: bool,
	dry_run: bool,
) -> Result<(), AlfredError> {
	let Some(parent) = to_path.parent() else {
		return Err(AlfredError::InvalidArgument(format!(
			"destination has no parent: {to_rel}"
		)));
	};
	if parent.exists() {
		return Ok(());
	}
	if !create_parents {
		return Err(AlfredError::NotFound(format!(
			"destination parent does not exist: {to_rel}"
		)));
	}
	if !dry_run {
		std::fs::create_dir_all(parent).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to create destination parents for {to_rel}: {error}"
			))
		})?;
	}
	Ok(())
}

fn reconcile_destination(
	to_path: &Path,
	to_rel: &str,
	overwrite: bool,
	dry_run: bool,
) -> Result<(), AlfredError> {
	match std::fs::symlink_metadata(to_path) {
		Ok(metadata) => {
			if !overwrite {
				return Err(AlfredError::Conflict(format!(
					"destination already exists: {to_rel}"
				)));
			}

			if dry_run {
				return Ok(());
			}
			if metadata.is_dir() {
				delete_dir_no_follow(to_path)?;
			} else {
				std::fs::remove_file(to_path).map_err(|error| {
					AlfredError::IoError(format!("failed to remove destination {to_rel}: {error}"))
				})?;
			}
			Ok(())
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
		Err(error) => Err(AlfredError::IoError(format!(
			"failed to stat destination {to_rel}: {error}"
		))),
	}
}

fn copy_file(from: &Path, to: &Path) -> Result<(), AlfredError> {
	std::fs::copy(from, to)
		.map(|_| ())
		.map_err(|error| AlfredError::IoError(format!("failed to copy file: {error}")))
}

fn copy_dir_no_follow(from: &Path, to: &Path) -> Result<(), AlfredError> {
	let entries = std::fs::read_dir(from).map_err(|error| {
		AlfredError::IoError(format!(
			"failed to read directory {}: {error}",
			from.display()
		))
	})?;

	for entry in entries {
		let entry = entry.map_err(|error| {
			AlfredError::IoError(format!(
				"failed to read directory entry {}: {error}",
				from.display()
			))
		})?;
		let file_type = entry.file_type().map_err(|error| {
			AlfredError::IoError(format!(
				"failed to stat directory entry {}: {error}",
				entry.path().display()
			))
		})?;
		if file_type.is_symlink() {
			return Err(AlfredError::InvalidArgument(format!(
				"copy does not support symlink entries: {}",
				entry.path().display()
			)));
		}

		let destination = to.join(entry.file_name());
		if file_type.is_dir() {
			std::fs::create_dir_all(destination.as_path()).map_err(|error| {
				AlfredError::IoError(format!(
					"failed to create directory {}: {error}",
					destination.display()
				))
			})?;
			copy_dir_no_follow(entry.path().as_path(), destination.as_path())?;
		} else {
			copy_file(entry.path().as_path(), destination.as_path())?;
		}
	}
	Ok(())
}

fn delete_dir_no_follow(path: &Path) -> Result<(), AlfredError> {
	let entries = std::fs::read_dir(path).map_err(|error| {
		AlfredError::IoError(format!(
			"failed to read directory {}: {error}",
			path.display()
		))
	})?;

	for entry in entries {
		let entry = entry.map_err(|error| {
			AlfredError::IoError(format!(
				"failed to read directory entry {}: {error}",
				path.display()
			))
		})?;
		let file_type = entry.file_type().map_err(|error| {
			AlfredError::IoError(format!(
				"failed to stat directory entry {}: {error}",
				entry.path().display()
			))
		})?;
		let child = entry.path();
		if file_type.is_dir() && !file_type.is_symlink() {
			delete_dir_no_follow(child.as_path())?;
			std::fs::remove_dir(child.as_path()).map_err(|error| {
				AlfredError::IoError(format!(
					"failed to remove directory {}: {error}",
					child.display()
				))
			})?;
		} else {
			std::fs::remove_file(child.as_path()).map_err(|error| {
				AlfredError::IoError(format!(
					"failed to remove file {}: {error}",
					child.display()
				))
			})?;
		}
	}

	std::fs::remove_dir(path).map_err(|error| {
		AlfredError::IoError(format!(
			"failed to remove directory {}: {error}",
			path.display()
		))
	})
}
