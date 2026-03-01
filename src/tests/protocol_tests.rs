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

#[cfg(all(unix, not(target_os = "macos")))]
use std::ffi::OsString;

#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::ffi::OsStringExt;

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

#[cfg(all(unix, not(target_os = "macos")))]
fn write_non_utf8_fixture_file(workspace_root: &Path) -> String {
	let mut bytes = b"bad-".to_vec();
	bytes.push(0xFF);
	bytes.extend_from_slice(b".txt");
	let filename = OsString::from_vec(bytes);
	let path = workspace_root.join(filename);
	fs::write(&path, "hello\n").expect("non-utf8 fixture file should write");
	"bad-\\xFF.txt".to_string()
}

fn build_services_for_fixture_with_disabled_tools(
	with_index: bool,
	disabled_tools: Vec<String>,
) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("protocol-tools-policy-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let mut config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	config.disabled_tools.extend(disabled_tools);
	config.disabled_tools.sort_unstable();
	config.disabled_tools.dedup();
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
fn classifies_prompts_list_request() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 10,
		"method": "prompts/list",
		"params": {}
	});

	assert_eq!(
		classify_wire_frame(frame.to_string().as_str()),
		"prompts_list_request"
	);
}

#[test]
fn classifies_prompts_get_request() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 11,
		"method": "prompts/get",
		"params": {
			"name": "alfred_agent",
			"arguments": {}
		}
	});

	assert_eq!(
		classify_wire_frame(frame.to_string().as_str()),
		"prompts_get_request"
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
fn returns_prompts_list_with_alfred_agent_prompt() {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 12,
		"method": "prompts/list",
		"params": {}
	});

	let response = handle_startup_frame(frame.to_string().as_str())
		.expect("request frame should parse")
		.expect("request with id should produce response");
	let response_json: Value =
		serde_json::from_str(&response).expect("response should be valid JSON");
	assert_eq!(response_json["id"], json!(12));

	let prompts = response_json["result"]["prompts"]
		.as_array()
		.expect("prompts/list should return prompts array");
	assert!(
		prompts
			.iter()
			.any(|prompt| prompt["name"] == "alfred_agent")
	);
}

#[test]
fn runtime_prompts_get_includes_workspace_root() {
	let (services, workspace) = build_services_for_fixture(true);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 13,
		"method": "prompts/get",
		"params": {
			"name": "alfred_agent",
			"arguments": {}
		}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("request with id should produce response");
	let response_json: Value =
		serde_json::from_str(&response).expect("response should be valid JSON");
	assert_eq!(response_json["id"], json!(13));
	let text = response_json["result"]["messages"][0]["content"][0]["text"]
		.as_str()
		.unwrap_or_default();
	assert!(text.contains(workspace.path.to_string_lossy().as_ref()));
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
fn runtime_tools_call_search_returns_index_matches() {
	let (services, _workspace) = build_services_for_fixture(true);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 101,
		"method": "tools/call",
		"params": {
			"name": "search",
			"arguments": {
				"query": "hello",
				"case_sensitive": false,
				"mode": "literal"
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
		.expect("search response should include matches");
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
fn runtime_tools_call_search_when_index_not_ready_has_reason_details() {
	let (services, _workspace) = build_services_for_fixture(false);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 103,
		"method": "tools/call",
		"params": {
			"name": "search",
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

#[test]
fn runtime_tools_call_fs_bulk_background_returns_pending_envelope() {
	let (services, _workspace) = build_services_for_fixture(false);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 999,
		"method": "tools/call",
		"params": {
			"name": "fs",
			"arguments": {
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
		json!("pending")
	);
	assert_eq!(
		payload["result"]["structuredContent"]["meta"]["transport_equivalent"]["http_status"],
		json!(202)
	);
	assert_eq!(
		payload["result"]["structuredContent"]["data"]["poll_with"],
		json!("fs")
	);

	let operation_id = payload["result"]["structuredContent"]["data"]["operation_id"]
		.as_str()
		.expect("pending data.operation_id should be a string")
		.to_string();
	assert_eq!(
		payload["result"]["structuredContent"]["job_id"],
		json!(operation_id)
	);
	assert!(matches!(
		payload["result"]["structuredContent"]["data"]["state"].as_str(),
		Some("queued") | Some("running")
	));

	for _ in 0..100 {
		let status = services
			.jobs
			.fs_bulk_status(operation_id.as_str())
			.expect("bulk status should be available");
		if !matches!(
			status.state,
			crate::services::job_manager::FsBulkState::Queued
				| crate::services::job_manager::FsBulkState::Running
		) {
			break;
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
}

#[test]
fn runtime_tools_list_omits_policy_disabled_tools() {
	let (services, _workspace) =
		build_services_for_fixture_with_disabled_tools(true, vec!["search".to_string()]);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 104,
		"method": "tools/list",
		"params": {}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("tools/list should return response");
	let payload: Value = serde_json::from_str(&response).expect("response should be valid JSON");
	let names = payload["result"]["tools"]
		.as_array()
		.expect("tools/list should return a tools array")
		.iter()
		.filter_map(|entry| entry.get("name").and_then(Value::as_str))
		.collect::<Vec<_>>();
	let mut sorted_names = names.clone();
	sorted_names.sort_unstable();

	assert!(!names.contains(&"search"));
	assert!(names.contains(&"workspace_dir"));
	assert!(!names.contains(&"memory_put"));
	assert_eq!(names, sorted_names);
}

#[test]
#[cfg(all(unix, not(target_os = "macos")))]
fn runtime_tools_call_emits_path_encoded_warning_and_round_trips_encoded_paths() {
	let (services, workspace) = build_services_for_fixture(false);
	let encoded_name = write_non_utf8_fixture_file(workspace.path.as_path());
	services
		.indexer
		.rebuild()
		.expect("fixture index should build");

	let ls_frame = json!({
		"jsonrpc": "2.0",
		"id": 200,
		"method": "tools/call",
		"params": {"name": "ls", "arguments": {}}
	});
	let ls_response = handle_runtime_frame(ls_frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("tools/call should return response");
	let ls_payload: Value =
		serde_json::from_str(&ls_response).expect("response should be valid JSON");

	let files = ls_payload["result"]["structuredContent"]["data"]["files"]
		.as_array()
		.expect("ls should return files array")
		.iter()
		.filter_map(|entry| entry.as_str())
		.collect::<Vec<_>>();
	assert!(
		files.contains(&encoded_name.as_str()),
		"ls should include encoded file name"
	);

	let warnings = ls_payload["result"]["structuredContent"]["meta"]["warnings"]
		.as_array()
		.expect("ls should include warnings when encoded paths are present");
	assert!(
		warnings
			.iter()
			.any(|warning| warning["kind"] == json!("path_encoded"))
	);

	let read_frame = json!({
		"jsonrpc": "2.0",
		"id": 201,
		"method": "tools/call",
		"params": {
			"name": "read_range",
			"arguments": {"path": encoded_name, "start_line": 1, "end_line": 1}
		}
	});
	let read_response = handle_runtime_frame(read_frame.to_string().as_str(), &services)
		.expect("runtime frame should parse")
		.expect("tools/call should return response");
	let read_payload: Value =
		serde_json::from_str(&read_response).expect("response should be valid JSON");

	assert_eq!(
		read_payload["result"]["structuredContent"]["status"],
		json!("ok")
	);
	assert_eq!(
		read_payload["result"]["structuredContent"]["data"]["text"],
		json!("hello")
	);
}

#[test]
fn runtime_tools_call_capabilities_returns_stable_tool_metadata() {
	let (services, _workspace) = build_services_for_fixture(true);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 106,
		"method": "tools/call",
		"params": {
			"name": "capabilities",
			"arguments": {}
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

	let tools = payload["result"]["structuredContent"]["data"]["tools"]
		.as_array()
		.expect("capabilities should return a tools array");
	let names = tools
		.iter()
		.filter_map(|tool| tool.get("name").and_then(Value::as_str))
		.collect::<Vec<_>>();
	let mut sorted_names = names.clone();
	sorted_names.sort_unstable();

	assert_eq!(names, sorted_names);
	assert!(names.contains(&"capabilities"));
	assert!(names.contains(&"workspace_dir"));
	assert!(!names.contains(&"file_read_bytes"));
	assert!(!names.contains(&"memory_put"));

	for tool in tools {
		let name = tool
			.get("name")
			.and_then(Value::as_str)
			.expect("tool should have name");

		assert_eq!(tool["version"], json!(env!("CARGO_PKG_VERSION")));
		assert_eq!(tool["schema_version"], json!(env!("CARGO_PKG_VERSION")));
		if name == "logs" {
			assert_eq!(tool["execution_modes"], json!(["sync", "stream"]));
		} else if name == "fs" {
			assert_eq!(tool["execution_modes"], json!(["sync", "background"]));
		} else {
			assert_eq!(tool["execution_modes"], json!(["sync"]));
		}
	}
}

#[test]
fn runtime_tools_call_returns_invalid_argument_for_policy_disabled_tool() {
	let (services, _workspace) =
		build_services_for_fixture_with_disabled_tools(true, vec!["search".to_string()]);
	let frame = json!({
		"jsonrpc": "2.0",
		"id": 105,
		"method": "tools/call",
		"params": {
			"name": "search",
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
		json!("invalid_argument")
	);
	assert_eq!(
		payload["result"]["structuredContent"]["error"]["retryable"],
		json!(false)
	);
}
