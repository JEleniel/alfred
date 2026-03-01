//! File operation service implementation.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::configuration::is_workspace_relative_path;
use crate::errors::AlfredError;
use crate::services::workspace_files::read_optional_utf8_text_capped_labeled;

const MAX_PATCH_SOURCE_BYTES: u64 = 8 * 1024 * 1024;

/// Executes safe workspace-scoped file mutations.
#[derive(Debug, Clone)]
pub struct FileOperationsEngine {
	workspace_root: PathBuf,
	last_patch: Arc<Mutex<Option<RevertPatchState>>>,
}

impl FileOperationsEngine {
	/// Creates a file operations engine bound to a workspace root.
	pub fn new(workspace_root: PathBuf) -> Self {
		Self {
			workspace_root,
			last_patch: Arc::new(Mutex::new(None)),
		}
	}

	/// Returns the workspace boundary root.
	pub fn workspace_root(&self) -> &Path {
		&self.workspace_root
	}

	/// Applies one or more text patches to workspace files.
	pub fn apply_patches(
		&self,
		patches: Vec<PatchRequest>,
		dry_run: bool,
	) -> Result<Vec<PatchFileResult>, AlfredError> {
		let mut results = Vec::new();
		let mut undo_entries = Vec::new();

		for request in patches {
			match self.apply_single_patch(&request, dry_run) {
				Ok((result, undo)) => {
					if let Some(undo) = undo {
						undo_entries.push(undo);
					}
					results.push(result);
				}
				Err(error) => {
					results.push(PatchFileResult {
						path: request.path,
						patched: false,
						bytes_written: None,
						conflicts: Some(vec![json!({
							"kind": "error",
							"message": error.to_string(),
						})]),
						warnings: None,
					});
				}
			}
		}

		if !dry_run && !undo_entries.is_empty() {
			let mut guard = self
				.last_patch
				.lock()
				.map_err(|_| AlfredError::Internal("patch state lock poisoned".to_string()))?;
			*guard = Some(RevertPatchState {
				entries: undo_entries,
			});
		}

		Ok(results)
	}

	/// Reverts the most recently applied patch batch (best-effort per file).
	pub fn revert_last_patch(&self, dry_run: bool) -> Result<Vec<RevertFileResult>, AlfredError> {
		let state = {
			let guard = self
				.last_patch
				.lock()
				.map_err(|_| AlfredError::Internal("patch state lock poisoned".to_string()))?;
			guard.clone()
		};

		let Some(state) = state else {
			return Err(AlfredError::InvalidArgument(
				"no patch available to revert".to_string(),
			));
		};

		let mut results = Vec::new();
		for entry in &state.entries {
			match self.revert_single_entry(entry, dry_run) {
				Ok(result) => results.push(result),
				Err(error) => results.push(RevertFileResult {
					path: entry.path.clone(),
					reverted: false,
					bytes_written: None,
					conflicts: Some(vec![json!({
						"kind": "error",
						"message": error.to_string(),
					})]),
				}),
			}
		}

		if !dry_run {
			let mut guard = self
				.last_patch
				.lock()
				.map_err(|_| AlfredError::Internal("patch state lock poisoned".to_string()))?;
			*guard = None;
		}

		Ok(results)
	}

	fn apply_single_patch(
		&self,
		request: &PatchRequest,
		dry_run: bool,
	) -> Result<(PatchFileResult, Option<RevertPatchEntry>), AlfredError> {
		let path = normalize_workspace_path(&request.path)?;
		let absolute_path = self.workspace_root.join(path.as_str());
		let read_path =
			match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
				self.workspace_root.as_path(),
				absolute_path.as_path(),
			)? {
				Some(resolved) => resolved,
				None => absolute_path.clone(),
			};
		let original = read_optional_utf8(path.as_str(), read_path.as_path())?;

		let patch_text = ensure_patch_includes_header(path.as_str(), request.patch.as_str());
		let mut parsed = mpatch::parse_single_patch(patch_text.as_str()).map_err(|error| {
			AlfredError::InvalidArgument(format!("failed to parse patch for {path}: {error}"))
		})?;
		parsed.file_path = PathBuf::from(path.as_str());

		let options = if dry_run {
			mpatch::ApplyOptions::dry_run()
		} else {
			mpatch::ApplyOptions::new()
		};
		let applied = mpatch::apply_patch_to_content(&parsed, original.as_deref(), &options);
		if !applied.report.all_applied_cleanly() {
			let conflicts = render_hunk_failures(&applied.report);
			return Ok((
				PatchFileResult {
					path,
					patched: false,
					bytes_written: None,
					conflicts: Some(conflicts),
					warnings: None,
				},
				None,
			));
		}

		let warnings =
			detect_duplicate_content_risk(path.as_str(), original.as_deref(), &applied.new_content);
		if dry_run {
			return Ok((
				PatchFileResult {
					path,
					patched: true,
					bytes_written: None,
					conflicts: None,
					warnings: warnings.map(|warning| vec![warning]),
				},
				None,
			));
		}

		let write_path = crate::workspace_boundary::resolve_write_target_within_workspace_root(
			self.workspace_root.as_path(),
			absolute_path.as_path(),
		)?;
		let bytes_written = write_text_atomic(write_path.as_path(), &applied.new_content)?;
		let undo_entry = RevertPatchEntry {
			path: path.clone(),
			reverse_patch: parsed.invert(),
			after_sha256: sha256_hex(&applied.new_content),
		};
		Ok((
			PatchFileResult {
				path,
				patched: true,
				bytes_written: Some(bytes_written),
				conflicts: None,
				warnings: warnings.map(|warning| vec![warning]),
			},
			Some(undo_entry),
		))
	}

	fn revert_single_entry(
		&self,
		entry: &RevertPatchEntry,
		dry_run: bool,
	) -> Result<RevertFileResult, AlfredError> {
		let absolute_path = self.workspace_root.join(entry.path.as_str());
		let read_path =
			match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
				self.workspace_root.as_path(),
				absolute_path.as_path(),
			)? {
				Some(resolved) => resolved,
				None => absolute_path.clone(),
			};
		let current = read_optional_utf8(entry.path.as_str(), read_path.as_path())?;
		let Some(current) = current else {
			return Ok(RevertFileResult {
				path: entry.path.clone(),
				reverted: false,
				bytes_written: None,
				conflicts: Some(vec![json!({
					"kind": "not_found",
					"message": "file missing while attempting revert",
				})]),
			});
		};

		let current_hash = sha256_hex(current.as_str());
		if current_hash != entry.after_sha256 {
			return Ok(RevertFileResult {
				path: entry.path.clone(),
				reverted: false,
				bytes_written: None,
				conflicts: Some(vec![json!({
					"kind": "content_mismatch",
					"message": "file changed since last patch; refusing to revert",
				})]),
			});
		}

		let options = if dry_run {
			mpatch::ApplyOptions::dry_run()
		} else {
			mpatch::ApplyOptions::new()
		};
		let reverted =
			mpatch::apply_patch_to_content(&entry.reverse_patch, Some(current.as_str()), &options);
		if !reverted.report.all_applied_cleanly() {
			return Ok(RevertFileResult {
				path: entry.path.clone(),
				reverted: false,
				bytes_written: None,
				conflicts: Some(render_hunk_failures(&reverted.report)),
			});
		}
		if dry_run {
			return Ok(RevertFileResult {
				path: entry.path.clone(),
				reverted: true,
				bytes_written: None,
				conflicts: None,
			});
		}

		let write_path = crate::workspace_boundary::resolve_write_target_within_workspace_root(
			self.workspace_root.as_path(),
			absolute_path.as_path(),
		)?;
		let bytes_written = write_text_atomic(write_path.as_path(), reverted.new_content.as_str())?;
		Ok(RevertFileResult {
			path: entry.path.clone(),
			reverted: true,
			bytes_written: Some(bytes_written),
			conflicts: None,
		})
	}
}

#[derive(Debug, Clone)]
pub struct PatchRequest {
	pub path: String,
	pub patch: String,
}

#[derive(Debug, Clone)]
pub struct PatchFileResult {
	pub path: String,
	pub patched: bool,
	pub bytes_written: Option<u64>,
	pub conflicts: Option<Vec<Value>>,
	pub warnings: Option<Vec<Value>>,
}

#[derive(Debug, Clone)]
pub struct RevertFileResult {
	pub path: String,
	pub reverted: bool,
	pub bytes_written: Option<u64>,
	pub conflicts: Option<Vec<Value>>,
}

#[derive(Debug, Clone)]
struct RevertPatchState {
	entries: Vec<RevertPatchEntry>,
}

#[derive(Debug, Clone)]
struct RevertPatchEntry {
	path: String,
	reverse_patch: mpatch::Patch,
	after_sha256: String,
}

fn normalize_workspace_path(raw_path: &str) -> Result<String, AlfredError> {
	let trimmed = crate::path_encoding::normalize_inbound_separators(raw_path.trim());
	if trimmed.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"path must not be empty".to_string(),
		));
	}
	if !is_workspace_relative_path(trimmed.as_str()) {
		return Err(AlfredError::WorkspaceBoundaryViolation(format!(
			"path escapes workspace boundary: {raw_path}"
		)));
	}
	Ok(trimmed)
}

fn ensure_patch_includes_header(path: &str, patch: &str) -> String {
	let patch = patch.trim();
	if patch.contains("--- ") && patch.contains("+++ ") {
		return patch.to_string();
	}

	format!("```diff\n--- a/{path}\n+++ b/{path}\n{patch}\n```\n",)
}

fn read_optional_utf8(
	relative_path: &str,
	absolute_path: &Path,
) -> Result<Option<String>, AlfredError> {
	read_optional_utf8_text_capped_labeled(absolute_path, relative_path, MAX_PATCH_SOURCE_BYTES)
		.map_err(|error| {
			AlfredError::IoError(format!("failed to read file {relative_path}: {error}"))
		})
}

fn write_text_atomic(path: &Path, content: &str) -> Result<u64, AlfredError> {
	let parent = path.parent().ok_or_else(|| {
		AlfredError::InvalidArgument(format!("path has no parent: {}", path.display()))
	})?;
	fs::create_dir_all(parent).map_err(|error| {
		AlfredError::IoError(format!(
			"failed to create parent directory {}: {error}",
			parent.display()
		))
	})?;

	let tmp_path = path.with_extension(format!("tmp-{}", std::process::id()));
	fs::write(&tmp_path, content).map_err(|error| {
		AlfredError::IoError(format!(
			"failed to write temporary file {}: {error}",
			tmp_path.display()
		))
	})?;

	match fs::rename(&tmp_path, path) {
		Ok(()) => Ok(content.len() as u64),
		Err(error) => {
			if error.kind() == std::io::ErrorKind::AlreadyExists {
				let _ = fs::remove_file(path);
				fs::rename(&tmp_path, path).map_err(|rename_error| {
					AlfredError::IoError(format!(
						"failed to replace file {}: {rename_error}",
						path.display()
					))
				})?;
				return Ok(content.len() as u64);
			}

			Err(AlfredError::IoError(format!(
				"failed to replace file {}: {error}",
				path.display()
			)))
		}
	}
}

fn sha256_hex(content: &str) -> String {
	let mut digest = Sha256::new();
	digest.update(content.as_bytes());
	hex::encode(digest.finalize())
}

fn render_hunk_failures(report: &mpatch::ApplyResult) -> Vec<Value> {
	report
		.failures()
		.iter()
		.map(|failure| {
			json!({
				"hunk_index": failure.hunk_index,
				"reason": format!("{:?}", failure.reason),
			})
		})
		.collect()
}

fn detect_duplicate_content_risk(
	path: &str,
	original: Option<&str>,
	new_content: &str,
) -> Option<Value> {
	let original = original?;
	let original = original.trim();
	if original.len() < 64 {
		return None;
	}
	let occurrences = new_content.matches(original).count();
	if occurrences < 2 {
		return None;
	}

	Some(json!({
		"kind": "duplicate_content_risk",
		"path": path,
		"details": {
			"original_bytes": original.len(),
			"occurrences": occurrences,
		},
	}))
}
