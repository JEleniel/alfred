//! Workspace boundary and path normalization primitives for Alfred.

use std::path::{Component, Path, PathBuf};

use thiserror::Error;

/// Resolves a workspace path into a normalized path.
pub fn normalize_workspace_relative_path(path: &str) -> Result<PathBuf, WorkspaceBoundaryError> {
	let path = Path::new(path);

	normalize_workspace_path(path)
}

fn normalize_workspace_path(path: &Path) -> Result<PathBuf, WorkspaceBoundaryError> {
	let mut normalized = PathBuf::new();

	for component in path.components() {
		match component {
			Component::CurDir => continue,
			Component::ParentDir => {
				if !normalized.pop() {
					return Err(WorkspaceBoundaryError::boundary_violation(
						path.to_string_lossy().as_ref(),
					));
				}
			}
			Component::Normal(part) => push_valid_component(
				&mut normalized,
				part.to_str().ok_or_else(|| {
					WorkspaceBoundaryError::invalid_path(
						path.to_string_lossy().as_ref(),
						"non_utf8_component",
					)
				})?,
				path.to_string_lossy().as_ref(),
			)?,
			Component::RootDir | Component::Prefix(_) => {
				normalized.push(component.as_os_str());
			}
		}
	}

	if normalized.as_os_str().is_empty() {
		return Err(WorkspaceBoundaryError::invalid_path(
			path.to_string_lossy().as_ref(),
			"empty_path",
		));
	}

	Ok(normalized)
}

/// Resolves a workspace-relative write target beneath the workspace root.
pub fn resolve_write_target_within_workspace_root(
	workspace_root: impl AsRef<Path>,
	path: &str,
) -> Result<PathBuf, WorkspaceBoundaryError> {
	let workspace_root = workspace_root.as_ref();
	let candidate = normalize_workspace_path(&workspace_root.join(path))?;

	if candidate.starts_with(workspace_root) {
		return Ok(candidate);
	}

	Err(WorkspaceBoundaryError::boundary_violation(path))
}

/// Resolves an existing path beneath the workspace root and rejects escapes.
pub fn try_resolve_existing_path_within_workspace_root(
	workspace_root: impl AsRef<Path>,
	path: &str,
) -> Result<PathBuf, WorkspaceBoundaryError> {
	let normalized = resolve_write_target_within_workspace_root(workspace_root.as_ref(), path)?;
	let canonical_root = workspace_root
		.as_ref()
		.canonicalize()
		.map_err(|_| WorkspaceBoundaryError::not_found(path))?;
	let canonical_candidate = normalized
		.canonicalize()
		.map_err(|_| WorkspaceBoundaryError::not_found(path))?;

	if !canonical_candidate.starts_with(&canonical_root) {
		return Err(WorkspaceBoundaryError::boundary_violation(path));
	}

	Ok(canonical_candidate)
}

fn push_valid_component(
	target: &mut PathBuf,
	component: &str,
	raw_path: &str,
) -> Result<(), WorkspaceBoundaryError> {
	validate_component_text(component, raw_path)?;
	target.push(component);
	Ok(())
}

fn validate_component_text(component: &str, raw_path: &str) -> Result<(), WorkspaceBoundaryError> {
	if component == "." {
		return Ok(());
	}

	if component == ".." {
		return Err(WorkspaceBoundaryError::boundary_violation(raw_path));
	}

	validate_platform_component_text(component, raw_path)?;

	Ok(())
}

#[cfg(windows)]
fn validate_platform_component_text(
	component: &str,
	raw_path: &str,
) -> Result<(), WorkspaceBoundaryError> {
	if component.ends_with('.') {
		return Err(WorkspaceBoundaryError::invalid_path(
			raw_path,
			"trailing_dot",
		));
	}

	if is_reserved_windows_component(component) {
		return Err(WorkspaceBoundaryError::invalid_path(
			raw_path,
			"reserved_name",
		));
	}

	if component.chars().any(is_forbidden_windows_character) {
		return Err(WorkspaceBoundaryError::invalid_path(
			raw_path,
			"forbidden_character",
		));
	}

	Ok(())
}

#[cfg(not(windows))]
fn validate_platform_component_text(
	_component: &str,
	_raw_path: &str,
) -> Result<(), WorkspaceBoundaryError> {
	Ok(())
}

#[cfg(windows)]
fn is_reserved_windows_component(component: &str) -> bool {
	matches!(
		component.to_ascii_uppercase().as_str(),
		"CON"
			| "PRN" | "AUX"
			| "NUL" | "COM0"
			| "COM1" | "COM2"
			| "COM3" | "COM4"
			| "COM5" | "COM6"
			| "COM7" | "COM8"
			| "COM9" | "LPT0"
			| "LPT1" | "LPT2"
			| "LPT3" | "LPT4"
			| "LPT5" | "LPT6"
			| "LPT7" | "LPT8"
			| "LPT9"
	)
}

#[cfg(windows)]
fn is_forbidden_windows_character(character: char) -> bool {
	matches!(
		character,
		'<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*' | '\0'
	) || character.is_control()
}

/// Canonical workspace boundary error.
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum WorkspaceBoundaryError {
	/// The supplied path is invalid for workspace-boundary processing.
	#[error("invalid workspace path `{path}` ({reason}).")]
	InvalidPath { path: String, reason: String },
	/// The supplied path escapes the configured workspace boundary.
	#[error("workspace boundary violation for `{path}`.")]
	BoundaryViolation { path: String },
	/// The supplied existing path could not be found.
	#[error("workspace path `{path}` was not found.")]
	NotFound { path: String },
}

impl WorkspaceBoundaryError {
	fn invalid_path(path: &str, reason: &str) -> Self {
		Self::InvalidPath {
			path: path.to_owned(),
			reason: reason.to_owned(),
		}
	}

	fn boundary_violation(path: &str) -> Self {
		Self::BoundaryViolation {
			path: path.to_owned(),
		}
	}

	fn not_found(path: &str) -> Self {
		Self::NotFound {
			path: path.to_owned(),
		}
	}
}

#[cfg(test)]
#[path = "workspace_boundary/tests/workspace_boundary_tests.rs"]
mod workspace_boundary_tests;
