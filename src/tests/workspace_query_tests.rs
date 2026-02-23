use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::tools::workspace_query;
use crate::tools::workspace_query::MAX_FILE_CHUNK_BYTES;

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

	let mut config = AppConfig::load_default().expect("default config should load");
	config.workspace_root = workspace.path.clone();
	let services = ServiceContainer::new(config).expect("service container should build");
	if with_index {
		services
			.indexer
			.rebuild()
			.expect("fixture index should build");
	}

	(services, workspace)
}

#[test]
fn grep_returns_tool_unavailable_when_index_is_not_ready() {
	let (services, _workspace) = build_services(false);

	let result = workspace_query::dispatch_tool_call("grep", json!({"query": "hello"}), &services);

	match result {
		Err(AlfredError::ToolUnavailable { message, details }) => {
			assert_eq!(message, "workspace index is not ready");
			assert_eq!(details, Some(json!({"reason": "index_not_ready"})));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn search_returns_tool_unavailable_when_index_is_not_ready() {
	let (services, _workspace) = build_services(false);

	let result = workspace_query::dispatch_tool_call(
		"search",
		json!({"search": {"query": "hello"}}),
		&services,
	);

	match result {
		Err(AlfredError::ToolUnavailable { message, details }) => {
			assert_eq!(message, "workspace index is not ready");
			assert_eq!(details, Some(json!({"reason": "index_not_ready"})));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn read_range_collapses_dot_segments() {
	let (services, _workspace) = build_services(true);

	let data = workspace_query::dispatch_tool_call(
		"read_range",
		json!({
			"path": "./nested/../nested/./beta.md",
			"start_line": 1,
			"end_line": 1
		}),
		&services,
	)
	.expect("read_range should succeed")
	.expect("read_range should return data");

	assert_eq!(data["path"], json!("nested/beta.md"));
	assert_eq!(data["text"], json!("Hello from beta"));
}

#[test]
fn read_range_rejects_boundary_escape_segments() {
	let (services, _workspace) = build_services(true);

	let result = workspace_query::dispatch_tool_call(
		"read_range",
		json!({
			"path": "../../etc/passwd",
			"start_line": 1,
			"end_line": 1
		}),
		&services,
	);

	match result {
		Err(AlfredError::WorkspaceBoundaryViolation(_)) => {}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn file_read_bytes_rejects_length_above_max_file_chunk_bytes() {
	let (services, _workspace) = build_services(true);

	let result = workspace_query::dispatch_tool_call(
		"file_read_bytes",
		json!({
			"path": "alpha.txt",
			"offset": 0,
			"length": MAX_FILE_CHUNK_BYTES + 1
		}),
		&services,
	);

	match result {
		Err(AlfredError::ResourceExhausted(message)) => {
			assert_eq!(
				message,
				format!("length exceeds max_file_chunk_bytes: {MAX_FILE_CHUNK_BYTES}")
			);
		}
		other => panic!("unexpected result: {other:?}"),
	}
}
