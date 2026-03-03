use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use crate::configuration::{AppConfig, HostPaths};
use crate::services::ServiceContainer;
use crate::services::memory_store::{MemoryFactInput, MemoryScope};

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

fn write_user_config(host_config_root: &PathBuf, content: &str) {
	let alfred_root = host_config_root.join("alfred");
	fs::create_dir_all(&alfred_root).expect("host config root should be created");
	fs::write(alfred_root.join("config.json"), content).expect("user config should be written");
}

fn build_host_paths(workspace: &TestDir) -> (HostPaths, PathBuf) {
	let host_config = workspace.path.join("host_config");
	let host_data = workspace.path.join("host_data");
	fs::create_dir_all(&host_data).expect("host data root should be created");

	(
		HostPaths {
			user_config_dir: Some(host_config),
			user_data_dir: Some(host_data.clone()),
			user_logs_dir: Some(workspace.path.join("host_logs")),
			system_logs_dir: None,
		},
		host_data,
	)
}

fn sample_input(id: &str) -> MemoryFactInput {
	MemoryFactInput {
		id: id.to_string(),
		subject: "subject".to_string(),
		fact: "fact".to_string(),
		citations: "citations".to_string(),
		reason: "reason".to_string(),
		category: "general".to_string(),
		tags: vec!["tag".to_string()],
	}
}

#[test]
fn workspace_index_persists_only_in_selected_location() {
	let workspace = TestDir::new("persistence-index-single-location");
	let (host, host_data) = build_host_paths(&workspace);
	let host_config = host
		.user_config_dir
		.clone()
		.expect("host config path should exist");
	write_user_config(
		&host_config,
		r#"{
			"index": {"persistence": {"location": "user"}}
		}"#,
	);

	let config = AppConfig::load_with_host_paths(workspace.path.clone(), host)
		.expect("config should load with host paths");
	let selected = config.effective_workspace_index_root();
	let workspace_local = config
		.workspace_root
		.join(config.workspace_storage_root_dir.as_str())
		.join("index");
	assert_ne!(selected, workspace_local);

	let services = ServiceContainer::new(config).expect("service container should build");
	services
		.indexer
		.rebuild()
		.expect("workspace index should rebuild");

	assert!(selected.exists());
	assert!(!workspace_local.exists());
	assert!(selected.starts_with(host_data));
}

#[test]
fn memory_scopes_persist_once_when_user_scope_is_workspace_local() {
	let workspace = TestDir::new("persistence-memory-workspace-user-scope");
	let (host, host_data) = build_host_paths(&workspace);
	let host_config = host
		.user_config_dir
		.clone()
		.expect("host config path should exist");
	write_user_config(
		&host_config,
		r#"{
			"storage": {
				"user": {"location": "workspace"},
				"workspace": {"location": "workspace"}
			},
			"workspace": {"storage": {"root": ".state"}}
		}"#,
	);
	let workspace_user_config_dir = workspace.path.join(".state").join("user");
	fs::create_dir_all(&workspace_user_config_dir)
		.expect("workspace-local user config directory should be created");
	fs::write(
		workspace_user_config_dir.join("config.json"),
		r#"{
			"memory": {"storage": {"workspace": {"enabled": true}}}
		}"#,
	)
	.expect("workspace-local user config should be written");

	let config = AppConfig::load_with_host_paths(workspace.path.clone(), host)
		.expect("config should load with host paths");
	let user_selected = config
		.effective_user_memory_store_path()
		.expect("user memory store should be enabled")
		.with_extension("tantivy");
	let workspace_selected = config
		.effective_workspace_memory_store_path()
		.expect("workspace memory store should be enabled")
		.with_extension("tantivy");

	let services = ServiceContainer::new(config.clone()).expect("service container should build");
	services
		.memory_store
		.upsert_in_scope(MemoryScope::User, sample_input("user-fact"))
		.expect("user fact should persist");
	services
		.memory_store
		.upsert_in_scope(MemoryScope::Workspace, sample_input("workspace-fact"))
		.expect("workspace fact should persist");

	let os_user_alternative = host_data
		.join("alfred")
		.join("memory")
		.join("alfred")
		.with_extension("tantivy");
	let user_workspace_alternative = host_data
		.join("alfred")
		.join(config.workspace_id.as_str())
		.join("data")
		.join("memory")
		.join("alfred")
		.with_extension("tantivy");

	assert!(user_selected.exists());
	assert!(workspace_selected.exists());
	assert_ne!(user_selected, workspace_selected);
	assert!(!os_user_alternative.exists());
	assert!(!user_workspace_alternative.exists());
}

#[test]
fn memory_scopes_persist_once_when_workspace_scope_is_user_relocated() {
	let workspace = TestDir::new("persistence-memory-user-relocated-workspace-scope");
	let (host, host_data) = build_host_paths(&workspace);
	let host_config = host
		.user_config_dir
		.clone()
		.expect("host config path should exist");
	write_user_config(
		&host_config,
		r#"{
			"storage": {"workspace": {"location": "user"}},
			"memory": {"storage": {"workspace": {"enabled": true}}}
		}"#,
	);

	let config = AppConfig::load_with_host_paths(workspace.path.clone(), host)
		.expect("config should load with host paths");
	let user_selected = config
		.effective_user_memory_store_path()
		.expect("user memory store should be enabled")
		.with_extension("tantivy");
	let workspace_selected = config
		.effective_workspace_memory_store_path()
		.expect("workspace memory store should be enabled")
		.with_extension("tantivy");

	let services = ServiceContainer::new(config.clone()).expect("service container should build");
	services
		.memory_store
		.upsert_in_scope(MemoryScope::User, sample_input("user-fact"))
		.expect("user fact should persist");
	services
		.memory_store
		.upsert_in_scope(MemoryScope::Workspace, sample_input("workspace-fact"))
		.expect("workspace fact should persist");

	let workspace_local_alternative = config
		.workspace_root
		.join(config.workspace_storage_root_dir.as_str())
		.join("memory")
		.join("alfred")
		.with_extension("tantivy");
	let user_workspace_alternative = config
		.workspace_root
		.join(config.workspace_storage_root_dir.as_str())
		.join("user")
		.join("memory")
		.join("alfred")
		.with_extension("tantivy");

	assert!(user_selected.exists());
	assert!(workspace_selected.exists());
	assert_ne!(user_selected, workspace_selected);
	assert!(workspace_selected.starts_with(host_data));
	assert!(!workspace_local_alternative.exists());
	assert!(!user_workspace_alternative.exists());
}
