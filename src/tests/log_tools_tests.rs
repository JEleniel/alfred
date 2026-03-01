use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
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

fn build_services() -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("log-search-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	let services = ServiceContainer::new(config).expect("service container should build");
	(services, workspace)
}

fn write_log_file(path: &Path, records: &[Value]) {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).expect("log parent directory should be created");
	}
	let mut content = String::new();
	for record in records {
		content.push_str(
			serde_json::to_string(record)
				.expect("log record should serialize")
				.as_str(),
		);
		content.push('\n');
	}
	fs::write(path, content).expect("log file should be written");
}

#[test]
fn logs_search_filters_and_preserves_file_order() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join(".alfred/logs/runtime.ndjson");
	write_log_file(
		log_path.as_path(),
		&[
			json!({
				"timestamp": "2026-02-23T10:00:00Z",
				"level": "INFO",
				"message": "build started",
				"source": "alfred::task::runner",
				"extra": {"task": "build"}
			}),
			json!({
				"timestamp": "2026-02-23T09:00:00Z",
				"level": "INFO",
				"message": "build completed",
				"source": "alfred::task::runner",
				"extra": {"task": "build"}
			}),
			json!({
				"timestamp": "2026-02-23T11:00:00Z",
				"level": "WARN",
				"message": "build warning",
				"source": "alfred::task::runner",
				"extra": {"task": "build"}
			}),
			json!({
				"timestamp": "2026-02-23T08:00:00Z",
				"level": "INFO",
				"message": "plan update",
				"source": "alfred::plan",
				"extra": {"id": "2"}
			}),
		],
	);

	let data = dispatch_tool_call(
		"logs",
		json!({
			"operation": "search",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"query": "build",
				"level": "info",
				"source_prefix": "alfred::task"
			}
		}),
		&services,
	)
	.expect("logs search should succeed");

	assert_eq!(data["operation"], json!("search"));
	let result = &data["result"];

	let matches = result["matches"]
		.as_array()
		.expect("logs search should return matches array");
	assert_eq!(matches.len(), 2);
	assert_eq!(matches[0]["timestamp"], json!("2026-02-23T10:00:00Z"));
	assert_eq!(matches[1]["timestamp"], json!("2026-02-23T09:00:00Z"));
	assert!(result["next_cursor"].is_null());
}

#[test]
fn logs_search_supports_stable_pagination() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join(".alfred/logs/runtime.ndjson");
	write_log_file(
		log_path.as_path(),
		&[
			json!({
				"timestamp": "2026-02-23T10:00:00Z",
				"level": "INFO",
				"message": "build one",
				"source": "alfred::task::runner",
				"extra": {}
			}),
			json!({
				"timestamp": "2026-02-23T10:01:00Z",
				"level": "INFO",
				"message": "build two",
				"source": "alfred::task::runner",
				"extra": {}
			}),
			json!({
				"timestamp": "2026-02-23T10:02:00Z",
				"level": "INFO",
				"message": "build three",
				"source": "alfred::task::runner",
				"extra": {}
			}),
		],
	);

	let first_page = dispatch_tool_call(
		"logs",
		json!({
			"operation": "search",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"query": "build",
				"limit": 2
			}
		}),
		&services,
	)
	.expect("first page should succeed");
	let first_result = &first_page["result"];

	let first_matches = first_result["matches"]
		.as_array()
		.expect("first page matches should exist");
	assert_eq!(first_matches.len(), 2);
	assert_eq!(first_result["next_cursor"], json!("2"));

	let second_page = dispatch_tool_call(
		"logs",
		json!({
			"operation": "search",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"query": "build",
				"cursor": "2",
				"limit": 2
			}
		}),
		&services,
	)
	.expect("second page should succeed");
	let second_result = &second_page["result"];

	let second_matches = second_result["matches"]
		.as_array()
		.expect("second page matches should exist");
	assert_eq!(second_matches.len(), 1);
	assert!(second_result["next_cursor"].is_null());
}

#[test]
fn logs_search_rejects_invalid_cursor() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join(".alfred/logs/runtime.ndjson");
	write_log_file(
		log_path.as_path(),
		&[json!({
			"timestamp": "2026-02-23T10:00:00Z",
			"level": "INFO",
			"message": "build one",
			"source": "alfred::task::runner",
			"extra": {}
		})],
	);

	let result = dispatch_tool_call(
		"logs",
		json!({
			"operation": "search",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"query": "build",
				"cursor": "oops"
			}
		}),
		&services,
	);

	match result {
		Err(AlfredError::InvalidArgument(message)) => {
			assert_eq!(message, "cursor is not a valid index: oops");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn logs_tail_returns_last_records_and_paginates() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join(".alfred/logs/runtime.ndjson");
	write_log_file(
		log_path.as_path(),
		&[
			json!({
				"timestamp": "2026-02-23T10:00:00Z",
				"level": "INFO",
				"message": "one",
				"source": "alfred::test",
				"extra": {}
			}),
			json!({
				"timestamp": "2026-02-23T10:01:00Z",
				"level": "INFO",
				"message": "two",
				"source": "alfred::test",
				"extra": {}
			}),
			json!({
				"timestamp": "2026-02-23T10:02:00Z",
				"level": "INFO",
				"message": "three",
				"source": "alfred::test",
				"extra": {}
			}),
		],
	);

	let first_page = dispatch_tool_call(
		"logs",
		json!({
			"operation": "tail",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"limit": 2
			}
		}),
		&services,
	)
	.expect("tail first page should succeed");

	assert_eq!(first_page["operation"], json!("tail"));
	let first_result = &first_page["result"];
	let first_records = first_result["records"]
		.as_array()
		.expect("tail should return records");
	assert_eq!(first_records.len(), 2);
	assert_eq!(first_records[0]["message"], json!("two"));
	assert_eq!(first_records[1]["message"], json!("three"));
	assert_eq!(first_result["next_cursor"], json!("2"));

	let second_page = dispatch_tool_call(
		"logs",
		json!({
			"operation": "tail",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"cursor": "2",
				"limit": 2
			}
		}),
		&services,
	)
	.expect("tail second page should succeed");

	let second_result = &second_page["result"];
	let second_records = second_result["records"]
		.as_array()
		.expect("tail should return records");
	assert_eq!(second_records.len(), 1);
	assert_eq!(second_records[0]["message"], json!("one"));
	assert!(second_result["next_cursor"].is_null());
}

#[test]
fn logs_follow_enforces_single_active_stream() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join(".alfred/logs/runtime.ndjson");
	write_log_file(
		log_path.as_path(),
		&[json!({
			"timestamp": "2026-02-23T10:00:00Z",
			"level": "INFO",
			"message": "one",
			"source": "alfred::test",
			"extra": {}
		})],
	);

	let first = dispatch_tool_call(
		"logs",
		json!({
			"operation": "follow",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"tail": 1
			}
		}),
		&services,
	)
	.expect("follow start should succeed");
	assert_eq!(first["operation"], json!("follow"));
	let records = first["result"]["records"]
		.as_array()
		.expect("follow should return records");
	assert_eq!(records.len(), 1);

	let second = dispatch_tool_call(
		"logs",
		json!({
			"operation": "follow",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"tail": 1
			}
		}),
		&services,
	);
	match second {
		Err(AlfredError::ConflictWithDetails { details, .. }) => {
			assert_eq!(details, Some(json!({"reason": "stream_active"})));
		}
		other => panic!("unexpected result: {other:?}"),
	}

	let stopped = dispatch_tool_call(
		"logs",
		json!({
			"operation": "follow",
			"args": {
				"stop": true
			}
		}),
		&services,
	)
	.expect("follow stop should succeed");
	assert_eq!(stopped["result"]["stopped"], json!(true));

	let stop_again = dispatch_tool_call(
		"logs",
		json!({
			"operation": "follow",
			"args": {
				"stop": true
			}
		}),
		&services,
	);
	match stop_again {
		Err(AlfredError::ConflictWithDetails { details, .. }) => {
			assert_eq!(details, Some(json!({"reason": "stream_not_active"})));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[cfg(unix)]
#[test]
fn log_search_rejects_symlink_escape_targets() {
	use std::os::unix::fs::symlink;

	let (services, workspace) = build_services();

	let outside = TestDir::new("log-search-symlink-escape-outside");
	let outside_log = outside.path.join("outside.ndjson");
	write_log_file(
		outside_log.as_path(),
		&[json!({
			"timestamp": "2026-02-23T10:00:00Z",
			"level": "INFO",
			"message": "outside",
			"source": "alfred::test",
			"extra": {}
		})],
	);

	let link_path = workspace.path.join(".alfred/logs/runtime.ndjson");
	if let Some(parent) = link_path.parent() {
		fs::create_dir_all(parent).expect("workspace log directory should exist");
	}
	symlink(&outside_log, &link_path).expect("symlink should be created");

	let result = dispatch_tool_call(
		"logs",
		json!({
			"operation": "search",
			"args": {
				"path": ".alfred/logs/runtime.ndjson",
				"query": "outside"
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
