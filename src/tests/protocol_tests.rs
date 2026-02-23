use serde_json::{Value, json};

use crate::protocol::{
	classify_wire_frame, extract_initialize_workspace_hints, handle_startup_frame,
};

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
