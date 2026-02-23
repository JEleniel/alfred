use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
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

fn build_services() -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("log-search-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let mut config = AppConfig::load_default().expect("default config should load");
	config.workspace_root = workspace.path.clone();
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
fn log_search_filters_and_preserves_file_order() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join("logs/runtime.ndjson");
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
		"log_search",
		json!({
			"path": "logs/runtime.ndjson",
			"query": "build",
			"level": "info",
			"source_prefix": "alfred::task"
		}),
		&services,
	)
	.expect("log_search should succeed");

	let matches = data["matches"]
		.as_array()
		.expect("log_search should return matches array");
	assert_eq!(matches.len(), 2);
	assert_eq!(matches[0]["timestamp"], json!("2026-02-23T10:00:00Z"));
	assert_eq!(matches[1]["timestamp"], json!("2026-02-23T09:00:00Z"));
	assert!(data["next_cursor"].is_null());
}

#[test]
fn log_search_supports_stable_pagination() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join("logs/runtime.ndjson");
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
		"log_search",
		json!({
			"path": "logs/runtime.ndjson",
			"query": "build",
			"limit": 2
		}),
		&services,
	)
	.expect("first page should succeed");

	let first_matches = first_page["matches"]
		.as_array()
		.expect("first page matches should exist");
	assert_eq!(first_matches.len(), 2);
	assert_eq!(first_page["next_cursor"], json!("2"));

	let second_page = dispatch_tool_call(
		"log_search",
		json!({
			"path": "logs/runtime.ndjson",
			"query": "build",
			"cursor": "2",
			"limit": 2
		}),
		&services,
	)
	.expect("second page should succeed");

	let second_matches = second_page["matches"]
		.as_array()
		.expect("second page matches should exist");
	assert_eq!(second_matches.len(), 1);
	assert!(second_page["next_cursor"].is_null());
}

#[test]
fn log_search_rejects_invalid_cursor() {
	let (services, workspace) = build_services();
	let log_path = workspace.path.join("logs/runtime.ndjson");
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
		"log_search",
		json!({
			"path": "logs/runtime.ndjson",
			"query": "build",
			"cursor": "oops"
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
