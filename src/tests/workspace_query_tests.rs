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

#[test]
fn ls_returns_normalized_stable_sorted_entries() {
	let (services, _workspace) = build_services(true);

	let data = workspace_query::dispatch_tool_call(
		"ls",
		json!({
			"path": "./nested/../",
			"recursive": false
		}),
		&services,
	)
	.expect("ls should succeed")
	.expect("ls should return data");

	assert_eq!(data["files"], json!(["alpha.txt", "Zeta.txt"]));
	assert_eq!(data["directories"], json!(["nested"]));
	assert!(data["next_cursor"].is_null());
}

#[test]
fn file_stat_returns_normalized_file_metadata() {
	let (services, _workspace) = build_services(true);

	let data = workspace_query::dispatch_tool_call(
		"file_stat",
		json!({"path": "./nested/../alpha.txt"}),
		&services,
	)
	.expect("file_stat should succeed")
	.expect("file_stat should return data");

	assert_eq!(data["path"], json!("alpha.txt"));
	assert_eq!(data["kind"], json!("file"));
	assert!(data["size_bytes"].as_u64().is_some_and(|value| value > 0));
	assert!(data["modified_at"].as_str().is_some());
}

#[test]
fn file_read_bytes_returns_chunk_and_next_offset() {
	let (services, _workspace) = build_services(true);

	let data = workspace_query::dispatch_tool_call(
		"file_read_bytes",
		json!({
			"path": "./nested/../alpha.txt",
			"offset": 0,
			"length": 5
		}),
		&services,
	)
	.expect("file_read_bytes should succeed")
	.expect("file_read_bytes should return data");

	assert_eq!(data["path"], json!("alpha.txt"));
	assert_eq!(data["bytes_b64"], json!("SGVsbG8="));
	assert_eq!(data["bytes_read"], json!(5));
	assert_eq!(data["eof"], json!(false));
	assert_eq!(data["next_offset"], json!(5));
}

#[test]
fn diff_returns_deterministic_unified_diff() {
	let (services, _workspace) = build_services(true);

	let data = workspace_query::dispatch_tool_call(
		"diff",
		json!({
			"a": {
				"path": "./nested/../alpha.txt",
				"from": 1,
				"to": 1
			},
			"b": {
				"path": "nested/beta.md",
				"from": 1,
				"to": 1
			}
		}),
		&services,
	)
	.expect("diff should succeed")
	.expect("diff should return data");

	assert_eq!(
		data["diff"],
		json!("--- a\n+++ b\n@@ -1,1 +1,1 @@\n-Hello from alpha\n+Hello from beta\n")
	);
}

#[test]
fn read_range_rejects_binary_non_text_input_deterministically() {
	let (services, workspace) = build_services(false);
	fs::write(
		workspace.path.join("binary.bin"),
		[0xFF_u8, 0xFE_u8, 0x00_u8, 0x01_u8],
	)
	.expect("binary fixture should be written");
	services
		.indexer
		.rebuild()
		.expect("index should rebuild with binary fixture");

	let result = workspace_query::dispatch_tool_call(
		"read_range",
		json!({
			"path": "binary.bin",
			"start_line": 1,
			"end_line": 1
		}),
		&services,
	);

	match result {
		Err(AlfredError::InvalidArgument(message)) => {
			assert_eq!(message, "file is not available as UTF-8 text: binary.bin");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}
