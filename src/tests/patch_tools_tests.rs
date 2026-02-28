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
fn patch_tool_applies_and_reverts_latest_batch() {
	let workspace = TestDir::new("patch-tool-tests");
	let services = build_services(workspace.path.clone());

	let file_path = workspace.path.join("hello.txt");
	fs::write(&file_path, "alpha\nbeta\n").expect("fixture file should write");

	let patch_text = "--- a/hello.txt\n+++ b/hello.txt\n@@ -1,2 +1,2 @@\n alpha\n-beta\n+gamma\n";

	let apply_args = json!({
		"operation": "apply",
		"dry_run": false,
		"patches": [
			{
				"path": "hello.txt",
				"patch": patch_text,
			}
		]
	});

	let apply_result =
		dispatch_tool_call("patch", apply_args, &services).expect("apply should succeed");
	assert_eq!(apply_result["operation"], "apply");
	assert_eq!(apply_result["dry_run"], false);
	assert!(
		apply_result["files"][0]["patched"]
			.as_bool()
			.unwrap_or(false)
	);

	let updated = fs::read_to_string(&file_path).expect("patched file should read");
	assert_eq!(updated, "alpha\ngamma\n");

	let revert_args = json!({
		"operation": "revert",
		"dry_run": false
	});

	let revert_result =
		dispatch_tool_call("patch", revert_args, &services).expect("revert should succeed");
	assert_eq!(revert_result["operation"], "revert");
	assert_eq!(revert_result["dry_run"], false);
	assert!(
		revert_result["files"][0]["reverted"]
			.as_bool()
			.unwrap_or(false)
	);

	let restored = fs::read_to_string(&file_path).expect("restored file should read");
	assert_eq!(restored, "alpha\nbeta\n");
}

#[test]
fn patch_tool_revert_requires_latest_after_hash() {
	let workspace = TestDir::new("patch-tool-revert-sha-tests");
	let services = build_services(workspace.path.clone());

	let file_path = workspace.path.join("hello.txt");
	fs::write(&file_path, "alpha\nbeta\n").expect("fixture file should write");

	let patch_text = "--- a/hello.txt\n+++ b/hello.txt\n@@ -1,2 +1,2 @@\n alpha\n-beta\n+gamma\n";
	let apply_args = json!({
		"operation": "apply",
		"dry_run": false,
		"patches": [
			{
				"path": "hello.txt",
				"patch": patch_text,
			}
		]
	});
	dispatch_tool_call("patch", apply_args, &services).expect("apply should succeed");

	fs::write(&file_path, "alpha\nmanual-change\n").expect("manual update should write");

	let revert_args = json!({
		"operation": "revert",
		"dry_run": false
	});
	let revert_result = dispatch_tool_call("patch", revert_args, &services)
		.expect("revert call should succeed with conflict payload");

	assert_eq!(revert_result["operation"], "revert");
	assert_eq!(revert_result["files"][0]["reverted"], false);
	let conflicts = revert_result["files"][0]["conflicts"]
		.as_array()
		.expect("revert result should include conflicts array");
	assert!(
		conflicts
			.iter()
			.any(|conflict| conflict["kind"] == "content_mismatch")
	);

	let unchanged = fs::read_to_string(&file_path).expect("file should read");
	assert_eq!(unchanged, "alpha\nmanual-change\n");
}

#[test]
fn patch_tool_emits_duplicate_content_warning() {
	let workspace = TestDir::new("patch-tool-duplicate-warning-tests");
	let services = build_services(workspace.path.clone());

	let file_path = workspace.path.join("dup.txt");
	let long_line = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
	fs::write(&file_path, format!("{long_line}\n")).expect("fixture file should write");

	let patch_text =
		format!("--- a/dup.txt\n+++ b/dup.txt\n@@ -1 +1,2 @@\n {long_line}\n+{long_line}\n");
	let apply_args = json!({
		"operation": "apply",
		"dry_run": true,
		"patches": [
			{
				"path": "dup.txt",
				"patch": patch_text,
			}
		]
	});

	let apply_result =
		dispatch_tool_call("patch", apply_args, &services).expect("apply should succeed");
	let warnings = apply_result["files"][0]["warnings"]
		.as_array()
		.expect("apply result should include warnings array");
	assert!(
		warnings
			.iter()
			.any(|warning| warning["kind"] == "duplicate_content_risk")
	);
}
