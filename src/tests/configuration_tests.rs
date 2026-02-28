use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::configuration::{
	AppConfig, IndexPersistenceLocation, default_runtime_log_path, default_workspace_index_root,
	default_workspace_local_index_root,
};
use uuid::Uuid;

struct TestDir {
	path: PathBuf,
}

impl TestDir {
	fn new(prefix: &str) -> Self {
		let path = std::env::current_dir()
			.expect("workspace current dir should resolve")
			.join("tmp")
			.join(format!("{prefix}-{}", Uuid::new_v4()));
		fs::create_dir_all(&path).expect("test directory should be created");
		Self { path }
	}
}

impl Drop for TestDir {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.path);
	}
}

#[test]
fn runtime_log_path_uses_workspace_scoped_alfred_logs() {
	let workspace_root = Path::new("/workspace-root");
	let runtime_path = default_runtime_log_path(workspace_root);

	assert_eq!(
		runtime_path,
		workspace_root
			.join(".alfred")
			.join("logs")
			.join("runtime.ndjson")
	);
}

#[test]
fn workspace_index_root_is_under_workspace_scoped_alfred_root() {
	let workspace_root = Path::new("/workspace-root");
	let index_root = default_workspace_index_root(workspace_root);

	assert!(index_root.starts_with(workspace_root.join(".alfred").join("index")));
}

#[test]
fn workspace_local_index_root_uses_alfred_index() {
	let workspace_root = Path::new("/workspace-root");
	let index_root = default_workspace_local_index_root(workspace_root);

	assert_eq!(index_root, workspace_root.join(".alfred").join("index"));
}

#[test]
fn config_load_from_paths_merges_disabled_tools_union() {
	let workspace = TestDir::new("configuration-merge-tests");
	let user_config_path = workspace.path.join("user-config.json");
	let workspace_config_path = workspace.path.join(".alfred/config.json");

	fs::create_dir_all(
		workspace_config_path
			.parent()
			.expect("workspace config parent should resolve"),
	)
	.expect("workspace config parent directory should be created");
	fs::write(
		&user_config_path,
		r#"{"tools":{"disabled":["grep","search"]}}"#,
	)
	.expect("user config should be written");
	fs::write(
		&workspace_config_path,
		r#"{"tools":{"disabled":["grep","workspace_dir"]}}"#,
	)
	.expect("workspace config should be written");

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		user_config_path,
		workspace_config_path,
	)
	.expect("config should load from explicit paths");

	assert!(config.disabled_tools.contains(&"grep".to_string()));
	assert!(config.disabled_tools.contains(&"search".to_string()));
	assert!(config.disabled_tools.contains(&"workspace_dir".to_string()));
	assert!(config.disabled_tools.contains(&"file_delete".to_string()));
}

#[test]
fn config_load_creates_workspace_scaffold_gitignore() {
	let workspace = TestDir::new("configuration-scaffold-tests");
	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join("missing-workspace-config.json"),
	)
	.expect("config should load when files are missing");

	let gitignore_path = config.workspace_root.join(".alfred").join(".gitignore");
	assert!(gitignore_path.exists());
	let content = fs::read_to_string(gitignore_path).expect("gitignore should be readable");
	assert_eq!(content, "*\n!config.json\n");
}

#[test]
fn config_load_honors_workspace_storage_root_for_artifacts() {
	let workspace = TestDir::new("configuration-storage-root-tests");
	let user_config_path = workspace.path.join("missing-user-config.json");
	let workspace_config_path = workspace.path.join(".alfred/config.json");

	fs::create_dir_all(
		workspace_config_path
			.parent()
			.expect("workspace config parent should resolve"),
	)
	.expect("workspace config parent directory should be created");
	fs::write(
		&workspace_config_path,
		r#"{"workspace":{"storage":{"root":".state"}},"index":{"enabled":false,"persistence":{"location":"workspace"}}}"#,
	)
	.expect("workspace config should be written");

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		user_config_path,
		workspace_config_path,
	)
	.expect("config should load from explicit paths");

	assert_eq!(config.workspace_storage_root_dir, ".state".to_string());
	assert_eq!(
		config.effective_workspace_storage_root(),
		config.workspace_root.join(".state")
	);
	assert_eq!(
		config.effective_workspace_index_root(),
		config.workspace_root.join(".state").join("index")
	);
	assert_eq!(
		config.effective_runtime_log_path(),
		config
			.workspace_root
			.join(".state")
			.join("logs")
			.join("runtime.ndjson")
	);
	assert!(!config.index_enabled);

	let gitignore_path = config.workspace_root.join(".state").join(".gitignore");
	assert!(gitignore_path.exists());
}

#[test]
fn config_load_honors_index_persistence_location_and_path_overrides() {
	let workspace = TestDir::new("configuration-index-location-tests");
	let user_config_path = workspace.path.join("missing-user-config.json");
	let workspace_config_path = workspace.path.join(".alfred/config.json");

	fs::create_dir_all(
		workspace_config_path
			.parent()
			.expect("workspace config parent should resolve"),
	)
	.expect("workspace config parent directory should be created");
	fs::write(
		&workspace_config_path,
		r#"{"index":{"persistence":{"location":"user","path":"workspaces/custom/index"}}}"#,
	)
	.expect("workspace config should be written");

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		user_config_path.clone(),
		workspace_config_path.clone(),
	)
	.expect("config should load from explicit paths");

	assert_eq!(
		config.index_persistence_location,
		IndexPersistenceLocation::UserData
	);
	if let Some(data_dir) = dirs::data_dir() {
		assert_eq!(
			config.effective_workspace_index_root(),
			data_dir.join("workspaces/custom/index")
		);
	} else {
		assert!(
			config
				.effective_workspace_index_root()
				.starts_with(config.workspace_root.join(".alfred"))
		);
	}

	fs::write(
		&workspace_config_path,
		r#"{"index":{"persistence":{"location":"workspace","path":".state/index"}}}"#,
	)
	.expect("workspace config should be overwritten");
	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		user_config_path,
		workspace_config_path,
	)
	.expect("config should reload");
	assert_eq!(
		config.index_persistence_location,
		IndexPersistenceLocation::Workspace
	);
	assert_eq!(
		config.effective_workspace_index_root(),
		config.workspace_root.join(".state").join("index")
	);
}

#[test]
fn config_default_policy_disables_mutating_tools() {
	let workspace = TestDir::new("configuration-policy-tests");
	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join("missing-workspace-config.json"),
	)
	.expect("config should load when files are missing");

	assert!(!config.is_tool_enabled("file_delete"));
	assert!(!config.is_tool_enabled("memory_put"));
	assert!(!config.is_tool_enabled("plan_update"));
	assert!(!config.is_tool_enabled("env_set"));
	assert!(!config.is_tool_enabled("task_run"));
	assert!(config.is_tool_enabled("workspace_dir"));
}

#[test]
fn config_derives_user_ignore_path_from_user_config_parent() {
	let workspace = TestDir::new("configuration-user-ignore-path");
	let user_config_path = workspace.path.join("alfred").join("config.json");
	let workspace_config_path = workspace.path.join(".alfred/config.json");

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		user_config_path.clone(),
		workspace_config_path,
	)
	.expect("config should load from explicit paths");

	assert_eq!(
		config.user_ignore_path,
		user_config_path
			.parent()
			.expect("user config parent should resolve")
			.join(".alfredignore")
	);
}
