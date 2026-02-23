use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::configuration::{
	AppConfig, default_runtime_log_path, default_workspace_index_root,
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
fn runtime_log_path_uses_user_data_alfred_subfolder() {
	let workspace_root = Path::new("/workspace-root");
	let runtime_path = default_runtime_log_path(workspace_root);

	assert!(runtime_path.ends_with(Path::new("alfred/logs/runtime.ndjson")));
}

#[test]
fn runtime_log_path_matches_data_dir_or_workspace_fallback() {
	let workspace_root = Path::new("/workspace-root");
	let runtime_path = default_runtime_log_path(workspace_root);

	if let Some(data_dir) = dirs::data_dir() {
		assert_eq!(
			runtime_path,
			data_dir.join("alfred").join("logs").join("runtime.ndjson")
		);
	} else {
		assert_eq!(
			runtime_path,
			workspace_root
				.join(".agents")
				.join("alfred")
				.join("logs")
				.join("runtime.ndjson")
		);
	}
}

#[test]
fn workspace_index_root_uses_user_data_alfred_subfolder() {
	let workspace_root = Path::new("/workspace-root");
	let index_root = default_workspace_index_root(workspace_root);

	assert!(index_root.to_string_lossy().contains("alfred/index/"));
}

#[test]
fn workspace_local_index_root_uses_agents_alfred_index() {
	let workspace_root = Path::new("/workspace-root");
	let index_root = default_workspace_local_index_root(workspace_root);

	assert_eq!(
		index_root,
		workspace_root.join(".agents").join("alfred").join("index")
	);
}

#[test]
fn config_load_from_paths_merges_disabled_tools_union() {
	let workspace = TestDir::new("configuration-merge-tests");
	let user_config_path = workspace.path.join("user-config.json");
	let workspace_config_path = workspace.path.join(".agents/alfred/config.json");

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
