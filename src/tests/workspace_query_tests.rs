use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::tools::dispatch_tool_call;
use crate::tools::workspace_query;

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

fn wait_for_mtime_change(path: &Path, baseline: std::time::SystemTime) {
	for _ in 0..50 {
		let modified = fs::metadata(path)
			.ok()
			.and_then(|metadata| metadata.modified().ok());
		if modified.is_some_and(|value| value > baseline) {
			return;
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
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

	for tool in ["ls", "grep", "search"] {
		let args = match tool {
			"ls" => json!({}),
			"grep" => json!({"query": "hello"}),
			"search" => json!({"query": "hello"}),
			_ => unreachable!(),
		};
		let result = workspace_query::dispatch_tool_call(tool, args, &services);
		match result {
			Err(AlfredError::ToolUnavailable { message, details }) => {
				assert_eq!(message, "workspace index is disabled");
				assert_eq!(details, Some(json!({"reason": "index_disabled"})));
			}
			other => panic!("unexpected result for {tool}: {other:?}"),
		}
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
fn read_range_falls_back_to_disk_when_index_snapshot_is_stale() {
	let (services, workspace) = build_services(true);

	let absolute_path = workspace.path.join("alpha.txt");
	let baseline_mtime = fs::metadata(&absolute_path)
		.expect("alpha.txt metadata should load")
		.modified()
		.expect("alpha.txt modified time should resolve");
	std::thread::sleep(std::time::Duration::from_millis(1100));

	let original = workspace_query::dispatch_tool_call(
		"read_range",
		json!({
			"path": "alpha.txt",
			"start_line": 1,
			"end_line": 1
		}),
		&services,
	)
	.expect("read_range should succeed")
	.expect("read_range should return data");
	assert_eq!(original["text"], json!("Hello from alpha"));

	fs::write(&absolute_path, "Hello from alpha (updated)\n")
		.expect("alpha.txt should be rewritten");
	wait_for_mtime_change(&absolute_path, baseline_mtime);

	let updated = workspace_query::dispatch_tool_call(
		"read_range",
		json!({
			"path": "alpha.txt",
			"start_line": 1,
			"end_line": 1
		}),
		&services,
	)
	.expect("read_range should succeed")
	.expect("read_range should return data");
	assert_eq!(updated["text"], json!("Hello from alpha (updated)"));
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

#[test]
fn read_range_supports_large_files_without_full_memory_load() {
	let (services, workspace) = build_services(false);
	let absolute_path = workspace.path.join("large.txt");
	let file = File::create(&absolute_path).expect("large fixture file should create");
	let mut writer = BufWriter::new(file);
	let filler = "x".repeat(180);

	for line in 1..=30_000u32 {
		if line == 25_000 {
			writeln!(writer, "line {line} UNIQUE_TOKEN {filler}")
				.expect("large fixture line should write");
		} else {
			writeln!(writer, "line {line} {filler}").expect("large fixture line should write");
		}
	}
	writer.flush().expect("large fixture should flush");

	services
		.indexer
		.rebuild()
		.expect("index should rebuild with large fixture");

	let range = workspace_query::dispatch_tool_call(
		"read_range",
		json!({
			"path": "large.txt",
			"start_line": 25_000,
			"end_line": 25_000
		}),
		&services,
	)
	.expect("read_range should succeed")
	.expect("read_range should return data");

	let text = range["text"].as_str().expect("text should be a string");
	assert!(text.contains("UNIQUE_TOKEN"));

	let search = workspace_query::dispatch_tool_call(
		"search",
		json!({
			"query": "UNIQUE_TOKEN",
			"mode": "literal",
			"limit": 10
		}),
		&services,
	)
	.expect("search should succeed")
	.expect("search should return data");

	let matches = search["matches"]
		.as_array()
		.expect("matches should be an array");
	assert!(
		matches.iter().any(|item| item["path"] == json!("large.txt")
			&& item["text"]
				.as_str()
				.is_some_and(|t| t.contains("UNIQUE_TOKEN"))),
		"expected search results to include large.txt"
	);
}
