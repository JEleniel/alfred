//! Configuration models and defaults.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use sha2::{Digest, Sha256};

const WORKSPACE_STORAGE_DIR: &str = ".alfred";
const WORKSPACE_GITIGNORE_PATH: &str = ".gitignore";
const WORKSPACE_GITIGNORE_TEMPLATE: &str = "*\n!config.json\n";

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
	/// Workspace-relative root folder used for Alfred-owned runtime artifacts.
	pub workspace_storage_root_dir: String,
	/// Default user-level configuration path.
	pub user_config_path: PathBuf,
	/// Default user-level `.alfredignore` path.
	pub user_ignore_path: PathBuf,
	/// Default workspace-level configuration path.
	pub workspace_config_path: PathBuf,
	/// Number of days to keep rotated runtime log archives.
	pub runtime_log_retention_days: u64,
	/// Whether workspace indexing is enabled.
	pub index_enabled: bool,
	/// Where workspace indexes are persisted.
	pub index_persistence_location: IndexPersistenceLocation,
	/// Optional index persistence path override.
	pub index_persistence_path: Option<String>,
	/// Interval between persisted index commits.
	pub index_persist_interval_seconds: u64,
	/// Per-path debounce window used by file watching.
	pub index_watch_debounce_millis: u64,
	/// Effective disabled tool names (stable sorted, deduplicated).
	pub disabled_tools: Vec<String>,
}

impl AppConfig {
	/// Loads default configuration locations from the host environment.
	pub fn load_default() -> Result<Self> {
		let workspace_root = env::current_dir().context("failed to resolve current directory")?;
		let user_config_path = default_user_config_path(&workspace_root);
		let workspace_config_path = default_workspace_config_path(&workspace_root);

		Self::load_from_paths(workspace_root, user_config_path, workspace_config_path)
	}

	/// Loads configuration from explicit locations, then applies deterministic merge rules.
	pub fn load_from_paths(
		workspace_root: PathBuf,
		user_config_path: PathBuf,
		workspace_config_path: PathBuf,
	) -> Result<Self> {
		ensure_workspace_storage_root(workspace_root.as_path())?;
		let user_ignore_path = default_user_ignore_path(user_config_path.as_path());

		let user_config_root = read_config_json(user_config_path.as_path())?;
		let workspace_config_root = read_config_json(workspace_config_path.as_path())?;

		let disabled_tools = load_effective_disabled_tools(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;

		let workspace_storage_root_dir = resolve_workspace_storage_root_dir(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;
		ensure_workspace_storage_dir(
			workspace_root
				.join(workspace_storage_root_dir.as_str())
				.as_path(),
		)?;

		let runtime_log_retention_days = default_runtime_log_retention_days();
		let index_enabled = resolve_index_enabled(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;
		let index_persistence_location = resolve_index_persistence_location(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or_else(default_index_persistence_location);
		let index_persistence_path = resolve_index_persistence_path(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;
		let index_persist_interval_seconds = resolve_index_persist_interval_seconds(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or_else(default_index_persist_interval_seconds);
		let index_watch_debounce_millis = default_index_watch_debounce_millis();

		Ok(Self {
			workspace_root,
			workspace_storage_root_dir,
			user_config_path,
			user_ignore_path,
			workspace_config_path,
			runtime_log_retention_days,
			index_enabled,
			index_persistence_location,
			index_persistence_path,
			index_persist_interval_seconds,
			index_watch_debounce_millis,
			disabled_tools,
		})
	}

	/// Updates `workspace_root` and recomputes any derived workspace-scoped defaults.
	pub fn rebind_workspace_root(&mut self, workspace_root: PathBuf) {
		self.workspace_root = workspace_root.clone();
		self.workspace_config_path = default_workspace_config_path(&workspace_root);
	}

	/// Returns true when a tool is enabled by deterministic policy.
	pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
		!self.disabled_tools.iter().any(|name| name == tool_name)
	}

	/// Resolves the effective persisted-index root directory for this workspace.
	pub fn effective_workspace_index_root(&self) -> PathBuf {
		match self.index_persistence_location {
			IndexPersistenceLocation::Workspace => {
				if let Some(path) = self.index_persistence_path.as_deref() {
					return self.workspace_root.join(path);
				}

				self.effective_workspace_storage_root().join("index")
			}
			IndexPersistenceLocation::UserData => {
				let Some(data_dir) = dirs::data_dir() else {
					return default_workspace_index_root(&self.workspace_root);
				};
				if let Some(path) = self.index_persistence_path.as_deref() {
					return data_dir.join(path);
				}

				let workspace_key = workspace_index_key(&self.workspace_root);
				data_dir
					.join("alfred")
					.join("workspaces")
					.join(workspace_key)
					.join("index")
			}
		}
	}

	/// Resolves the effective workspace storage root directory.
	pub fn effective_workspace_storage_root(&self) -> PathBuf {
		self.workspace_root
			.join(self.workspace_storage_root_dir.as_str())
	}

	/// Resolves the effective runtime log path.
	pub fn effective_runtime_log_path(&self) -> PathBuf {
		self.effective_workspace_storage_root()
			.join("logs")
			.join("runtime.ndjson")
	}
}

const DEFAULT_RUNTIME_LOG_RETENTION_DAYS: u64 = 7;
const DEFAULT_INDEX_PERSIST_INTERVAL_SECONDS: u64 = 5;
const DEFAULT_INDEX_WATCH_DEBOUNCE_MILLIS: u64 = 250;
const DEFAULT_DISABLED_MUTATING_TOOLS: &[&str] = &[
	"dir_create",
	"dir_delete",
	"env_set",
	"env_unset",
	"file_append",
	"file_append_bytes",
	"file_create",
	"file_create_bytes",
	"file_delete",
	"file_patch",
	"memory_delete",
	"memory_put",
	"multi_file_patch",
	"path_copy",
	"path_delete",
	"path_move",
	"plan_add",
	"plan_delete",
	"plan_edit",
	"plan_update",
	"task_run",
];

fn default_user_config_path(workspace_root: &Path) -> PathBuf {
	if let Some(config_dir) = dirs::config_dir() {
		return config_dir.join("alfred").join("config.json");
	}

	// Extremely constrained environments may not expose user config dirs; fall back to a
	// workspace-controlled location without colliding with workspace config.
	workspace_root
		.join(WORKSPACE_STORAGE_DIR)
		.join("config.user.json")
}

fn default_workspace_config_path(workspace_root: &Path) -> PathBuf {
	workspace_root
		.join(WORKSPACE_STORAGE_DIR)
		.join("config.json")
}

fn default_user_ignore_path(user_config_path: &Path) -> PathBuf {
	user_config_path
		.parent()
		.map(|parent| parent.join(".alfredignore"))
		.unwrap_or_else(|| PathBuf::from(".alfredignore"))
}

fn load_effective_disabled_tools(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Vec<String>> {
	let mut names = DEFAULT_DISABLED_MUTATING_TOOLS
		.iter()
		.map(|name| (*name).to_string())
		.collect::<Vec<_>>();
	names.extend(extract_disabled_tools(user_root, user_path)?);
	names.extend(extract_disabled_tools(workspace_root, workspace_path)?);
	names.sort_unstable();
	names.dedup();
	Ok(names)
}

fn read_config_json(config_path: &Path) -> Result<Option<Value>> {
	let raw = match fs::read_to_string(config_path) {
		Ok(contents) => contents,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
		Err(error) => {
			return Err(error)
				.with_context(|| format!("failed to read config file {}", config_path.display()));
		}
	};

	serde_json::from_str::<Value>(raw.as_str())
		.map(Some)
		.with_context(|| format!("failed to parse config file {}", config_path.display()))
}

fn extract_disabled_tools(config_root: Option<&Value>, config_path: &Path) -> Result<Vec<String>> {
	let Some(config_root) = config_root else {
		return Ok(Vec::new());
	};

	let Some(tools) = config_root.get("tools") else {
		return Ok(Vec::new());
	};
	let Some(tools_object) = tools.as_object() else {
		return Err(anyhow!(
			"config field tools must be an object in {}",
			config_path.display()
		));
	};

	let Some(disabled) = tools_object.get("disabled") else {
		return Ok(Vec::new());
	};
	let Some(disabled_array) = disabled.as_array() else {
		return Err(anyhow!(
			"config field tools.disabled must be an array in {}",
			config_path.display()
		));
	};

	disabled_array
		.iter()
		.map(|item| {
			item.as_str().map(|name| name.to_string()).ok_or_else(|| {
				anyhow!(
					"config field tools.disabled must contain strings in {}",
					config_path.display()
				)
			})
		})
		.collect()
}

fn resolve_workspace_storage_root_dir(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<String> {
	if let Some(dir) = extract_workspace_storage_root_dir(workspace_root, workspace_path)? {
		return Ok(dir);
	}
	if let Some(dir) = extract_workspace_storage_root_dir(user_root, user_path)? {
		return Ok(dir);
	}

	Ok(WORKSPACE_STORAGE_DIR.to_string())
}

fn extract_workspace_storage_root_dir(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<String>> {
	let Some(config_root) = config_root else {
		return Ok(None);
	};
	let Some(root) = config_root
		.get("workspace")
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("storage"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("root"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	let normalized = normalize_workspace_relative_dir(root, config_path)?;
	Ok(Some(normalized))
}

fn resolve_index_enabled(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<bool> {
	if let Some(enabled) = extract_index_enabled(workspace_root, workspace_path)? {
		return Ok(enabled);
	}
	if let Some(enabled) = extract_index_enabled(user_root, user_path)? {
		return Ok(enabled);
	}

	Ok(true)
}

fn extract_index_enabled(config_root: Option<&Value>, _config_path: &Path) -> Result<Option<bool>> {
	let Some(config_root) = config_root else {
		return Ok(None);
	};

	Ok(config_root
		.get("index")
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("enabled"))
		.and_then(Value::as_bool))
}

fn resolve_index_persistence_location(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<IndexPersistenceLocation>> {
	if let Some(location) = extract_index_persistence_location(workspace_root, workspace_path)? {
		return Ok(Some(location));
	}
	if let Some(location) = extract_index_persistence_location(user_root, user_path)? {
		return Ok(Some(location));
	}

	Ok(None)
}

fn extract_index_persistence_location(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<IndexPersistenceLocation>> {
	let Some(value) = config_root
		.and_then(|root| root.get("index"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("persistence"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("location"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	if value.eq_ignore_ascii_case("workspace") {
		return Ok(Some(IndexPersistenceLocation::Workspace));
	}
	if value.eq_ignore_ascii_case("user") {
		return Ok(Some(IndexPersistenceLocation::UserData));
	}

	Err(anyhow!(
		"config field index.persistence.location must be 'workspace' or 'user' in {}",
		config_path.display()
	))
}

fn resolve_index_persistence_path(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<String>> {
	if let Some(path) = extract_index_persistence_path(workspace_root, workspace_path)? {
		return Ok(Some(path));
	}
	if let Some(path) = extract_index_persistence_path(user_root, user_path)? {
		return Ok(Some(path));
	}

	Ok(None)
}

fn extract_index_persistence_path(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<String>> {
	let Some(value) = config_root
		.and_then(|root| root.get("index"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("persistence"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("path"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	Ok(Some(normalize_workspace_relative_dir(value, config_path)?))
}

fn resolve_index_persist_interval_seconds(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<u64>> {
	if let Some(value) = extract_index_persist_interval_seconds(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_index_persist_interval_seconds(user_root, user_path)? {
		return Ok(Some(value));
	}

	Ok(None)
}

fn extract_index_persist_interval_seconds(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<u64>> {
	let Some(value) = config_root
		.and_then(|root| root.get("index"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("persistence"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("interval_seconds"))
	else {
		return Ok(None);
	};

	let Some(value) = value.as_u64() else {
		return Err(anyhow!(
			"config field index.persistence.interval_seconds must be an integer in {}",
			config_path.display()
		));
	};
	if value == 0 {
		return Err(anyhow!(
			"config field index.persistence.interval_seconds must be greater than 0 in {}",
			config_path.display()
		));
	}
	Ok(Some(value))
}

fn normalize_workspace_relative_dir(value: &str, config_path: &Path) -> Result<String> {
	let trimmed = value.trim().replace('\\', "/");
	let trimmed = trimmed.trim_end_matches('/');
	if trimmed.is_empty() {
		return Err(anyhow!(
			"workspace storage root must not be empty in {}",
			config_path.display()
		));
	}
	if !is_workspace_relative_path(trimmed) {
		return Err(anyhow!(
			"workspace storage root must be workspace-relative in {}",
			config_path.display()
		));
	}
	Ok(trimmed.to_string())
}

fn default_runtime_log_retention_days() -> u64 {
	env::var("ALFRED_RUNTIME_LOG_RETENTION_DAYS")
		.ok()
		.and_then(|value| value.trim().parse::<u64>().ok())
		.unwrap_or(DEFAULT_RUNTIME_LOG_RETENTION_DAYS)
}

fn default_index_persistence_location() -> IndexPersistenceLocation {
	match env::var("ALFRED_INDEX_PERSISTENCE_LOCATION") {
		Ok(value) if value.eq_ignore_ascii_case("user") => IndexPersistenceLocation::UserData,
		_ => IndexPersistenceLocation::Workspace,
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
	workspace_root
		.join(WORKSPACE_STORAGE_DIR)
		.join("logs")
		.join("runtime.ndjson")
}

/// Builds the default per-workspace index persistence root in user data storage.
pub fn default_workspace_index_root(workspace_root: &Path) -> PathBuf {
	// For isolation and user control, index persistence is workspace-scoped.
	// (The workspace key is retained for stable directory naming even if the root is per-workspace.)
	let workspace_key = workspace_index_key(workspace_root);
	workspace_root
		.join(WORKSPACE_STORAGE_DIR)
		.join("index")
		.join(workspace_key)
}

/// Builds the default workspace-local index persistence root.
pub fn default_workspace_local_index_root(workspace_root: &Path) -> PathBuf {
	workspace_root.join(WORKSPACE_STORAGE_DIR).join("index")
}

fn workspace_index_key(workspace_root: &Path) -> String {
	let normalized = workspace_root.to_string_lossy().replace('\\', "/");
	let digest = Sha256::digest(normalized.as_bytes());
	hex::encode(digest)
}

/// Ensures `.alfred/` exists under the workspace root, and that it contains the
/// default `.gitignore` used to keep runtime state out of source control.
///
/// This is idempotent and safe to call multiple times.
pub fn ensure_workspace_storage_root(workspace_root: &Path) -> Result<()> {
	ensure_workspace_storage_dir(workspace_root.join(WORKSPACE_STORAGE_DIR).as_path())
}

/// Ensures a workspace-relative storage directory exists and is gitignored by default.
pub fn ensure_workspace_storage_dir(storage_root: &Path) -> Result<()> {
	fs::create_dir_all(storage_root).with_context(|| {
		format!(
			"failed to create Alfred workspace storage root at {}",
			storage_root.display()
		)
	})?;

	let gitignore_path = storage_root.join(WORKSPACE_GITIGNORE_PATH);
	if !gitignore_path.exists() {
		fs::write(&gitignore_path, WORKSPACE_GITIGNORE_TEMPLATE).with_context(|| {
			format!(
				"failed to write Alfred workspace .gitignore at {}",
				gitignore_path.display()
			)
		})?;
	}

	Ok(())
}
