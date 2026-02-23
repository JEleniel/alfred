//! Configuration models and defaults.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

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

		Ok(Self {
			workspace_root,
			user_config_path,
			workspace_config_path,
			runtime_log_retention_days,
		})
	}
}

const DEFAULT_RUNTIME_LOG_RETENTION_DAYS: u64 = 7;

fn default_runtime_log_retention_days() -> u64 {
	env::var("ALFRED_RUNTIME_LOG_RETENTION_DAYS")
		.ok()
		.and_then(|value| value.trim().parse::<u64>().ok())
		.unwrap_or(DEFAULT_RUNTIME_LOG_RETENTION_DAYS)
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
