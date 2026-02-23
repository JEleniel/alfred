//! Plan storage service skeleton.

use std::path::{Path, PathBuf};

/// Reads and writes the project plan artifact.
#[derive(Debug, Clone)]
pub struct PlanStore {
	plan_path: PathBuf,
}

impl PlanStore {
	/// Creates a plan store bound to a plan file path.
	pub fn new(plan_path: PathBuf) -> Self {
		Self { plan_path }
	}

	/// Returns the configured plan path.
	pub fn plan_path(&self) -> &Path {
		&self.plan_path
	}
}
