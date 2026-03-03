use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use alfred::configuration::{AppConfig, HostPaths};
use alfred::services::ServiceContainer;
use alfred::tools::{ToolCallResult, dispatch_tool_call};
use serde_json::{Value, json};
use uuid::Uuid;

pub const PLAN_FIXTURE: &str = "# Plan: Benchmark Plan\n\n1. [ ] First task\n    - Priority: 1\n    - Cards: \"ART-001\"\n    - Description: First description\n    - Deliverables:\n        - First deliverable\n    - Status: planned\n\n2. [x] Second task\n    - Priority: 2\n    - Cards: \"ART-002\"\n    - Description: Second description\n    - Deliverables:\n        - Second deliverable\n    - Status: completed\n";

pub const PATCH_TEXT: &str =
	"--- a/hello.txt\n+++ b/hello.txt\n@@ -1,2 +1,2 @@\n alpha\n-beta\n+gamma\n";

static BULK_COPY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TestDir {
	pub path: PathBuf,
}

impl TestDir {
	pub fn new(prefix: &str) -> Self {
		let path = std::env::current_dir()
			.expect("workspace current dir should resolve")
			.join("tmp")
			.join(format!("{prefix}-{}", Uuid::new_v4()));
		fs::create_dir_all(&path).expect("benchmark workspace should be created");
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
			copy_fixture_tree(source_path.as_path(), destination_path.as_path());
		} else {
			fs::copy(&source_path, &destination_path).expect("fixture file should copy");
		}
	}
}

pub fn build_services(prefix: &str, with_index: bool) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new(prefix);
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let host = HostPaths {
		user_config_dir: Some(workspace.path.join("host-config")),
		user_data_dir: Some(workspace.path.join("host-data")),
		user_logs_dir: Some(workspace.path.join("host-logs")),
		system_logs_dir: None,
	};
	let config = AppConfig::load_with_host_paths(workspace.path.clone(), host)
		.expect("benchmark config should load");
	let services = ServiceContainer::new(config).expect("service container should build");
	if with_index {
		services
			.indexer
			.rebuild()
			.expect("benchmark index should build");
	}

	(services, workspace)
}

pub fn dispatch_ok(name: &str, args: Value, services: &ServiceContainer) -> Value {
	match dispatch_tool_call(name, args, services) {
		Ok(ToolCallResult::Ok(data)) => data,
		Ok(ToolCallResult::Pending { .. }) => {
			panic!("unexpected pending response for sync benchmark path")
		}
		Err(error) => panic!("tool dispatch failed for {name}: {error}"),
	}
}

fn dispatch_outcome(name: &str, args: Value, services: &ServiceContainer) -> ToolCallResult {
	match dispatch_tool_call(name, args, services) {
		Ok(outcome) => outcome,
		Err(error) => panic!("tool dispatch failed for {name}: {error}"),
	}
}

pub fn write_log_fixture(services: &ServiceContainer) {
	let log_path = services.logs.log_path();
	if let Some(parent) = log_path.parent() {
		fs::create_dir_all(parent).expect("runtime log parent should be created");
	}

	let rows = [
		json!({
			"timestamp": "2026-03-01T10:00:00Z",
			"level": "INFO",
			"message": "build started",
			"source": "alfred::bench",
			"extra": {"task": "build"}
		}),
		json!({
			"timestamp": "2026-03-01T10:01:00Z",
			"level": "WARN",
			"message": "build warning",
			"source": "alfred::bench",
			"extra": {"task": "build"}
		}),
		json!({
			"timestamp": "2026-03-01T10:02:00Z",
			"level": "ERROR",
			"message": "build failed",
			"source": "alfred::bench",
			"extra": {"task": "build"}
		}),
	];

	let mut content = String::new();
	for row in rows {
		content.push_str(
			serde_json::to_string(&row)
				.expect("log entry should serialize")
				.as_str(),
		);
		content.push('\n');
	}
	fs::write(log_path, content).expect("runtime log fixture should write");
}

pub fn write_plan_fixture(services: &ServiceContainer) {
	let plan_path = services.plan_store.plan_path();
	if let Some(parent) = plan_path.parent() {
		fs::create_dir_all(parent).expect("plan parent should be created");
	}
	fs::write(plan_path, PLAN_FIXTURE).expect("plan fixture should write");
}

pub fn seed_patch_target(services: &ServiceContainer) {
	let patch_file = services.config.workspace_root.join("hello.txt");
	fs::write(patch_file, "alpha\nbeta\n").expect("patch target fixture should write");
}

pub fn seed_memory_fact(services: &ServiceContainer) -> String {
	let created = dispatch_ok(
		"memory",
		json!({
			"operation": "create",
			"args": {
				"scope": "user",
				"subject": "Benchmark subject",
				"category": "general",
				"fact": "Benchmark fact",
				"reasoning": "benchmark setup",
				"tags": ["bench", "seed"]
			}
		}),
		services,
	);

	created["result"]["memory"]["id"]
		.as_str()
		.expect("memory id should exist")
		.to_string()
}

pub fn start_bulk_background(services: &ServiceContainer) -> String {
	let copy_id = BULK_COPY_COUNTER.fetch_add(1, Ordering::Relaxed);
	let destination = format!("bulk-copy-{copy_id}.txt");
	let outcome = dispatch_outcome(
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
						"to": destination,
						"overwrite": true,
						"create_parents": false
					}
				]
			}
		}),
		services,
	);

	match outcome {
		ToolCallResult::Pending { job_id, .. } => job_id,
		ToolCallResult::Ok(data) => panic!("expected pending fs.bulk execute outcome, got: {data}"),
	}
}

pub fn runtime_log_relative_path(services: &ServiceContainer) -> String {
	let absolute = services.logs.log_path();
	match absolute.strip_prefix(services.config.workspace_root.as_path()) {
		Ok(relative) => relative.to_string_lossy().to_string(),
		Err(_) => absolute.to_string_lossy().to_string(),
	}
}
