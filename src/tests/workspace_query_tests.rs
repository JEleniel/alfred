use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::tools::workspace_query;
use crate::tools::{ToolCallResult, dispatch_tool_call as dispatch_tool_call_outcome};

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

fn fixture_workspace() -> PathBuf {
	std::env::current_dir()
		.expect("workspace current dir should resolve")
		.join("src")
		.join("testdata")
		.join("indexer")
}

fn copy_fixture_tree(source: &Path, destination: &Path) {
	for entry in fs::read_dir(source).expect("fixture directory should be readable") {
		let entry = entry.expect("fixture entry should be valid");
		let source_path = entry.path();
		let destination_path = destination.join(entry.file_name());
		if source_path.is_dir() {
			fs::create_dir_all(&destination_path).expect("destination directory should be created");
			copy_fixture_tree(&source_path, &destination_path);
		} else {
			fs::copy(&source_path, &destination_path).expect("fixture file should copy");
		}
	}
}

fn build_services(with_index: bool) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("workspace-query-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	let services = ServiceContainer::new(config).expect("service container should build");
	if with_index {
		services
			.indexer
			.rebuild()
			.expect("fixture index should build");
	}

	(services, workspace)
}

fn dispatch_tool_call(
	name: &str,
	args: serde_json::Value,
	services: &ServiceContainer,
) -> Result<serde_json::Value, AlfredError> {
	match dispatch_tool_call_outcome(name, args, services)? {
		ToolCallResult::Ok(data) => Ok(data),
		ToolCallResult::Pending { .. } => Err(AlfredError::Internal(
			"unexpected pending response in sync test".to_string(),
		)),
	}
}

#[test]
fn search_returns_tool_unavailable_when_index_is_not_ready() {
	let (services, _workspace) = build_services(false);

	let result =
		workspace_query::dispatch_tool_call("search", json!({"query": "hello"}), &services);

	match result {
		Err(AlfredError::ToolUnavailable { message, details }) => {
			assert_eq!(message, "workspace index is not ready");
			assert_eq!(details, Some(json!({"reason": "index_not_ready"})));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn search_supports_regex_mode_and_pagination() {
	let (services, _workspace) = build_services(true);

	let first_page = workspace_query::dispatch_tool_call(
		"search",
		json!({
			"query": "Hello from (alpha|beta)",
			"mode": "regex",
			"limit": 1
		}),
		&services,
	)
	.expect("search should succeed")
	.expect("search should return data");

	let first_matches = first_page["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(first_matches.len(), 1);
	assert_eq!(first_matches[0]["path"], json!("alpha.txt"));
	assert_eq!(first_page["next_cursor"], json!("1"));

	let second_page = workspace_query::dispatch_tool_call(
		"search",
		json!({
			"query": "Hello from (alpha|beta)",
			"mode": "regex",
			"cursor": "1",
			"limit": 2
		}),
		&services,
	)
	.expect("search should succeed")
	.expect("search should return data");

	let second_matches = second_page["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(second_matches.len(), 1);
	assert_eq!(second_matches[0]["path"], json!("nested/beta.md"));
	assert!(second_page["next_cursor"].is_null());
}

#[test]
fn index_backed_tools_return_tool_unavailable_when_index_is_disabled() {
	let workspace = TestDir::new("workspace-query-disabled-index-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let mut config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	config.index_enabled = false;
	let services = ServiceContainer::new(config).expect("service container should build");
	services
		.indexer
		.rebuild()
		.expect("fixture index should build");

	let result =
		workspace_query::dispatch_tool_call("search", json!({"query": "hello"}), &services);
	match result {
		Err(AlfredError::ToolUnavailable { message, details }) => {
			assert_eq!(message, "workspace index is disabled");
			assert_eq!(details, Some(json!({"reason": "index_disabled"})));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn file_read_bytes_is_not_implemented() {
	let (services, _workspace) = build_services(true);

	let result = dispatch_tool_call(
		"file_read_bytes",
		json!({
			"path": "alpha.txt",
			"offset": 0,
			"length": 1
		}),
		&services,
	);

	match result {
		Err(AlfredError::InvalidArgument(message)) => {
			assert_eq!(message, "tool not implemented: file_read_bytes");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}
