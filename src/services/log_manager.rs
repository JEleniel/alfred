//! Log handling service skeleton.

use std::path::{Path, PathBuf};

/// Manages runtime/tool logs.
#[derive(Debug, Clone)]
pub struct LogManager {
	log_path: PathBuf,
}

impl LogManager {
	/// Creates a log manager using the configured NDJSON log path.
	pub fn new(log_path: PathBuf) -> Self {
		Self { log_path }
	}

	/// Returns the configured log path.
	pub fn log_path(&self) -> &Path {
		&self.log_path
	}
}
