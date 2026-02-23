//! Configuration models and defaults.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// Storage location used for persisted workspace indexes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IndexPersistenceLocation {
	UserData,
	Workspace,
}

/// Effective Alfred configuration values.
#[derive(Debug, Clone)]
pub struct AppConfig {
	/// Effective workspace root.
	pub workspace_root: PathBuf,
	/// Default user-level configuration path.
	pub user_config_path: PathBuf,
	/// Default workspace-level configuration path.
	pub workspace_config_path: PathBuf,
	/// Number of days to keep rotated runtime log archives.
	pub runtime_log_retention_days: u64,
	/// Where workspace indexes are persisted.
	pub index_persistence_location: IndexPersistenceLocation,
	/// Interval between persisted index commits.
	pub index_persist_interval_seconds: u64,
	/// Per-path debounce window used by file watching.
	pub index_watch_debounce_millis: u64,
}

impl AppConfig {
	/// Loads default configuration locations from the host environment.
	pub fn load_default() -> Result<Self> {
		let workspace_root = env::current_dir().context("failed to resolve current directory")?;

		let user_config_path = dirs::config_dir()
			.unwrap_or_else(|| workspace_root.join(".agents"))
			.join("alfred")
			.join("config.json");

		let workspace_config_path = workspace_root
			.join(".agents")
			.join("alfred")
			.join("config.json");

		let runtime_log_retention_days = default_runtime_log_retention_days();
		let index_persistence_location = default_index_persistence_location();
		let index_persist_interval_seconds = default_index_persist_interval_seconds();
		let index_watch_debounce_millis = default_index_watch_debounce_millis();

		Ok(Self {
			workspace_root,
			user_config_path,
			workspace_config_path,
			runtime_log_retention_days,
			index_persistence_location,
			index_persist_interval_seconds,
			index_watch_debounce_millis,
		})
	}

	/// Resolves the effective persisted-index root directory for this workspace.
	pub fn effective_workspace_index_root(&self) -> PathBuf {
		match self.index_persistence_location {
			IndexPersistenceLocation::UserData => {
				default_workspace_index_root(&self.workspace_root)
			}
			IndexPersistenceLocation::Workspace => {
				default_workspace_local_index_root(&self.workspace_root)
			}
		}
	}
}

const DEFAULT_RUNTIME_LOG_RETENTION_DAYS: u64 = 7;
const DEFAULT_INDEX_PERSIST_INTERVAL_SECONDS: u64 = 5;
const DEFAULT_INDEX_WATCH_DEBOUNCE_MILLIS: u64 = 250;

fn default_runtime_log_retention_days() -> u64 {
	env::var("ALFRED_RUNTIME_LOG_RETENTION_DAYS")
		.ok()
		.and_then(|value| value.trim().parse::<u64>().ok())
		.unwrap_or(DEFAULT_RUNTIME_LOG_RETENTION_DAYS)
}

fn default_index_persistence_location() -> IndexPersistenceLocation {
	match env::var("ALFRED_INDEX_PERSISTENCE_LOCATION") {
		Ok(value) if value.eq_ignore_ascii_case("workspace") => IndexPersistenceLocation::Workspace,
		_ => IndexPersistenceLocation::UserData,
	}
}

fn default_index_persist_interval_seconds() -> u64 {
	env::var("ALFRED_INDEX_PERSIST_INTERVAL_SECONDS")
		.ok()
		.and_then(|value| value.trim().parse::<u64>().ok())
		.filter(|value| *value > 0)
		.unwrap_or(DEFAULT_INDEX_PERSIST_INTERVAL_SECONDS)
}

fn default_index_watch_debounce_millis() -> u64 {
	env::var("ALFRED_INDEX_WATCH_DEBOUNCE_MILLIS")
		.ok()
		.and_then(|value| value.trim().parse::<u64>().ok())
		.filter(|value| *value > 0)
		.unwrap_or(DEFAULT_INDEX_WATCH_DEBOUNCE_MILLIS)
}

/// Validates that a path is workspace-relative and safe for tool contracts.
pub fn is_workspace_relative_path(path: &str) -> bool {
	if path.is_empty() {
		return false;
	}

	let normalized = path.replace('\\', "/");
	if normalized.starts_with('/') {
		return false;
	}

	if normalized.len() >= 3 {
		let bytes = normalized.as_bytes();
		if bytes[1] == b':' && bytes[2] == b'/' {
			return false;
		}
	}

	!normalized.split('/').any(|segment| segment == "..")
}

/// Builds the default project plan path according to repository conventions.
pub fn default_plan_path(workspace_root: &Path) -> PathBuf {
	let preferred = workspace_root
		.join("docs")
		.join("design")
		.join("ProjectPlan.md");
	if preferred.exists() {
		return preferred;
	}

	workspace_root.join("ProjectPlan.md")
}

/// Builds the default runtime log path under user data storage.
pub fn default_runtime_log_path(workspace_root: &Path) -> PathBuf {
	let user_data_root = dirs::data_dir()
		.unwrap_or_else(|| workspace_root.join(".agents"))
		.join("alfred");

	user_data_root.join("logs").join("runtime.ndjson")
}

/// Builds the default per-workspace index persistence root in user data storage.
pub fn default_workspace_index_root(workspace_root: &Path) -> PathBuf {
	let user_data_root = dirs::data_dir()
		.unwrap_or_else(|| workspace_root.join(".agents"))
		.join("alfred");

	let workspace_key = workspace_index_key(workspace_root);
	user_data_root.join("index").join(workspace_key)
}

/// Builds the default workspace-local index persistence root.
pub fn default_workspace_local_index_root(workspace_root: &Path) -> PathBuf {
	workspace_root.join(".agents").join("alfred").join("index")
}

fn workspace_index_key(workspace_root: &Path) -> String {
	let normalized = workspace_root.to_string_lossy().replace('\\', "/");
	let digest = Sha256::digest(normalized.as_bytes());
	hex::encode(digest)
}
