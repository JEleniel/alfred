//! Task execution service skeleton.

use std::path::{Path, PathBuf};

/// Runs tasks within the workspace boundary.
#[derive(Debug, Clone)]
pub struct TaskRunner {
	workspace_root: PathBuf,
}

impl TaskRunner {
	/// Creates a task runner bound to a workspace root.
	pub fn new(workspace_root: PathBuf) -> Self {
		Self { workspace_root }
	}

	/// Returns the workspace root used for task working directory.
	pub fn workspace_root(&self) -> &Path {
		&self.workspace_root
	}
}
