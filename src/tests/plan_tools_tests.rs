use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::tools::{ToolCallResult, dispatch_tool_call as dispatch_tool_call_outcome};

const SAMPLE_PLAN: &str = "# Plan: Test Plan\n\n1. [ ] First task\n    - Priority: 1\n    - Cards: \"ART-001\", \"STR-001\"\n    - Description: First description\n    - Deliverables:\n        - First deliverable\n    - Status: planned\n\n2. [x] Second task\n    - Priority: 0\n    - Cards: \"ART-002\"\n    - Description: Second description\n    - Deliverables:\n        - Second deliverable\n    - Notes: Existing note\n    - Status: completed\n";

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
		tool != "plan_update" && tool != "plan_edit" && tool != "plan_add" && tool != "plan_delete"
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

fn write_plan(workspace_root: &std::path::Path, content: &str) {
	let plan_path = workspace_root.join("ProjectPlan.md");
	fs::write(plan_path, content).expect("plan fixture should be written");
}

#[test]
fn plan_get_returns_not_found_when_plan_file_is_missing() {
	let (services, _workspace) = build_services(false);

	let result = dispatch_tool_call("plan_get", json!({}), &services);

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

	let result = dispatch_tool_call("plan_get", json!({}), &services);

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

	let data =
		dispatch_tool_call("plan_get", json!({}), &services).expect("plan_get should succeed");

	let items = data["items"].as_array().expect("items should be an array");
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

	dispatch_tool_call(
		"plan_update",
		json!({"id": 1, "status": "in-progress"}),
		&services,
	)
	.expect("plan_update should succeed");

	let data = dispatch_tool_call("plan_get", json!({}), &services)
		.expect("plan_get should succeed after update");
	let items = data["items"].as_array().expect("items should be an array");
	assert_eq!(items[0]["status"], json!("in-progress"));
}

#[test]
fn plan_update_returns_not_found_for_missing_id() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let result = dispatch_tool_call(
		"plan_update",
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
fn plan_edit_replaces_existing_item() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	dispatch_tool_call(
		"plan_edit",
		json!({
			"id": 2,
			"title": "Second task edited",
			"priority": 2,
			"cards": ["ART-002", "STR-010"],
			"description": "Edited description",
			"deliverables": ["Edited deliverable"],
			"acceptance_criteria": "Edited acceptance",
			"notes": "Edited notes",
			"status": "cancelled"
		}),
		&services,
	)
	.expect("plan_edit should succeed");

	let data = dispatch_tool_call("plan_get", json!({}), &services)
		.expect("plan_get should succeed after edit");
	let items = data["items"].as_array().expect("items should be an array");
	assert_eq!(items[1]["id"], json!(2));
	assert_eq!(items[1]["title"], json!("Second task edited"));
	assert_eq!(items[1]["priority"], json!(2));
	assert_eq!(items[1]["status"], json!("cancelled"));
}

#[test]
fn plan_edit_rejects_invalid_priority() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let result = dispatch_tool_call(
		"plan_edit",
		json!({
			"id": 2,
			"title": "Second task edited",
			"priority": 8,
			"cards": ["ART-002"],
			"description": "Edited description",
			"deliverables": ["Edited deliverable"],
			"status": "planned"
		}),
		&services,
	);

	match result {
		Err(AlfredError::InvalidArgument(message)) => {
			assert_eq!(message, "priority must be between 0 and 3");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn plan_add_appends_item_with_next_sequential_id() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let data = dispatch_tool_call(
		"plan_add",
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
	.expect("plan_add should succeed");

	assert_eq!(data["id"], json!(3));

	let after = dispatch_tool_call("plan_get", json!({}), &services)
		.expect("plan_get should succeed after add");
	let items = after["items"].as_array().expect("items should be an array");
	assert_eq!(items.len(), 3);
	assert_eq!(items[2]["id"], json!(3));
	assert_eq!(items[2]["title"], json!("Third task"));
}

#[test]
fn plan_delete_removes_existing_item() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	dispatch_tool_call("plan_delete", json!({"id": 1}), &services)
		.expect("plan_delete should succeed");

	let data = dispatch_tool_call("plan_get", json!({}), &services)
		.expect("plan_get should succeed after delete");
	let items = data["items"].as_array().expect("items should be an array");
	assert_eq!(items.len(), 1);
	assert_eq!(items[0]["id"], json!(2));
}

#[test]
fn plan_delete_returns_not_found_for_missing_item_id() {
	let (services, workspace) = build_services(true);
	write_plan(workspace.path.as_path(), SAMPLE_PLAN);

	let result = dispatch_tool_call("plan_delete", json!({"id": 999}), &services);

	match result {
		Err(AlfredError::NotFound(message)) => {
			assert_eq!(message, "plan item id not found: 999");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}
