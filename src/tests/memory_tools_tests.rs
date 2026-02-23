use std::fs;
use std::path::PathBuf;

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

fn build_services(enable_mutations: bool) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("memory-tools-tests");
	let mut config = AppConfig::load_default().expect("default config should load");
	config.workspace_root = workspace.path.clone();
	config.user_config_path = workspace.path.join(".agents").join("user-config.json");
	config.workspace_config_path = workspace.path.join(".agents").join("workspace-config.json");
	if enable_mutations {
		config
			.disabled_tools
			.retain(|name| name != "memory_put" && name != "memory_delete");
	}

	let services = ServiceContainer::new(config).expect("service container should build");
	(services, workspace)
}

fn put_fact(
	services: &ServiceContainer,
	id: &str,
	subject: &str,
	fact: &str,
	category: &str,
	tags: Vec<&str>,
) -> Value {
	dispatch_tool_call(
		"memory_put",
		json!({
			"id": id,
			"subject": subject,
			"fact": fact,
			"citations": "source",
			"reason": "reason",
			"category": category,
			"tags": tags,
		}),
		services,
	)
	.expect("memory_put should succeed")
}

#[test]
fn memory_put_validates_required_fields() {
	let (services, _workspace) = build_services(true);

	let result = dispatch_tool_call(
		"memory_put",
		json!({
			"id": "fact-1",
			"subject": "subject",
			"fact": "   ",
			"citations": "source",
			"reason": "because",
			"category": "general",
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
fn memory_put_preserves_id_and_updates_fields() {
	let (services, _workspace) = build_services(true);

	let first = put_fact(
		&services,
		"fact-1",
		"Rust",
		"Rust facts",
		"coding_practices",
		vec!["rust", "alpha"],
	);
	assert_eq!(first["id"], json!("fact-1"));

	let second = put_fact(
		&services,
		"fact-1",
		"Rust Updated",
		"Rust updated facts",
		"coding_practices",
		vec!["alpha", "rust", "alpha"],
	);
	assert_eq!(second["id"], json!("fact-1"));

	let data = dispatch_tool_call("memory_get", json!({"id": "fact-1"}), &services)
		.expect("memory_get should succeed");
	let fact = &data["fact"];
	assert_eq!(fact["id"], json!("fact-1"));
	assert_eq!(fact["subject"], json!("Rust Updated"));
	assert_eq!(fact["fact"], json!("Rust updated facts"));
	assert_eq!(fact["tags"], json!(["alpha", "rust"]));
	assert!(fact["created_at"].as_str().is_some());
	assert!(fact["updated_at"].as_str().is_some());
}

#[test]
fn memory_get_returns_not_found_for_missing_id() {
	let (services, _workspace) = build_services(false);

	let result = dispatch_tool_call("memory_get", json!({"id": "missing"}), &services);
	match result {
		Err(AlfredError::NotFound(message)) => {
			assert_eq!(message, "memory fact not found: missing");
		}
		other => panic!("unexpected result: {other:?}"),
	}
}

#[test]
fn memory_delete_supports_dry_run() {
	let (services, _workspace) = build_services(true);
	put_fact(
		&services,
		"fact-1",
		"Subject",
		"Body",
		"general",
		vec!["ops"],
	);

	let dry_run = dispatch_tool_call(
		"memory_delete",
		json!({"id": "fact-1", "dry_run": true}),
		&services,
	)
	.expect("memory_delete dry-run should succeed");
	assert_eq!(dry_run["deleted"], json!(true));

	dispatch_tool_call("memory_get", json!({"id": "fact-1"}), &services)
		.expect("fact should still exist after dry-run");

	let committed = dispatch_tool_call("memory_delete", json!({"id": "fact-1"}), &services)
		.expect("memory_delete should succeed");
	assert_eq!(committed["deleted"], json!(true));

	let after = dispatch_tool_call("memory_delete", json!({"id": "fact-1"}), &services)
		.expect("memory_delete should return false when missing");
	assert_eq!(after["deleted"], json!(false));
}

#[test]
fn memory_list_supports_order_filter_and_pagination() {
	let (services, _workspace) = build_services(true);
	put_fact(&services, "a", "Zebra", "text", "general", vec!["ops", "rust"]);
	put_fact(&services, "b", "Alpha", "text", "general", vec!["ops"]);
	put_fact(&services, "c", "Middle", "text", "general", vec!["rust"]);

	let first_page = dispatch_tool_call(
		"memory_list",
		json!({
			"order": "subject",
			"limit": 2
		}),
		&services,
	)
	.expect("memory_list first page should succeed");
	let facts = first_page["facts"]
		.as_array()
		.expect("facts should be an array");
	assert_eq!(facts.len(), 2);
	assert_eq!(facts[0]["subject"], json!("Alpha"));
	assert_eq!(facts[1]["subject"], json!("Middle"));
	let cursor = first_page["next_cursor"]
		.as_str()
		.expect("next_cursor should exist")
		.to_string();

	let second_page = dispatch_tool_call(
		"memory_list",
		json!({
			"order": "subject",
			"limit": 2,
			"cursor": cursor
		}),
		&services,
	)
	.expect("memory_list second page should succeed");
	let second_facts = second_page["facts"]
		.as_array()
		.expect("facts should be an array");
	assert_eq!(second_facts.len(), 1);
	assert_eq!(second_facts[0]["subject"], json!("Zebra"));
	assert!(second_page["next_cursor"].is_null());

	let tag_and = dispatch_tool_call(
		"memory_list",
		json!({
			"order": "subject",
			"tags_any": ["ops", "rust"],
			"tags_and": true
		}),
		&services,
	)
	.expect("memory_list tags_and should succeed");
	let and_facts = tag_and["facts"].as_array().expect("facts should be an array");
	assert_eq!(and_facts.len(), 1);
	assert_eq!(and_facts[0]["id"], json!("a"));
}

#[test]
fn memory_search_returns_ranked_filtered_matches() {
	let (services, _workspace) = build_services(true);
	put_fact(
		&services,
		"one",
		"Rust rust",
		"Rust book",
		"coding_practices",
		vec!["rust", "backend"],
	);
	put_fact(
		&services,
		"two",
		"Rust",
		"memory patterns",
		"coding_practices",
		vec!["backend"],
	);
	put_fact(
		&services,
		"three",
		"Python",
		"memory patterns",
		"coding_practices",
		vec!["python"],
	);

	let data = dispatch_tool_call(
		"memory_search",
		json!({
			"query": "rust",
			"tags_any": ["backend"],
			"limit": 10
		}),
		&services,
	)
	.expect("memory_search should succeed");

	let matches = data["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(matches.len(), 2);
	assert_eq!(matches[0]["fact"]["id"], json!("one"));
	assert_eq!(matches[1]["fact"]["id"], json!("two"));
	assert!(matches[0]["score"].as_u64().unwrap_or(0) >= matches[1]["score"].as_u64().unwrap_or(0));
	assert!(data["next_cursor"].is_null());
}
