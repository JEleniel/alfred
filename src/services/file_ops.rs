//! File operation service skeleton.

use std::path::{Path, PathBuf};

/// Executes safe workspace-scoped file mutations.
#[derive(Debug, Clone)]
pub struct FileOperationsEngine {
	workspace_root: PathBuf,
}

impl FileOperationsEngine {
	/// Creates a file operations engine bound to a workspace root.
	pub fn new(workspace_root: PathBuf) -> Self {
		Self { workspace_root }
	}

	/// Returns the workspace boundary root.
	pub fn workspace_root(&self) -> &Path {
		&self.workspace_root
	}
}
