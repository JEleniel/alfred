use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::configuration::HostPaths;
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

fn build_services(enable_mutations: bool) -> (ServiceContainer, TestDir) {
	build_services_with_workspace_memory(enable_mutations, true)
}

fn build_services_with_workspace_memory(
	enable_mutations: bool,
	workspace_memory_enabled: bool,
) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("memory-tools-tests");
	fs::create_dir_all(workspace.path.join(".alfred"))
		.expect("workspace config directory should be created");
	let config_json = json!({
		"memory": {
			"storage": {
				"workspace": {
					"enabled": workspace_memory_enabled,
				}
			}
		}
	});
	fs::write(
		workspace.path.join(".alfred").join("config.json"),
		config_json.to_string(),
	)
	.expect("workspace config should be written");

	let host = HostPaths {
		user_config_dir: Some(workspace.path.join("host-config")),
		user_data_dir: Some(workspace.path.join("host-data")),
		user_logs_dir: Some(workspace.path.join("host-logs")),
		system_logs_dir: None,
	};
	let mut config = AppConfig::load_with_host_paths(workspace.path.clone(), host)
		.expect("config should load with deterministic host paths");
	if enable_mutations {
		config.disabled_tools.retain(|name| {
			name != "memory"
				&& name != "memory_put"
				&& name != "memory_delete"
				&& name != "memory_get"
				&& name != "memory_list"
				&& name != "memory_search"
		});
	}

	let services = ServiceContainer::new(config).expect("service container should build");
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

fn dispatch_memory_call(
	operation: &str,
	args: serde_json::Value,
	services: &ServiceContainer,
) -> Result<serde_json::Value, AlfredError> {
	dispatch_tool_call(
		"memory",
		json!({
			"operation": operation,
			"args": args,
		}),
		services,
	)
}

fn create_fact(
	services: &ServiceContainer,
	scope: &str,
	subject: &str,
	fact: &str,
	category: &str,
	tags: Vec<&str>,
) -> String {
	let data = dispatch_memory_call(
		"create",
		json!({
			"scope": scope,
			"subject": subject,
			"fact": fact,
			"category": category,
			"reasoning": "because",
			"tags": tags,
		}),
		services,
	)
	.expect("memory create should succeed");

	data["result"]["memory"]["id"]
		.as_str()
		.expect("memory id should be present")
		.to_string()
}

#[test]
fn memory_create_validates_required_fields() {
	let (services, _workspace) = build_services(true);

	let result = dispatch_memory_call(
		"create",
		json!({
			"scope": "user",
			"subject": "subject",
			"fact": "   ",
			"category": "general",
			"reasoning": "because"
		}),
		&services,
	);

	match result {
		Err(AlfredError::InvalidArgument(message)) => {
			assert_eq!(message, "fact must not be empty");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn memory_create_issues_uuid_and_update_preserves_id() {
	let (services, _workspace) = build_services(true);

	let created = dispatch_memory_call(
		"create",
		json!({
			"scope": "user",
			"subject": "Rust",
			"fact": "Rust facts",
			"category": "coding_practices",
			"reasoning": "initial",
			"tags": ["rust", "alpha"]
		}),
		&services,
	)
	.expect("memory create should succeed");
	let id = created["result"]["memory"]["id"]
		.as_str()
		.expect("memory id should be present")
		.to_string();
	assert!(Uuid::parse_str(id.as_str()).is_ok());

	dispatch_memory_call(
		"update",
		json!({
			"id": id,
			"scope": "user",
			"subject": "Rust Updated",
			"fact": "Rust updated facts",
			"category": "coding_practices",
			"reasoning": "updated",
			"tags": ["alpha", "rust", "alpha"]
		}),
		&services,
	)
	.expect("memory update should succeed");

	let retrieved = dispatch_memory_call("retrieve", json!({"id": id}), &services)
		.expect("memory retrieve should succeed");
	let memory = &retrieved["result"]["memory"];
	assert_eq!(memory["id"], created["result"]["memory"]["id"]);
	assert_eq!(memory["scope"], json!("user"));
	assert_eq!(memory["subject"], json!("Rust Updated"));
	assert_eq!(memory["fact"], json!("Rust updated facts"));
	assert_eq!(memory["tags"], json!(["alpha", "rust"]));
	assert!(memory["created_at"].as_str().is_some());
	assert!(memory["updated_at"].as_str().is_some());
}

#[test]
fn memory_retrieve_returns_scope_for_workspace_fact() {
	let (services, _workspace) = build_services(true);
	let id = create_fact(
		&services,
		"workspace",
		"Workspace Subject",
		"Workspace Body",
		"general",
		vec!["workspace"],
	);

	let data = dispatch_memory_call("retrieve", json!({"id": id}), &services)
		.expect("memory retrieve should succeed");
	assert_eq!(data["result"]["memory"]["scope"], json!("workspace"));
}

#[test]
fn memory_update_returns_invalid_argument_when_scope_is_disabled() {
	let (services, _workspace) = build_services_with_workspace_memory(true, false);

	let result = dispatch_memory_call(
		"update",
		json!({
			"id": Uuid::new_v4().to_string(),
			"scope": "workspace",
			"subject": "subject",
			"fact": "fact",
			"category": "general",
			"reasoning": "because",
			"tags": ["tag"]
		}),
		&services,
	);

	match result {
		Err(AlfredError::InvalidArgument(message)) => {
			assert_eq!(
				message,
				"workspace memory storage is disabled by configuration"
			);
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn memory_delete_supports_dry_run() {
	let (services, _workspace) = build_services(true);
	let id = create_fact(&services, "user", "Subject", "Body", "general", vec!["ops"]);

	let dry_run = dispatch_memory_call("delete", json!({"id": id, "dry_run": true}), &services)
		.expect("memory delete dry-run should succeed");
	assert_eq!(dry_run["result"]["deleted"], json!(true));

	dispatch_memory_call("retrieve", json!({"id": id}), &services)
		.expect("fact should still exist after dry-run");

	let committed = dispatch_memory_call("delete", json!({"id": id, "dry_run": false}), &services)
		.expect("memory delete should succeed");
	assert_eq!(committed["result"]["deleted"], json!(true));

	let after = dispatch_memory_call("delete", json!({"id": id, "dry_run": false}), &services)
		.expect("memory delete should return false when missing");
	assert_eq!(after["result"]["deleted"], json!(false));
}

#[test]
fn memory_search_returns_ranked_filtered_matches_and_paginates() {
	let (services, _workspace) = build_services(true);
	create_fact(
		&services,
		"user",
		"Rust rust",
		"Rust book",
		"coding_practices",
		vec!["rust", "backend"],
	);
	create_fact(
		&services,
		"workspace",
		"Rust",
		"memory patterns",
		"coding_practices",
		vec!["backend", "rust"],
	);
	create_fact(
		&services,
		"user",
		"Python",
		"memory patterns",
		"coding_practices",
		vec!["python"],
	);

	let data = dispatch_memory_call(
		"search",
		json!({
			"query": "rust",
			"tags": ["backend"],
			"limit": 10
		}),
		&services,
	)
	.expect("memory search should succeed");

	let matches = data["result"]["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(matches.len(), 2);
	assert!(matches[0]["score"].as_u64().unwrap_or(0) >= matches[1]["score"].as_u64().unwrap_or(0));
	assert!(
		matches
			.iter()
			.any(|entry| entry["fact"]["scope"] == json!("user"))
	);
	assert!(
		matches
			.iter()
			.any(|entry| entry["fact"]["scope"] == json!("workspace"))
	);
	assert!(data["result"]["next_cursor"].is_null());

	let first_page = dispatch_memory_call(
		"search",
		json!({
			"query": "memory",
			"limit": 1
		}),
		&services,
	)
	.expect("first page should succeed");
	let first_matches = first_page["result"]["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(first_matches.len(), 1);
	let cursor = first_page["result"]["next_cursor"]
		.as_str()
		.expect("next cursor should exist")
		.to_string();

	let second_page = dispatch_memory_call(
		"search",
		json!({
			"query": "memory",
			"limit": 1,
			"cursor": cursor
		}),
		&services,
	)
	.expect("second page should succeed");
	let second_matches = second_page["result"]["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(second_matches.len(), 1);

	let tags_and = dispatch_memory_call(
		"search",
		json!({
			"tags": ["backend", "rust"],
			"tags_and": true
		}),
		&services,
	)
	.expect("tags_and search should succeed");
	let and_matches = tags_and["result"]["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(and_matches.len(), 2);
}

#[test]
fn memory_create_returns_io_error_when_store_root_is_not_directory() {
	let (services, _workspace) = build_services(true);
	let user_store = services
		.memory_store
		.user_store_path()
		.expect("user memory store should be enabled")
		.to_path_buf();
	let index_root = user_store.with_extension("tantivy");

	if index_root.is_dir() {
		fs::remove_dir_all(&index_root).expect("existing index root should be removed");
	} else if index_root.exists() {
		fs::remove_file(&index_root).expect("existing blocking file should be removed");
	}
	if let Some(parent) = index_root.parent() {
		fs::create_dir_all(parent).expect("index root parent should exist");
	}
	fs::write(&index_root, "blocking-file").expect("blocking file should be created");

	let result = dispatch_memory_call(
		"create",
		json!({
			"scope": "user",
			"subject": "Subject",
			"fact": "Fact",
			"category": "general",
			"reasoning": "because"
		}),
		&services,
	);

	match result {
		Err(AlfredError::IoError(message)) => {
			assert!(message.contains("failed to create memory store directory"));
		}
		other => panic!("unexpected result: {other:?}"),
	}
}
