//! Workspace boundary enforcement helpers.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::errors::AlfredError;

/// Canonical error message for resolved-path boundary violations.
///
/// See `docs/design/MIS-001/Constraint/CNS-016-Symlink_and_JunctionSafe_Boundary_Checks.md`.
pub const SYMLINK_JUNCTION_ESCAPE_MESSAGE: &str = "symlink/junction escapes workspace";

fn canonicalize_path(path: &Path) -> Result<PathBuf, AlfredError> {
	std::fs::canonicalize(path).map_err(|error| {
		AlfredError::IoError(format!(
			"failed to canonicalize {}: {error}",
			path.display()
		))
	})
}

fn ensure_within_root(resolved_root: &Path, resolved_candidate: &Path) -> Result<(), AlfredError> {
	if resolved_candidate.starts_with(resolved_root) {
		return Ok(());
	}

	Err(AlfredError::PermissionDenied(
		SYMLINK_JUNCTION_ESCAPE_MESSAGE.to_string(),
	))
}

/// If `candidate` exists, resolves it to a canonical target and ensures it remains within the
/// canonicalized `workspace_root` boundary.
///
/// Returns `Ok(None)` when the path does not exist.
pub fn try_resolve_existing_path_within_workspace_root(
	workspace_root: &Path,
	candidate: &Path,
) -> Result<Option<PathBuf>, AlfredError> {
	match std::fs::symlink_metadata(candidate) {
		Ok(_) => {}
		Err(error) => {
			if error.kind() == std::io::ErrorKind::NotFound {
				return Ok(None);
			}
			return Err(AlfredError::IoError(format!(
				"failed to stat {}: {error}",
				candidate.display()
			)));
		}
	}

	let resolved_root = canonicalize_path(workspace_root)?;
	let resolved_candidate = canonicalize_path(candidate)?;
	ensure_within_root(resolved_root.as_path(), resolved_candidate.as_path())?;
	Ok(Some(resolved_candidate))
}

fn find_existing_ancestor_with_remainder(
	workspace_root: &Path,
	target: &Path,
) -> Result<(PathBuf, Vec<OsString>), AlfredError> {
	let mut remainder: Vec<OsString> = Vec::new();
	let mut cursor = target.to_path_buf();

	loop {
		if cursor == workspace_root {
			return Ok((cursor, remainder));
		}

		match std::fs::symlink_metadata(cursor.as_path()) {
			Ok(_) => return Ok((cursor, remainder)),
			Err(error) => {
				if error.kind() != std::io::ErrorKind::NotFound {
					return Err(AlfredError::IoError(format!(
						"failed to stat {}: {error}",
						cursor.display()
					)));
				}
			}
		}

		let name = cursor.file_name().ok_or_else(|| {
			AlfredError::Internal(format!(
				"path has no file name while resolving write target: {}",
				target.display()
			))
		})?;
		remainder.push(name.to_os_string());
		cursor = cursor
			.parent()
			.ok_or_else(|| {
				AlfredError::Internal(format!(
					"path has no parent while resolving write target: {}",
					target.display()
				))
			})?
			.to_path_buf();
	}
}

/// Resolves a potentially-nonexistent target path for a write operation and ensures the resolved
/// (canonical) destination remains within the canonicalized workspace root.
///
/// This prevents writing through symlink/junction components that escape the workspace.
pub fn resolve_write_target_within_workspace_root(
	workspace_root: &Path,
	target: &Path,
) -> Result<PathBuf, AlfredError> {
	let resolved_root = canonicalize_path(workspace_root)?;

	if std::fs::symlink_metadata(target).is_ok() {
		let resolved_target = canonicalize_path(target)?;
		ensure_within_root(resolved_root.as_path(), resolved_target.as_path())?;
		return Ok(resolved_target);
	}

	let (ancestor, remainder) = find_existing_ancestor_with_remainder(workspace_root, target)?;
	let resolved_ancestor = canonicalize_path(ancestor.as_path())?;
	ensure_within_root(resolved_root.as_path(), resolved_ancestor.as_path())?;

	let mut resolved_target = resolved_ancestor;
	for component in remainder.into_iter().rev() {
		resolved_target.push(component);
	}
	Ok(resolved_target)
}
