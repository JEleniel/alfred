use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::protocol::{
	ToolMeta, ToolResponse, classify_wire_frame, extract_initialize_workspace_hints,
	handle_runtime_frame, handle_startup_frame,
};
use crate::services::ServiceContainer;

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

fn build_services_for_fixture(with_index: bool) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("protocol-tools-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let mut config = AppConfig::load_default().expect("default config should load");
	config.workspace_root = workspace.path.clone();
	let services = ServiceContainer::new(config).expect("service container should build");
	if with_index {
		services
			.indexer
			.rebuild()
			.expect("fixture index should build");
	}
	(services, workspace)
}

#[test]
fn classifies_initialize_request() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "initialize",
		"params": {
			"capabilities": {
				"roots": {"listChanged": true}
			}
		}
	});

	assert_eq!(
		classify_wire_frame(frame.to_string().as_str()),
		"initialize_request"
	);
}

#[test]
fn classifies_tools_list_request() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 9,
		"method": "tools/list",
		"params": {}
	});

	assert_eq!(
		classify_wire_frame(frame.to_string().as_str()),
		"tools_list_request"
	);
}

#[test]
fn returns_initialize_response_with_same_id() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": "abc-123",
		"method": "initialize",
		"params": {
			"clientInfo": {"name": "vscode"}
		}
	});

	let response = handle_startup_frame(frame.to_string().as_str())
		.expect("initialize frame should parse")
		.expect("initialize request should produce response");

	let response_json: Value =
		serde_json::from_str(&response).expect("response should be valid JSON");
	assert_eq!(response_json["id"], json!("abc-123"));
	assert_eq!(
		response_json["result"]["serverInfo"]["name"],
		env!("CARGO_PKG_NAME")
	);
	assert!(response_json["result"]["protocolVersion"].is_string());
}

#[test]
fn ignores_initialized_notification() {
	let frame = json!({
		"jsonrpc": "2.0",
		"method": "notifications/initialized",
		"params": {}
	});

	let response = handle_startup_frame(frame.to_string().as_str())
		.expect("initialized notification should parse");
	assert!(response.is_none());
}

#[test]
fn returns_tools_list_with_workspace_tools() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 7,
		"method": "tools/list",
		"params": {}
	});

	let response = handle_startup_frame(frame.to_string().as_str())
		.expect("request frame should parse")
		.expect("request with id should produce response");

	let response_json: Value =
		serde_json::from_str(&response).expect("response should be valid JSON");
	assert_eq!(response_json["id"], json!(7));
	let tools = response_json["result"]["tools"]
		.as_array()
		.expect("tools/list should return tools array");
	assert!(tools.iter().any(|tool| tool["name"] == "workspace_dir"));
}

#[test]
fn extracts_workspace_hints_from_initialize_payload() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": "init-1",
		"method": "initialize",
		"params": {
			"rootUri": "file:///Users/jeleniel/repos/alfred",
			"workspaceFolders": [
				{"uri": "file:///Users/jeleniel/repos/alfred", "name": "alfred"}
			]
		}
	});

	let hints = extract_initialize_workspace_hints(frame.to_string().as_str())
		.expect("hint extraction should parse")
		.expect("initialize payload should produce hints");

	assert_eq!(
		hints["rootUri"],
		json!("file:///Users/jeleniel/repos/alfred")
	);
	assert_eq!(hints["workspaceFolders"][0]["name"], json!("alfred"));
}

#[test]
fn no_workspace_hints_for_non_initialize_frame() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 3,
		"method": "tools/list",
		"params": {}
	});

	let hints = extract_initialize_workspace_hints(frame.to_string().as_str())
		.expect("hint extraction should parse");
	assert!(hints.is_none());
}

#[test]
fn returns_method_not_implemented_for_unknown_request() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 42,
		"method": "unknown/method",
		"params": {}
	});

	let response = handle_startup_frame(frame.to_string().as_str())
		.expect("request frame should parse")
		.expect("request with id should produce response");

	let response_json: Value =
		serde_json::from_str(&response).expect("response should be valid JSON");
	assert_eq!(response_json["id"], json!(42));
	assert_eq!(response_json["error"]["code"], json!(-32601));
}

#[test]
fn runtime_tools_call_workspace_dir_returns_ok_envelope() {
	let (services, _workspace) = build_services_for_fixture(true);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 100,
		"method": "tools/call",
		"params": {
			"name": "workspace_dir",
			"arguments": {}
		}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("tools/call should return response");
	let payload: Value = serde_json::from_str(&response).expect("response should be valid JSON");

	assert_eq!(payload["result"]["isError"], json!(false));
	assert_eq!(payload["result"]["content"][0]["type"], json!("text"));
	assert_eq!(
		payload["result"]["structuredContent"]["status"],
		json!("ok")
	);
	assert!(payload["result"]["structuredContent"]["data"]["root"].is_string());
}

#[test]
fn runtime_tools_call_grep_returns_index_matches() {
	let (services, _workspace) = build_services_for_fixture(true);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 101,
		"method": "tools/call",
		"params": {
			"name": "grep",
			"arguments": {
				"query": "hello",
				"case_sensitive": false
			}
		}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("tools/call should return response");
	let payload: Value = serde_json::from_str(&response).expect("response should be valid JSON");

	assert_eq!(payload["result"]["isError"], json!(false));
	assert_eq!(
		payload["result"]["structuredContent"]["status"],
		json!("ok")
	);
	let matches = payload["result"]["structuredContent"]["data"]["matches"]
		.as_array()
		.expect("grep response should include matches");
	assert_eq!(matches.len(), 2);
}

#[test]
fn runtime_tools_call_unknown_tool_returns_error_envelope_with_taxonomy_kind() {
	let (services, _workspace) = build_services_for_fixture(true);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 102,
		"method": "tools/call",
		"params": {
			"name": "does_not_exist",
			"arguments": {}
		}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("tools/call should return response");
	let payload: Value = serde_json::from_str(&response).expect("response should be valid JSON");

	assert_eq!(payload["result"]["isError"], json!(true));
	assert_eq!(
		payload["result"]["structuredContent"]["status"],
		json!("error")
	);
	assert_eq!(
		payload["result"]["structuredContent"]["error"]["kind"],
		json!("invalid_argument")
	);
	assert_eq!(
		payload["result"]["structuredContent"]["error"]["retryable"],
		json!(false)
	);
	let error_object = payload["result"]["structuredContent"]["error"]
		.as_object()
		.expect("error payload should be an object");
	assert!(!error_object.contains_key("details"));
}

#[test]
fn runtime_tools_call_grep_when_index_not_ready_has_reason_details() {
	let (services, _workspace) = build_services_for_fixture(false);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 103,
		"method": "tools/call",
		"params": {
			"name": "grep",
			"arguments": {
				"query": "hello"
			}
		}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("tools/call should return response");
	let payload: Value = serde_json::from_str(&response).expect("response should be valid JSON");

	assert_eq!(payload["result"]["isError"], json!(true));
	assert_eq!(
		payload["result"]["structuredContent"]["status"],
		json!("error")
	);
	assert_eq!(
		payload["result"]["structuredContent"]["error"]["kind"],
		json!("tool_unavailable")
	);
	assert_eq!(
		payload["result"]["structuredContent"]["error"]["details"]["reason"],
		json!("index_not_ready")
	);
}

#[test]
fn pending_envelope_includes_transport_equivalent_metadata() {
	let pending = ToolResponse::pending(
		"job-1".to_string(),
		json!({"queued": true}),
		ToolMeta {
			tool: "task_run".to_string(),
			schema_version: "1.0.0".to_string(),
			duration_ms: Some(1),
			warnings: Vec::new(),
			transport_equivalent: None,
		},
	);

	let payload = serde_json::to_value(pending).expect("pending envelope should serialize");
	assert_eq!(payload["status"], json!("pending"));
	assert_eq!(payload["job_id"], json!("job-1"));
	assert_eq!(payload["data"]["queued"], json!(true));
	assert_eq!(
		payload["meta"]["transport_equivalent"]["http_status"],
		json!(202)
	);
	assert!(payload["meta"].get("warnings").is_none());
}
