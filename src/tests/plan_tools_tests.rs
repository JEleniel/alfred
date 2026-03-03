use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::tools::{ToolCallResult, dispatch_tool_call as dispatch_tool_call_outcome};

const SAMPLE_PLAN: &str = "# Plan: Test Plan\n\n1. [ ] First task\n    - Priority: 1\n    - Cards: \"ART-001\", \"STR-001\"\n    - Description: First description\n    - Deliverables:\n        - First deliverable\n    - Status: planned\n\n2. [x] Second task\n    - Priority: 0\n    - Cards: \"ART-002\"\n    - Description: Second description\n    - Deliverables:\n        - Second deliverable\n    - Notes: Existing note\n    - Status: completed\n";
const SPARSE_PLAN: &str = "# Plan: Sparse\n\n1. [ ] Keep formatting\n    - Priority: 1\n    - Cards: \"ART-100\"\n    - Description: Preserve blank lines\n\n    - Deliverables:\n        - Keep this line\n    - Status: planned\n";

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

fn build_services(enable_plan_mutations: bool) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("plan-tools-tests");
	let mut config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	if enable_plan_mutations {
		enable_plan_mutation_tools(&mut config);
	}
	let services = ServiceContainer::new(config).expect("service container should build");
	(services, workspace)
}

fn enable_plan_mutation_tools(config: &mut AppConfig) {
	config.disabled_tools.retain(|tool| {
		tool != "plan"
			&& tool != "plan_update"
			&& tool != "plan_edit"
			&& tool != "plan_add"
			&& tool != "plan_delete"
	});
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

fn dispatch_plan_call(
	operation: &str,
	args: serde_json::Value,
	services: &ServiceContainer,
) -> Result<serde_json::Value, AlfredError> {
	dispatch_tool_call(
		"plan",
		json!({
			"operation": operation,
			"args": args,
		}),
		services,
	)
}

fn write_plan(workspace_root: &std::path::Path, content: &str) {
	let plan_path = workspace_root.join("ProjectPlan.md");
	fs::write(plan_path, content).expect("plan fixture should be written");
}

#[test]
fn plan_get_returns_not_found_when_plan_file_is_missing() {
	let (services, _workspace) = build_services(false);

	let result = dispatch_plan_call("get", json!({}), &services);

	match result {
		Err(AlfredError::NotFound(message)) => {
			assert!(message.contains("plan file not found"));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn plan_get_returns_parse_error_for_invalid_plan_format() {
	let (services, workspace) = build_services(false);
	write_plan(
		workspace.path.as_path(),
		"# Plan: Broken\n\n1. [ ] Missing fields\n    - Status: planned\n",
	);

	let result = dispatch_plan_call("get", json!({}), &services);

	match result {
		Err(AlfredError::InvalidArgument(message)) => {
			assert!(message.contains("failed to parse plan file"));
			assert!(message.contains("missing priority"));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn plan_get_returns_items_from_valid_plan_file() {
	let (services, workspace) = build_services(false);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let data = dispatch_plan_call("get", json!({}), &services).expect("plan get should succeed");

	assert_eq!(data["operation"], json!("get"));
	let items = data["result"]["items"]
		.as_array()
		.expect("items should be an array");
	assert_eq!(items.len(), 2);
	assert_eq!(items[0]["id"], json!(1));
	assert_eq!(items[0]["title"], json!("First task"));
	assert_eq!(items[0]["status"], json!("planned"));
	assert_eq!(items[1]["id"], json!(2));
	assert_eq!(items[1]["status"], json!("completed"));
}

#[test]
fn plan_update_changes_status_for_existing_item() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	dispatch_plan_call(
		"update_status",
		json!({"id": 1, "status": "in-progress"}),
		&services,
	)
	.expect("plan update_status should succeed");

	let data = dispatch_plan_call("get", json!({}), &services)
		.expect("plan get should succeed after update");
	let items = data["result"]["items"]
		.as_array()
		.expect("items should be an array");
	assert_eq!(items[0]["status"], json!("in-progress"));
}

#[test]
fn plan_update_status_preserves_surrounding_formatting() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SPARSE_PLAN);

	dispatch_plan_call(
		"update_status",
		json!({"id": 1, "status": "completed"}),
		&services,
	)
	.expect("plan update_status should succeed");

	let raw = fs::read_to_string(workspace.path.join("ProjectPlan.md"))
		.expect("plan file should be readable");
	assert!(raw.contains("1. [x] Keep formatting"));
	assert!(raw.contains("    - Status: completed"));
	assert!(raw.contains("Description: Preserve blank lines\n\n    - Deliverables:"));
}

#[test]
fn plan_update_returns_not_found_for_missing_id() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let result = dispatch_plan_call(
		"update_status",
		json!({"id": 999, "status": "completed"}),
		&services,
	);

	match result {
		Err(AlfredError::NotFound(message)) => {
			assert_eq!(message, "plan item id not found: 999");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn plan_update_preserves_original_content_on_partial_write_failure() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let _fail_guard = crate::services::workspace_files::fail_writes_after_bytes(12);
	let result = dispatch_plan_call(
		"update_status",
		json!({"id": 1, "status": "in-progress"}),
		&services,
	);

	match result {
		Err(AlfredError::IoError(message)) => {
			assert!(message.contains("simulated partial write failure"));
		}
		other => panic!("unexpected result: {other:?}"),
	}

	let raw = fs::read_to_string(workspace.path.join("ProjectPlan.md"))
		.expect("plan file should remain readable after rollback");
	assert_eq!(raw, SAMPLE_PLAN);
}

#[test]
fn plan_update_surfaces_write_and_rollback_failures() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let _write_fail_guard = crate::services::workspace_files::fail_writes_after_bytes(12);
	let _rollback_fail_guard = crate::services::workspace_files::fail_rollbacks();
	let result = dispatch_plan_call(
		"update_status",
		json!({"id": 1, "status": "in-progress"}),
		&services,
	);

	match result {
		Err(AlfredError::IoError(message)) => {
			assert!(message.contains("simulated partial write failure"));
			assert!(message.contains("simulated rollback failure"));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn plan_add_appends_item_with_next_sequential_id() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let data = dispatch_plan_call(
		"add",
		json!({
			"id": 123,
			"title": "Third task",
			"priority": 3,
			"cards": ["ART-003"],
			"description": "Third description",
			"deliverables": ["Third deliverable"],
			"status": "planned"
		}),
		&services,
	)
	.expect("plan add should succeed");

	assert_eq!(data["operation"], json!("add"));
	assert_eq!(data["result"]["id"], json!(3));

	let after =
		dispatch_plan_call("get", json!({}), &services).expect("plan get should succeed after add");
	let items = after["result"]["items"]
		.as_array()
		.expect("items should be an array");
	assert_eq!(items.len(), 3);
	assert_eq!(items[2]["id"], json!(3));
	assert_eq!(items[2]["title"], json!("Third task"));
}

#[test]
fn plan_delete_removes_existing_item() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	dispatch_plan_call("delete", json!({"id": 1}), &services).expect("plan delete should succeed");

	let data = dispatch_plan_call("get", json!({}), &services)
		.expect("plan get should succeed after delete");
	let items = data["result"]["items"]
		.as_array()
		.expect("items should be an array");
	assert_eq!(items.len(), 1);
	assert_eq!(items[0]["id"], json!(2));
}

#[test]
fn plan_delete_returns_not_found_for_missing_item_id() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let result = dispatch_plan_call("delete", json!({"id": 999}), &services);

	match result {
		Err(AlfredError::NotFound(message)) => {
			assert_eq!(message, "plan item id not found: 999");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}
