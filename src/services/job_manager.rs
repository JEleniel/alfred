//! Background job service skeleton.

use std::sync::atomic::{AtomicU64, Ordering};

/// Tracks and allocates background job identifiers.
#[derive(Debug)]
pub struct JobManager {
	next_id: AtomicU64,
}

impl JobManager {
	/// Creates a new in-memory job manager.
	pub fn new() -> Self {
		Self {
			next_id: AtomicU64::new(1),
		}
	}

	/// Allocates the next job id as a string.
	pub fn allocate_job_id(&self) -> String {
		let id = self.next_id.fetch_add(1, Ordering::Relaxed);
		format!("job-{id}")
	}
}
