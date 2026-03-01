use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;
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
	let workspace = TestDir::new("fs-tool-tests");
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
fn fs_bulk_execute_sync_runs_copy_and_delete_operations() {
	let (services, workspace) = build_services(false);

	let result = dispatch_tool_call(
		"fs",
		json!({
			"operation": "bulk",
			"dry_run": false,
			"args": {
				"mode": "execute",
				"operations": [
					{
						"kind": "copy",
						"from": "alpha.txt",
						"to": "alpha-copy.txt",
						"overwrite": false,
						"create_parents": false
					},
					{
						"kind": "delete",
						"path": "nested/beta.md",
						"recursive": false
					}
				]
			}
		}),
		&services,
	)
	.expect("fs.bulk execute should succeed");

	assert_eq!(result["operation"], json!("bulk"));
	assert_eq!(result["result"]["summary"]["total"], json!(2));
	assert_eq!(result["result"]["summary"]["failed"], json!(0));
	assert!(matches!(
		result["result"]["state"].as_str(),
		Some("succeeded")
	));

	let copied = fs::read_to_string(workspace.path.join("alpha-copy.txt"))
		.expect("copied file should exist");
	assert_eq!(copied, "Hello from alpha\nSecond line\n");
	assert!(!workspace.path.join("nested").join("beta.md").exists());
}

#[test]
fn fs_bulk_execute_background_returns_pending_and_status_can_be_polled() {
	let (services, workspace) = build_services(false);

	let outcome = dispatch_tool_call_outcome(
		"fs",
		json!({
			"operation": "bulk",
			"dry_run": false,
			"args": {
				"mode": "execute",
				"run_in_background": true,
				"operations": [
					{
						"kind": "copy",
						"from": "alpha.txt",
						"to": "alpha-bg-copy.txt",
						"overwrite": false,
						"create_parents": false
					}
				]
			}
		}),
		&services,
	)
	.expect("fs.bulk background execute should dispatch");

	let (job_id, pending_data) = match outcome {
		ToolCallResult::Pending { job_id, data } => (job_id, data),
		ToolCallResult::Ok(other) => panic!("expected pending outcome, got ok: {other:?}"),
	};

	assert_eq!(pending_data["poll_with"], json!("fs"));
	assert_eq!(pending_data["operation_id"], json!(job_id));
	assert!(matches!(
		pending_data["state"].as_str(),
		Some("queued") | Some("running")
	));

	let mut last_state = String::new();
	for _ in 0..100 {
		let status = dispatch_tool_call(
			"fs",
			json!({
				"operation": "bulk",
				"args": {
					"mode": "status",
					"operation_id": job_id,
				}
			}),
			&services,
		)
		.expect("fs.bulk status should succeed");

		let state = status["result"]["state"]
			.as_str()
			.unwrap_or_default()
			.to_string();
		last_state = state.clone();
		if state != "queued" && state != "running" {
			break;
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}

	assert_eq!(last_state, "succeeded");
	let copied = fs::read_to_string(workspace.path.join("alpha-bg-copy.txt"))
		.expect("background copied file should exist");
	assert_eq!(copied, "Hello from alpha\nSecond line\n");
}

#[test]
fn fs_search_returns_normalized_stable_sorted_entries() {
	let (services, _workspace) = build_services(false);

	let data = dispatch_tool_call(
		"fs",
		json!({
			"operation": "search",
			"args": {
				"path": "./nested/../",
				"recursive": false
			}
		}),
		&services,
	)
	.expect("fs should succeed");

	assert_eq!(data["operation"], json!("search"));
	assert_eq!(data["result"]["files"], json!(["alpha.txt", "Zeta.txt"]));
	assert_eq!(data["result"]["directories"], json!(["nested"]));
	assert!(data["result"]["next_cursor"].is_null());
}

#[test]

fn fs_search_succeeds_when_index_is_disabled() {
	let (mut services, _workspace) = build_services(true);
	services.config.index_enabled = false;

	let data = dispatch_tool_call(
		"fs",
		json!({
			"operation": "search",
			"args": {}
		}),
		&services,
	)
	.expect("fs.search should succeed without the workspace index");

	assert_eq!(data["operation"], json!("search"));
	assert_eq!(data["result"]["files"], json!(["alpha.txt", "Zeta.txt"]));
	assert_eq!(data["result"]["directories"], json!(["nested"]));
}

#[test]
fn fs_read_range_collapses_dot_segments() {
	let (services, _workspace) = build_services(false);

	let data = dispatch_tool_call(
		"fs",
		json!({
			"operation": "read_range",
			"args": {
				"path": "./nested/../nested/./beta.md",
				"start_line": 1,
				"end_line": 1
			}
		}),
		&services,
	)
	.expect("fs.read_range should succeed");

	assert_eq!(data["operation"], json!("read_range"));
	assert_eq!(data["result"]["path"], json!("nested/beta.md"));
	assert_eq!(data["result"]["text"], json!("Hello from beta"));
}

#[test]
fn fs_read_range_reads_latest_bytes_after_file_changes() {
	let (services, workspace) = build_services(false);

	let absolute_path = workspace.path.join("alpha.txt");
	let original = dispatch_tool_call(
		"fs",
		json!({
			"operation": "read_range",
			"args": {
				"path": "alpha.txt",
				"start_line": 1,
				"end_line": 1
			}
		}),
		&services,
	)
	.expect("fs.read_range should succeed");
	assert_eq!(original["result"]["text"], json!("Hello from alpha"));

	fs::write(&absolute_path, "Hello from alpha (updated)\n")
		.expect("alpha.txt should be rewritten");

	let updated = dispatch_tool_call(
		"fs",
		json!({
			"operation": "read_range",
			"args": {
				"path": "alpha.txt",
				"start_line": 1,
				"end_line": 1
			}
		}),
		&services,
	)
	.expect("fs.read_range should succeed");
	assert_eq!(
		updated["result"]["text"],
		json!("Hello from alpha (updated)")
	);
}

#[test]
fn fs_read_range_rejects_boundary_escape_segments() {
	let (services, _workspace) = build_services(false);

	let result = dispatch_tool_call(
		"fs",
		json!({
			"operation": "read_range",
			"args": {
				"path": "../../etc/passwd",
				"start_line": 1,
				"end_line": 1
			}
		}),
		&services,
	);

	match result {
		Err(AlfredError::WorkspaceBoundaryViolation(_)) => {}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[cfg(unix)]
#[test]
fn fs_read_range_rejects_symlink_escape_targets() {
	use std::os::unix::fs::symlink;

	let (services, workspace) = build_services(false);
	let outside = TestDir::new("fs-tool-symlink-escape-outside");
	let outside_file = outside.path.join("outside.txt");
	fs::write(&outside_file, "secret\n").expect("outside file should write");

	let link_path = workspace.path.join("escape.txt");
	symlink(&outside_file, &link_path).expect("symlink should be created");

	let result = dispatch_tool_call(
		"fs",
		json!({
			"operation": "read_range",
			"args": {
				"path": "escape.txt",
				"start_line": 1,
				"end_line": 1
			}
		}),
		&services,
	);

	match result {
		Err(AlfredError::PermissionDenied(message)) => {
			assert_eq!(
				message,
				crate::workspace_boundary::SYMLINK_JUNCTION_ESCAPE_MESSAGE
			);
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn fs_stat_returns_normalized_file_metadata() {
	let (services, _workspace) = build_services(false);

	let data = dispatch_tool_call(
		"fs",
		json!({
			"operation": "stat",
			"args": {"path": "./nested/../alpha.txt"}
		}),
		&services,
	)
	.expect("fs.stat should succeed");

	assert_eq!(data["operation"], json!("stat"));
	assert_eq!(data["result"]["path"], json!("alpha.txt"));
	assert_eq!(data["result"]["kind"], json!("file"));
	assert!(
		data["result"]["size_bytes"]
			.as_u64()
			.is_some_and(|value| value > 0)
	);
	assert!(data["result"]["modified_at"].as_str().is_some());
}

#[test]
fn fs_diff_returns_deterministic_unified_diff() {
	let (services, _workspace) = build_services(false);

	let data = dispatch_tool_call(
		"fs",
		json!({
			"operation": "diff",
			"args": {
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
			}
		}),
		&services,
	)
	.expect("fs.diff should succeed");

	assert_eq!(
		data["result"]["diff"],
		json!("--- a\n+++ b\n@@ -1,1 +1,1 @@\n-Hello from alpha\n+Hello from beta\n")
	);
}

#[test]
fn fs_read_range_rejects_binary_non_text_input_deterministically() {
	let (services, workspace) = build_services(false);
	fs::write(
		workspace.path.join("binary.bin"),
		[0xFF_u8, 0xFE_u8, 0x00_u8, 0x01_u8],
	)
	.expect("binary fixture should be written");

	let result = dispatch_tool_call(
		"fs",
		json!({
			"operation": "read_range",
			"args": {
				"path": "binary.bin",
				"start_line": 1,
				"end_line": 1
			}
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
fn fs_read_range_supports_large_files_without_full_memory_load() {
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

	let range = dispatch_tool_call(
		"fs",
		json!({
			"operation": "read_range",
			"args": {
				"path": "large.txt",
				"start_line": 25_000,
				"end_line": 25_000
			}
		}),
		&services,
	)
	.expect("fs.read_range should succeed");

	let text = range["result"]["text"]
		.as_str()
		.expect("text should be a string");
	assert!(text.contains("UNIQUE_TOKEN"));

	let search = dispatch_tool_call(
		"search",
		json!({
			"query": "UNIQUE_TOKEN",
			"mode": "literal",
			"limit": 10
		}),
		&services,
	)
	.expect("search should succeed");

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

#[test]
fn fs_mutations_honor_dry_run_and_apply_when_enabled() {
	let workspace = TestDir::new("fs-mutation-tests");

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	let services = ServiceContainer::new(config).expect("service container should build");

	let created = dispatch_tool_call(
		"fs",
		json!({
			"operation": "create_file",
			"args": {"path": "notes.txt", "content": "hello"}
		}),
		&services,
	)
	.expect("fs.create_file should succeed");
	assert_eq!(created["result"]["bytes_written"], json!(5));
	assert!(!workspace.path.join("notes.txt").exists());

	let applied = dispatch_tool_call(
		"fs",
		json!({
			"operation": "create_file",
			"dry_run": false,
			"args": {"path": "notes.txt", "content": "hello"}
		}),
		&services,
	)
	.expect("fs.create_file should succeed");
	assert_eq!(applied["result"]["bytes_written"], json!(5));
	assert_eq!(
		fs::read_to_string(workspace.path.join("notes.txt")).unwrap(),
		"hello"
	);

	let appended_dry = dispatch_tool_call(
		"fs",
		json!({
			"operation": "append_file",
			"args": {"path": "notes.txt", "content": " world"}
		}),
		&services,
	)
	.expect("fs.append_file dry-run should succeed");
	assert_eq!(appended_dry["result"]["bytes_written"], json!(6));
	assert_eq!(
		fs::read_to_string(workspace.path.join("notes.txt")).unwrap(),
		"hello"
	);

	let appended = dispatch_tool_call(
		"fs",
		json!({
			"operation": "append_file",
			"dry_run": false,
			"args": {"path": "notes.txt", "content": " world"}
		}),
		&services,
	)
	.expect("fs.append_file should succeed");
	assert_eq!(appended["result"]["bytes_written"], json!(6));
	assert_eq!(
		fs::read_to_string(workspace.path.join("notes.txt")).unwrap(),
		"hello world"
	);

	let deleted_dry = dispatch_tool_call(
		"fs",
		json!({
			"operation": "delete_file",
			"args": {"path": "notes.txt"}
		}),
		&services,
	)
	.expect("fs.delete_file dry-run should succeed");
	assert_eq!(deleted_dry["result"]["deleted"], json!(true));
	assert!(workspace.path.join("notes.txt").exists());

	let deleted = dispatch_tool_call(
		"fs",
		json!({
			"operation": "delete_file",
			"dry_run": false,
			"args": {"path": "notes.txt"}
		}),
		&services,
	)
	.expect("fs.delete_file should succeed");
	assert_eq!(deleted["result"]["deleted"], json!(true));
	assert!(!workspace.path.join("notes.txt").exists());
}
