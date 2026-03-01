//! Log handling service skeleton.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::errors::AlfredError;

#[derive(Debug, Default)]
struct FollowState {
	active: bool,
}

/// Manages runtime/tool logs.
#[derive(Debug, Clone)]
pub struct LogManager {
	log_path: PathBuf,
	follow_state: Arc<Mutex<FollowState>>,
}

impl LogManager {
	/// Creates a log manager using the configured NDJSON log path.
	pub fn new(log_path: PathBuf) -> Self {
		Self {
			log_path,
			follow_state: Arc::new(Mutex::new(FollowState::default())),
		}
	}

	/// Returns the configured log path.
	pub fn log_path(&self) -> &Path {
		&self.log_path
	}

	pub fn try_start_follow(&self) -> Result<(), AlfredError> {
		let mut state = self
			.follow_state
			.lock()
			.map_err(|_| AlfredError::Internal("follow state lock poisoned".to_string()))?;
		if state.active {
			return Err(AlfredError::ConflictWithDetails {
				message: "follow stream already active".to_string(),
				details: Some(json!({"reason": "stream_active"})),
			});
		}

		state.active = true;
		Ok(())
	}

	pub fn stop_follow(&self) -> Result<(), AlfredError> {
		let mut state = self
			.follow_state
			.lock()
			.map_err(|_| AlfredError::Internal("follow state lock poisoned".to_string()))?;
		if !state.active {
			return Err(AlfredError::ConflictWithDetails {
				message: "no follow stream active".to_string(),
				details: Some(json!({"reason": "stream_not_active"})),
			});
		}

		state.active = false;
		Ok(())
	}
}
