use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::services::ServiceContainer;
use crate::tools::dispatch_tool_call;

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

fn build_services(workspace_root: PathBuf) -> ServiceContainer {
	let config = AppConfig::load_from_paths(
		workspace_root.clone(),
		workspace_root.join("missing-user-config.json"),
		workspace_root.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	ServiceContainer::new(config).expect("service container should build")
}

#[test]
fn status_tool_reports_index_and_memory() {
	let workspace = TestDir::new("status-tool-tests");
	let services = build_services(workspace.path.clone());

	let result =
		dispatch_tool_call("status", json!({}), &services).expect("status call should succeed");

	assert_eq!(
		result["workspace_root"],
		workspace.path.to_string_lossy().as_ref()
	);
	assert!(result["index"]["ready"].is_boolean());
	assert!(result["index"]["indexed_files"].is_number());
	assert_eq!(result["memory"]["unit"], "bytes");
	assert!(result["memory"]["system"]["total"].is_number());
	assert!(result["memory"]["process"]["rss"].is_number());
}
