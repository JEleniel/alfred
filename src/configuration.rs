//! Configuration models and defaults.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::redaction::{DEFAULT_REPLACEMENT_TOKEN, RedactionRule, default_redaction_rules};

const WORKSPACE_STORAGE_DIR: &str = ".alfred";
const WORKSPACE_GITIGNORE_PATH: &str = ".gitignore";
const WORKSPACE_GITIGNORE_TEMPLATE: &str = "*\n!config.json\n";

/// Storage location used for persisted workspace indexes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IndexPersistenceLocation {
	UserData,
	Workspace,
}

/// Storage location used for user-scoped configuration and data.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StorageUserLocation {
	/// Store user-scoped artifacts under OS user directories.
	Os,
	/// Store user-scoped artifacts under the workspace storage root.
	Workspace,
}

/// Storage location used for workspace-scoped configuration and data.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StorageWorkspaceLocation {
	/// Store workspace-scoped artifacts under the workspace storage root.
	Workspace,
	/// Store workspace-scoped artifacts under OS user directories keyed by workspace id.
	User,
}

/// Storage location selection for runtime logs.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RuntimeLogLocation {
	Auto,
	Workspace,
	UserLogs,
	UserData,
	SystemLogs,
}

/// Host-provided default directories used for config/data discovery.
#[derive(Debug, Clone, Default)]
pub struct HostPaths {
	pub user_config_dir: Option<PathBuf>,
	pub user_data_dir: Option<PathBuf>,
	pub user_logs_dir: Option<PathBuf>,
	pub system_logs_dir: Option<PathBuf>,
}

impl HostPaths {
	pub fn from_os() -> Self {
		Self {
			user_config_dir: dirs::config_dir(),
			user_data_dir: dirs::data_dir(),
			user_logs_dir: default_os_user_logs_dir(),
			system_logs_dir: default_os_system_logs_dir(),
		}
	}
}

/// Effective Alfred configuration values.
#[derive(Debug, Clone)]
pub struct AppConfig {
	/// Host-provided OS user config directory (when available).
	pub host_user_config_dir: Option<PathBuf>,
	/// Host-provided OS user data directory (when available).
	pub host_user_data_dir: Option<PathBuf>,
	/// Host-provided OS user logs directory (when available).
	pub host_user_logs_dir: Option<PathBuf>,
	/// Host-provided OS system logs directory (when available).
	pub host_system_logs_dir: Option<PathBuf>,
	/// Effective workspace root.
	pub workspace_root: PathBuf,
	/// Stable identifier derived from `workspace_root`.
	pub workspace_id: String,
	/// Workspace-relative root folder used for Alfred-owned runtime artifacts.
	pub workspace_storage_root_dir: String,
	/// Effective user-scoped storage location selection.
	pub storage_user_location: StorageUserLocation,
	/// Effective workspace-scoped storage location selection.
	pub storage_workspace_location: StorageWorkspaceLocation,
	/// Default user-level configuration path.
	pub user_config_path: PathBuf,
	/// Default user-level `.alfredignore` path.
	pub user_ignore_path: PathBuf,
	/// Default workspace-level configuration path.
	pub workspace_config_path: PathBuf,
	/// Optional relocated workspace `.alfredignore` path when workspace config is relocated.
	pub workspace_ignore_path: Option<PathBuf>,
	/// Selected runtime log location.
	pub runtime_log_location: RuntimeLogLocation,
	/// Optional runtime log directory override.
	pub runtime_log_path: Option<String>,
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
	/// Whether deterministic redaction is enabled.
	pub redaction_enabled: bool,
	/// Replacement token used for redaction.
	pub redaction_replacement_token: String,
	/// Whether redaction should preserve the scalar-length of redacted spans.
	pub redaction_preserve_length: bool,
	/// Effective, ordered redaction rule list.
	pub redaction_rules: Vec<RedactionRule>,
	/// Whether user-scoped memory persistence is enabled.
	pub memory_user_enabled: bool,
	/// Whether workspace-scoped memory persistence is enabled.
	pub memory_workspace_enabled: bool,
	/// Optional memory store path override for user scope (relative to the selected user root).
	pub memory_user_path: Option<String>,
	/// Optional memory store path override for workspace scope (relative to the selected workspace root).
	pub memory_workspace_path: Option<String>,
}

impl AppConfig {
	/// Loads default configuration locations from the host environment.
	pub fn load_default() -> Result<Self> {
		let workspace_root = env::current_dir().context("failed to resolve current directory")?;
		Self::load_with_host_paths(workspace_root, HostPaths::from_os())
	}

	/// Loads configuration using explicit host directory defaults (for deterministic tests).
	pub fn load_with_host_paths(workspace_root: PathBuf, host: HostPaths) -> Result<Self> {
		let os_user_config_path =
			default_os_user_config_path(&workspace_root, host.user_config_dir.as_deref());
		let os_user_root = read_config_json(os_user_config_path.as_path())?;

		let seed_workspace_storage_root_dir = extract_workspace_storage_root_dir(
			os_user_root.as_ref(),
			os_user_config_path.as_path(),
		)?
		.unwrap_or_else(|| WORKSPACE_STORAGE_DIR.to_string());
		let seed_user_location = resolve_storage_user_location(
			os_user_root.as_ref(),
			None,
			os_user_config_path.as_path(),
			os_user_config_path.as_path(),
		)?
		.unwrap_or(StorageUserLocation::Os);
		let seed_workspace_location = resolve_storage_workspace_location(
			os_user_root.as_ref(),
			None,
			os_user_config_path.as_path(),
			os_user_config_path.as_path(),
		)?
		.unwrap_or(StorageWorkspaceLocation::Workspace);
		let defaults = ConfigDefaults {
			workspace_storage_root_dir: seed_workspace_storage_root_dir.clone(),
			storage_user_location: seed_user_location,
			storage_workspace_location: seed_workspace_location,
		};

		let workspace_id = workspace_id(&workspace_root);
		let user_config_path = match seed_user_location {
			StorageUserLocation::Os => os_user_config_path.clone(),
			StorageUserLocation::Workspace => workspace_root
				.join(seed_workspace_storage_root_dir.as_str())
				.join("user")
				.join("config.json"),
		};
		let workspace_config_path = match seed_workspace_location {
			StorageWorkspaceLocation::Workspace => workspace_root
				.join(seed_workspace_storage_root_dir.as_str())
				.join("config.json"),
			StorageWorkspaceLocation::User => default_relocated_workspace_config_path(
				host.user_config_dir.as_deref(),
				&workspace_root,
				workspace_id.as_str(),
			),
		};

		Self::load_from_paths_with_host(
			workspace_root,
			host,
			user_config_path,
			workspace_config_path,
			defaults,
		)
	}

	/// Loads configuration from explicit locations, then applies deterministic merge rules.
	pub fn load_from_paths(
		workspace_root: PathBuf,
		user_config_path: PathBuf,
		workspace_config_path: PathBuf,
	) -> Result<Self> {
		Self::load_from_paths_with_host(
			workspace_root,
			HostPaths::from_os(),
			user_config_path,
			workspace_config_path,
			ConfigDefaults::standard(),
		)
	}

	fn load_from_paths_with_host(
		workspace_root: PathBuf,
		host: HostPaths,
		user_config_path: PathBuf,
		workspace_config_path: PathBuf,
		defaults: ConfigDefaults,
	) -> Result<Self> {
		let user_ignore_path = default_user_ignore_path(user_config_path.as_path());
		let user_config_root = read_config_json(user_config_path.as_path())?;
		let workspace_config_root = read_config_json(workspace_config_path.as_path())?;

		let disabled_tools = load_effective_disabled_tools(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;

		let workspace_storage_root_dir = if let Some(dir) = extract_workspace_storage_root_dir(
			workspace_config_root.as_ref(),
			workspace_config_path.as_path(),
		)? {
			dir
		} else if let Some(dir) = extract_workspace_storage_root_dir(
			user_config_root.as_ref(),
			user_config_path.as_path(),
		)? {
			dir
		} else {
			defaults.workspace_storage_root_dir.clone()
		};
		let storage_user_location = resolve_storage_user_location(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or(defaults.storage_user_location);
		let storage_workspace_location = resolve_storage_workspace_location(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or(defaults.storage_workspace_location);
		let workspace_id = workspace_id(&workspace_root);

		if needs_workspace_storage_scaffold(storage_user_location, storage_workspace_location) {
			ensure_workspace_storage_dir(
				workspace_root
					.join(workspace_storage_root_dir.as_str())
					.as_path(),
			)?;
		}

		let workspace_ignore_path = if storage_workspace_location == StorageWorkspaceLocation::User
		{
			workspace_config_path
				.parent()
				.map(|parent| parent.join(".alfredignore"))
		} else {
			None
		};

		let runtime_log_location = resolve_runtime_log_location(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or(RuntimeLogLocation::Auto);
		let runtime_log_path = resolve_runtime_log_path(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;
		let runtime_log_retention_days = resolve_runtime_log_retention_days(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or_else(default_runtime_log_retention_days);
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

		let redaction_enabled = resolve_redaction_enabled(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;
		let redaction_replacement_token = resolve_redaction_replacement_token(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or_else(|| DEFAULT_REPLACEMENT_TOKEN.to_string());
		let redaction_preserve_length = resolve_redaction_preserve_length(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or(true);
		let redaction_rules = resolve_redaction_rules(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or_else(default_redaction_rules);

		let memory_user_enabled = resolve_memory_user_enabled(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or(true);
		let memory_workspace_enabled = resolve_memory_workspace_enabled(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?
		.unwrap_or(false);
		let memory_user_path = resolve_memory_user_path(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;
		let memory_workspace_path = resolve_memory_workspace_path(
			user_config_root.as_ref(),
			workspace_config_root.as_ref(),
			user_config_path.as_path(),
			workspace_config_path.as_path(),
		)?;

		Ok(Self {
			host_user_config_dir: host.user_config_dir,
			host_user_data_dir: host.user_data_dir,
			host_user_logs_dir: host.user_logs_dir,
			host_system_logs_dir: host.system_logs_dir,
			workspace_root,
			workspace_id,
			workspace_storage_root_dir,
			storage_user_location,
			storage_workspace_location,
			user_config_path,
			user_ignore_path,
			workspace_config_path,
			workspace_ignore_path,
			runtime_log_location,
			runtime_log_path,
			runtime_log_retention_days,
			index_enabled,
			index_persistence_location,
			index_persistence_path,
			index_persist_interval_seconds,
			index_watch_debounce_millis,
			disabled_tools,
			redaction_enabled,
			redaction_replacement_token,
			redaction_preserve_length,
			redaction_rules,
			memory_user_enabled,
			memory_workspace_enabled,
			memory_user_path,
			memory_workspace_path,
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
		let host_data_dir = self.host_user_data_dir.clone().or_else(dirs::data_dir);
		match self.index_persistence_location {
			IndexPersistenceLocation::Workspace => {
				if let Some(path) = self.index_persistence_path.as_deref() {
					return self.workspace_root.join(path);
				}

				self.effective_workspace_storage_root().join("index")
			}
			IndexPersistenceLocation::UserData => {
				let Some(data_dir) = host_data_dir else {
					return default_workspace_index_root(&self.workspace_root);
				};
				if let Some(path) = self.index_persistence_path.as_deref() {
					return data_dir.join(path);
				}

				data_dir
					.join("alfred")
					.join(self.workspace_id.as_str())
					.join("data")
					.join("index")
			}
		}
	}

	/// Resolves the effective workspace storage root directory.
	pub fn effective_workspace_storage_root(&self) -> PathBuf {
		match self.storage_workspace_location {
			StorageWorkspaceLocation::Workspace => self
				.workspace_root
				.join(self.workspace_storage_root_dir.as_str()),
			StorageWorkspaceLocation::User => {
				let Some(data_dir) = self.host_user_data_dir.clone().or_else(dirs::data_dir) else {
					return self
						.workspace_root
						.join(self.workspace_storage_root_dir.as_str());
				};
				data_dir
					.join("alfred")
					.join(self.workspace_id.as_str())
					.join("data")
			}
		}
	}

	/// Resolves the workspace storage root directory path inside the workspace.
	///
	/// This is used for the workspace-local user profile layout.
	pub fn workspace_storage_root_in_workspace(&self) -> PathBuf {
		self.workspace_root
			.join(self.workspace_storage_root_dir.as_str())
	}

	/// Resolves the effective runtime log path.
	pub fn effective_runtime_log_path(&self) -> PathBuf {
		let override_dir = self
			.runtime_log_path
			.as_deref()
			.map(|raw| resolve_runtime_log_override_dir(&self.workspace_root, raw));
		let workspace_dir = self.workspace_storage_root_in_workspace().join("logs");
		let user_logs_dir = self
			.host_user_logs_dir
			.clone()
			.or_else(default_os_user_logs_dir);
		let user_data_dir = self
			.host_user_data_dir
			.clone()
			.or_else(dirs::data_dir)
			.map(|root| root.join("alfred").join("logs"));
		let system_logs_dir = self
			.host_system_logs_dir
			.clone()
			.or_else(default_os_system_logs_dir);

		let candidates: Vec<PathBuf> = if let Some(dir) = override_dir {
			vec![dir]
		} else {
			match self.runtime_log_location {
				RuntimeLogLocation::Workspace => vec![workspace_dir.clone()],
				RuntimeLogLocation::UserLogs => vec![
					user_logs_dir
						.clone()
						.unwrap_or_else(|| workspace_dir.clone()),
					workspace_dir.clone(),
				],
				RuntimeLogLocation::UserData => vec![
					user_data_dir
						.clone()
						.unwrap_or_else(|| workspace_dir.clone()),
					workspace_dir.clone(),
				],
				RuntimeLogLocation::SystemLogs => vec![
					system_logs_dir
						.clone()
						.unwrap_or_else(|| workspace_dir.clone()),
					workspace_dir.clone(),
				],
				RuntimeLogLocation::Auto => {
					let mut dirs = Vec::new();
					if let Some(dir) = user_logs_dir {
						dirs.push(dir);
					}
					if let Some(dir) = user_data_dir {
						dirs.push(dir);
					}
					if let Some(dir) = system_logs_dir {
						dirs.push(dir);
					}
					dirs.push(workspace_dir.clone());
					dirs
				}
			}
		};

		candidates
			.first()
			.cloned()
			.unwrap_or(workspace_dir)
			.join("runtime.ndjson")
	}

	/// Returns candidate runtime log directories in preferred order.
	pub fn runtime_log_dir_candidates(&self) -> Vec<PathBuf> {
		let override_dir = self
			.runtime_log_path
			.as_deref()
			.map(|raw| resolve_runtime_log_override_dir(&self.workspace_root, raw));
		if let Some(dir) = override_dir {
			return vec![dir];
		}

		let workspace_dir = self.workspace_storage_root_in_workspace().join("logs");
		let user_logs_dir = self
			.host_user_logs_dir
			.clone()
			.or_else(default_os_user_logs_dir);
		let user_data_dir = self
			.host_user_data_dir
			.clone()
			.or_else(dirs::data_dir)
			.map(|root| root.join("alfred").join("logs"));
		let system_logs_dir = self
			.host_system_logs_dir
			.clone()
			.or_else(default_os_system_logs_dir);

		match self.runtime_log_location {
			RuntimeLogLocation::Workspace => vec![workspace_dir],
			RuntimeLogLocation::UserLogs => vec![
				user_logs_dir.unwrap_or_else(|| workspace_dir.clone()),
				workspace_dir,
			],
			RuntimeLogLocation::UserData => vec![
				user_data_dir.unwrap_or_else(|| workspace_dir.clone()),
				workspace_dir,
			],
			RuntimeLogLocation::SystemLogs => vec![
				system_logs_dir.unwrap_or_else(|| workspace_dir.clone()),
				workspace_dir,
			],
			RuntimeLogLocation::Auto => {
				let mut dirs = Vec::new();
				if let Some(dir) = user_logs_dir {
					dirs.push(dir);
				}
				if let Some(dir) = user_data_dir {
					dirs.push(dir);
				}
				if let Some(dir) = system_logs_dir {
					dirs.push(dir);
				}
				dirs.push(workspace_dir);
				dirs
			}
		}
	}
	/// Resolves the effective user-scoped memory root.
	pub fn effective_user_memory_root(&self) -> PathBuf {
		match self.storage_user_location {
			StorageUserLocation::Workspace => self
				.workspace_storage_root_in_workspace()
				.join("user")
				.join("memory"),
			StorageUserLocation::Os => {
				let Some(data_dir) = self.host_user_data_dir.clone().or_else(dirs::data_dir) else {
					return self
						.workspace_storage_root_in_workspace()
						.join("user")
						.join("memory");
				};
				data_dir.join("alfred").join("memory")
			}
		}
	}

	/// Resolves the effective workspace-scoped memory root.
	pub fn effective_workspace_memory_root(&self) -> PathBuf {
		self.effective_workspace_storage_root().join("memory")
	}

	/// Resolves the effective configured user memory store path.
	pub fn effective_user_memory_store_path(&self) -> Option<PathBuf> {
		if !self.memory_user_enabled {
			return None;
		}
		let base = self.effective_user_memory_root();
		let root = self
			.memory_user_path
			.as_deref()
			.map(|value| base.join(value))
			.unwrap_or(base);
		Some(root.join("alfred"))
	}

	/// Resolves the effective configured workspace memory store path.
	pub fn effective_workspace_memory_store_path(&self) -> Option<PathBuf> {
		if !self.memory_workspace_enabled {
			return None;
		}
		let base = self.effective_workspace_memory_root();
		let root = self
			.memory_workspace_path
			.as_deref()
			.map(|value| base.join(value))
			.unwrap_or(base);
		Some(root.join("alfred"))
	}
}

fn resolve_runtime_log_override_dir(workspace_root: &Path, raw: &str) -> PathBuf {
	let trimmed = raw.trim();
	if trimmed.is_empty() {
		return workspace_root.join(WORKSPACE_STORAGE_DIR).join("logs");
	}
	let normalized = trimmed.replace('\\', "/");
	let candidate = PathBuf::from(&normalized);
	if candidate.is_absolute() {
		candidate
	} else {
		workspace_root.join(candidate)
	}
}

fn default_os_user_logs_dir() -> Option<PathBuf> {
	let home = dirs::home_dir()?;
	if cfg!(target_os = "macos") {
		return Some(home.join("Library").join("Logs").join("alfred"));
	}
	if cfg!(target_os = "windows") {
		return None;
	}
	Some(
		home.join(".local")
			.join("state")
			.join("alfred")
			.join("logs"),
	)
}

fn default_os_system_logs_dir() -> Option<PathBuf> {
	if cfg!(target_os = "windows") {
		return None;
	}
	Some(PathBuf::from("/var/log/alfred"))
}

fn needs_workspace_storage_scaffold(
	user_location: StorageUserLocation,
	workspace_location: StorageWorkspaceLocation,
) -> bool {
	workspace_location == StorageWorkspaceLocation::Workspace
		|| user_location == StorageUserLocation::Workspace
}

#[derive(Debug, Clone)]
struct ConfigDefaults {
	workspace_storage_root_dir: String,
	storage_user_location: StorageUserLocation,
	storage_workspace_location: StorageWorkspaceLocation,
}

impl ConfigDefaults {
	fn standard() -> Self {
		Self {
			workspace_storage_root_dir: WORKSPACE_STORAGE_DIR.to_string(),
			storage_user_location: StorageUserLocation::Os,
			storage_workspace_location: StorageWorkspaceLocation::Workspace,
		}
	}
}

fn default_os_user_config_path(workspace_root: &Path, user_config_dir: Option<&Path>) -> PathBuf {
	if let Some(config_dir) = user_config_dir {
		return config_dir.join("alfred").join("config.json");
	}

	// Extremely constrained environments may not expose user config dirs; fall back to a
	// workspace-controlled location without colliding with workspace config.
	workspace_root
		.join(WORKSPACE_STORAGE_DIR)
		.join("config.user.json")
}

fn default_relocated_workspace_config_path(
	user_config_dir: Option<&Path>,
	workspace_root: &Path,
	workspace_id: &str,
) -> PathBuf {
	if let Some(config_dir) = user_config_dir {
		return config_dir
			.join("alfred")
			.join(workspace_id)
			.join("config.json");
	}

	workspace_root
		.join(WORKSPACE_STORAGE_DIR)
		.join("config.workspace.user.json")
}

fn resolve_redaction_enabled(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<bool> {
	if let Some(enabled) = extract_redaction_enabled(workspace_root, workspace_path)? {
		return Ok(enabled);
	}
	if let Some(enabled) = extract_redaction_enabled(user_root, user_path)? {
		return Ok(enabled);
	}

	Ok(true)
}

fn extract_redaction_enabled(
	config_root: Option<&Value>,
	_config_path: &Path,
) -> Result<Option<bool>> {
	Ok(config_root
		.and_then(|root| root.get("redaction"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("enabled"))
		.and_then(Value::as_bool))
}

fn resolve_redaction_replacement_token(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<String>> {
	if let Some(token) = extract_redaction_replacement_token(workspace_root, workspace_path)? {
		return Ok(Some(token));
	}
	if let Some(token) = extract_redaction_replacement_token(user_root, user_path)? {
		return Ok(Some(token));
	}

	Ok(None)
}

fn extract_redaction_replacement_token(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<String>> {
	let Some(value) = config_root
		.and_then(|root| root.get("redaction"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("replacement_token"))
	else {
		return Ok(None);
	};

	let Some(token) = value.as_str() else {
		return Err(anyhow!(
			"config field redaction.replacement_token must be a string in {}",
			config_path.display()
		));
	};

	Ok(Some(token.to_string()))
}

fn resolve_redaction_preserve_length(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<bool>> {
	if let Some(value) = extract_redaction_preserve_length(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_redaction_preserve_length(user_root, user_path)? {
		return Ok(Some(value));
	}

	Ok(None)
}

fn extract_redaction_preserve_length(
	config_root: Option<&Value>,
	_config_path: &Path,
) -> Result<Option<bool>> {
	Ok(config_root
		.and_then(|root| root.get("redaction"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("preserve_length"))
		.and_then(Value::as_bool))
}

fn resolve_redaction_rules(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<Vec<RedactionRule>>> {
	if let Some(value) = extract_redaction_rules(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_redaction_rules(user_root, user_path)? {
		return Ok(Some(value));
	}

	Ok(None)
}

fn extract_redaction_rules(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<Vec<RedactionRule>>> {
	let Some(value) = config_root
		.and_then(|root| root.get("redaction"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("rules"))
	else {
		return Ok(None);
	};

	let Some(rules) = value.as_array() else {
		return Err(anyhow!(
			"config field redaction.rules must be an array in {}",
			config_path.display()
		));
	};

	let mut parsed = Vec::new();
	for rule in rules {
		let Some(object) = rule.as_object() else {
			return Err(anyhow!(
				"config field redaction.rules must contain objects in {}",
				config_path.display()
			));
		};

		let kind = object
			.get("kind")
			.and_then(Value::as_str)
			.map(|value| value.trim().to_lowercase());

		let parsed_rule = match kind.as_deref() {
			Some("structured_key") | Some("structured_keys") | None
				if object.get("keys").is_some() =>
			{
				let Some(keys) = object.get("keys").and_then(Value::as_array) else {
					return Err(anyhow!(
						"config field redaction.rules[].keys must be an array in {}",
						config_path.display()
					));
				};
				let keys = keys
					.iter()
					.map(|value| {
						value.as_str().map(|text| text.to_string()).ok_or_else(|| {
							anyhow!(
								"config field redaction.rules[].keys must contain strings in {}",
								config_path.display()
							)
						})
					})
					.collect::<Result<Vec<_>>>()?;
				RedactionRule::StructuredKeys { keys }
			}
			Some("regex") | None if object.get("pattern").is_some() => {
				let Some(pattern) = object.get("pattern").and_then(Value::as_str) else {
					return Err(anyhow!(
						"config field redaction.rules[].pattern must be a string in {}",
						config_path.display()
					));
				};
				let case_sensitive = object
					.get("case_sensitive")
					.and_then(Value::as_bool)
					.unwrap_or(false);
				RedactionRule::Regex {
					pattern: pattern.to_string(),
					case_sensitive,
				}
			}
			Some("builtin_email") | Some("email") => RedactionRule::BuiltinEmail,
			Some("builtin_bearer") | Some("builtin_bearer_token") | Some("bearer") => {
				RedactionRule::BuiltinBearerToken
			}
			Some(other) => {
				return Err(anyhow!(
					"config field redaction.rules[].kind is not supported ({other}) in {}",
					config_path.display()
				));
			}
			None => {
				return Err(anyhow!(
					"config field redaction.rules[] must include kind, keys, or pattern in {}",
					config_path.display()
				));
			}
		};

		parsed.push(parsed_rule);
	}

	Ok(Some(parsed))
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

	let normalized = crate::path_encoding::normalize_inbound_separators(path);
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
	workspace_root.join(WORKSPACE_STORAGE_DIR).join("index")
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

fn workspace_id(workspace_root: &Path) -> String {
	workspace_index_key(workspace_root)
}

fn resolve_storage_user_location(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<StorageUserLocation>> {
	if let Some(location) = extract_storage_user_location(workspace_root, workspace_path)? {
		return Ok(Some(location));
	}
	if let Some(location) = extract_storage_user_location(user_root, user_path)? {
		return Ok(Some(location));
	}
	Ok(None)
}

fn extract_storage_user_location(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<StorageUserLocation>> {
	let Some(value) = config_root
		.and_then(|root| root.get("storage"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("user"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("location"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	if value.eq_ignore_ascii_case("os") {
		return Ok(Some(StorageUserLocation::Os));
	}
	if value.eq_ignore_ascii_case("workspace") {
		return Ok(Some(StorageUserLocation::Workspace));
	}

	Err(anyhow!(
		"config field storage.user.location must be 'os' or 'workspace' in {}",
		config_path.display()
	))
}

fn resolve_storage_workspace_location(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<StorageWorkspaceLocation>> {
	if let Some(location) = extract_storage_workspace_location(workspace_root, workspace_path)? {
		return Ok(Some(location));
	}
	if let Some(location) = extract_storage_workspace_location(user_root, user_path)? {
		return Ok(Some(location));
	}
	Ok(None)
}

fn extract_storage_workspace_location(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<StorageWorkspaceLocation>> {
	let Some(value) = config_root
		.and_then(|root| root.get("storage"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("workspace"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("location"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	if value.eq_ignore_ascii_case("workspace") {
		return Ok(Some(StorageWorkspaceLocation::Workspace));
	}
	if value.eq_ignore_ascii_case("user") {
		return Ok(Some(StorageWorkspaceLocation::User));
	}

	Err(anyhow!(
		"config field storage.workspace.location must be 'workspace' or 'user' in {}",
		config_path.display()
	))
}

fn resolve_runtime_log_location(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<RuntimeLogLocation>> {
	if let Some(value) = extract_runtime_log_location(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_runtime_log_location(user_root, user_path)? {
		return Ok(Some(value));
	}
	Ok(None)
}

fn extract_runtime_log_location(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<RuntimeLogLocation>> {
	let Some(value) = config_root
		.and_then(|root| root.get("logging"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("runtime"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("location"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	if value.eq_ignore_ascii_case("auto") {
		return Ok(Some(RuntimeLogLocation::Auto));
	}
	if value.eq_ignore_ascii_case("workspace") {
		return Ok(Some(RuntimeLogLocation::Workspace));
	}
	if value.eq_ignore_ascii_case("user_logs") {
		return Ok(Some(RuntimeLogLocation::UserLogs));
	}
	if value.eq_ignore_ascii_case("user_data") {
		return Ok(Some(RuntimeLogLocation::UserData));
	}
	if value.eq_ignore_ascii_case("system_logs") {
		return Ok(Some(RuntimeLogLocation::SystemLogs));
	}

	Err(anyhow!(
		"config field logging.runtime.location must be one of 'auto', 'workspace', 'user_logs', 'user_data', 'system_logs' in {}",
		config_path.display()
	))
}

fn resolve_runtime_log_path(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<String>> {
	if let Some(value) = extract_runtime_log_path(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_runtime_log_path(user_root, user_path)? {
		return Ok(Some(value));
	}
	Ok(None)
}

fn extract_runtime_log_path(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<String>> {
	let Some(value) = config_root
		.and_then(|root| root.get("logging"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("runtime"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("path"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	let trimmed = value.trim();
	if trimmed.is_empty() {
		return Err(anyhow!(
			"config field logging.runtime.path must not be empty in {}",
			config_path.display()
		));
	}

	Ok(Some(trimmed.to_string()))
}

fn resolve_runtime_log_retention_days(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<u64>> {
	if let Some(value) = extract_runtime_log_retention_days(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_runtime_log_retention_days(user_root, user_path)? {
		return Ok(Some(value));
	}
	Ok(None)
}

fn extract_runtime_log_retention_days(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<u64>> {
	let Some(value) = config_root
		.and_then(|root| root.get("logging"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("runtime"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("retention_days"))
	else {
		return Ok(None);
	};

	let Some(days) = value.as_u64() else {
		return Err(anyhow!(
			"config field logging.runtime.retention_days must be an integer in {}",
			config_path.display()
		));
	};
	if days < 1 {
		return Err(anyhow!(
			"config field logging.runtime.retention_days must be at least 1 in {}",
			config_path.display()
		));
	}

	Ok(Some(days))
}

fn resolve_memory_user_enabled(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<bool>> {
	if let Some(value) = extract_memory_user_enabled(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_memory_user_enabled(user_root, user_path)? {
		return Ok(Some(value));
	}
	Ok(None)
}

fn extract_memory_user_enabled(
	config_root: Option<&Value>,
	_config_path: &Path,
) -> Result<Option<bool>> {
	Ok(config_root
		.and_then(|root| root.get("memory"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("storage"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("user"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("enabled"))
		.and_then(Value::as_bool))
}

fn resolve_memory_workspace_enabled(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<bool>> {
	if let Some(value) = extract_memory_workspace_enabled(workspace_root, workspace_path)? {
		return Ok(Some(value));
	}
	if let Some(value) = extract_memory_workspace_enabled(user_root, user_path)? {
		return Ok(Some(value));
	}
	Ok(None)
}

fn extract_memory_workspace_enabled(
	config_root: Option<&Value>,
	_config_path: &Path,
) -> Result<Option<bool>> {
	Ok(config_root
		.and_then(|root| root.get("memory"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("storage"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("workspace"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("enabled"))
		.and_then(Value::as_bool))
}

fn resolve_memory_user_path(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<String>> {
	if let Some(path) = extract_memory_user_path(workspace_root, workspace_path)? {
		return Ok(Some(path));
	}
	if let Some(path) = extract_memory_user_path(user_root, user_path)? {
		return Ok(Some(path));
	}
	Ok(None)
}

fn extract_memory_user_path(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<String>> {
	let Some(value) = config_root
		.and_then(|root| root.get("memory"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("storage"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("user"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("path"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	Ok(Some(normalize_workspace_relative_dir(value, config_path)?))
}

fn resolve_memory_workspace_path(
	user_root: Option<&Value>,
	workspace_root: Option<&Value>,
	user_path: &Path,
	workspace_path: &Path,
) -> Result<Option<String>> {
	if let Some(path) = extract_memory_workspace_path(workspace_root, workspace_path)? {
		return Ok(Some(path));
	}
	if let Some(path) = extract_memory_workspace_path(user_root, user_path)? {
		return Ok(Some(path));
	}
	Ok(None)
}

fn extract_memory_workspace_path(
	config_root: Option<&Value>,
	config_path: &Path,
) -> Result<Option<String>> {
	let Some(value) = config_root
		.and_then(|root| root.get("memory"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("storage"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("workspace"))
		.and_then(|value| value.as_object())
		.and_then(|object| object.get("path"))
		.and_then(Value::as_str)
	else {
		return Ok(None);
	};

	Ok(Some(normalize_workspace_relative_dir(value, config_path)?))
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
