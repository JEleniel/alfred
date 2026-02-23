//! Local memory storage service skeleton.

use std::path::{Path, PathBuf};

/// Represents configured memory store locations.
#[derive(Debug, Clone)]
pub struct MemoryStore {
	user_store_path: PathBuf,
	workspace_store_path: PathBuf,
}

impl MemoryStore {
	/// Creates memory store paths for user and workspace scopes.
	pub fn new(user_store_path: PathBuf, workspace_store_path: PathBuf) -> Self {
		Self {
			user_store_path,
			workspace_store_path,
		}
	}

	/// Returns the configured user store path.
	pub fn user_store_path(&self) -> &Path {
		&self.user_store_path
	}

	/// Returns the configured workspace store path.
	pub fn workspace_store_path(&self) -> &Path {
		&self.workspace_store_path
	}
}
