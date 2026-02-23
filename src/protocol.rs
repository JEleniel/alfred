//! Protocol envelope models for tool responses.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::errors::ToolError;
use crate::tools::ToolRegistry;

const JSONRPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-11-05";

/// Warning metadata emitted with a tool result.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ToolWarning {
	pub kind: String,
}

/// Shared metadata attached to all tool results.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ToolMeta {
	pub tool: String,
	pub schema_version: String,
	pub duration_ms: Option<u128>,
	pub warnings: Vec<ToolWarning>,
}

/// Deterministic result envelope used by all tools.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ToolResponse<T> {
	Ok { data: T, meta: ToolMeta },
	Error { error: ToolError, meta: ToolMeta },
	Pending { job_id: String, meta: ToolMeta },
}

impl<T> ToolResponse<T> {
	/// Creates a successful tool response.
	pub fn ok(data: T, meta: ToolMeta) -> Self {
		Self::Ok { data, meta }
	}

	/// Creates an error tool response.
	pub fn error(error: ToolError, meta: ToolMeta) -> Self {
		Self::Error { error, meta }
	}

	/// Creates a pending tool response.
	pub fn pending(job_id: String, meta: ToolMeta) -> Self {
		Self::Pending { job_id, meta }
	}
}

/// Classifies a raw JSON-RPC frame for trace logging.
pub fn classify_wire_frame(raw_frame: &str) -> &'static str {
	let Ok(frame) = serde_json::from_str::<Value>(raw_frame) else {
		return "malformed";
	};

	let Some(method) = frame.get("method").and_then(Value::as_str) else {
		if frame.get("result").is_some() || frame.get("error").is_some() {
			return "response";
		}

		return "unknown";
	};

	match method {
		"initialize" => "initialize_request",
		"tools/list" => "tools_list_request",
		"tools/call" => "tools_call_request",
		"notifications/initialized" => "initialized_notification",
		_ if frame.get("id").is_some() => "request",
		_ => "notification",
	}
}

/// Extracts optional workspace hints from an initialize frame.
pub fn extract_initialize_workspace_hints(raw_frame: &str) -> Result<Option<Value>> {
	let frame = serde_json::from_str::<Value>(raw_frame)
		.context("failed to parse frame while extracting workspace hints")?;

	if frame.get("method").and_then(Value::as_str) != Some("initialize") {
		return Ok(None);
	}

	let Some(params) = frame.get("params").and_then(Value::as_object) else {
		return Ok(None);
	};

	let mut hints = serde_json::Map::new();
	if let Some(root_uri) = params.get("rootUri").and_then(Value::as_str) {
		hints.insert("rootUri".to_string(), Value::String(root_uri.to_string()));
	}
	if let Some(cwd) = params.get("cwd").and_then(Value::as_str) {
		hints.insert("cwd".to_string(), Value::String(cwd.to_string()));
	}
	if let Some(workspace_root) = params.get("workspaceRoot").and_then(Value::as_str) {
		hints.insert(
			"workspaceRoot".to_string(),
			Value::String(workspace_root.to_string()),
		);
	}
	if let Some(workspace_folders) = params.get("workspaceFolders").and_then(Value::as_array) {
		hints.insert(
			"workspaceFolders".to_string(),
			normalize_workspace_folders(workspace_folders),
		);
	}

	if hints.is_empty() {
		return Ok(None);
	}

	Ok(Some(Value::Object(hints)))
}

/// Handles startup-frame responses for the stdio JSON-RPC skeleton.
pub fn handle_startup_frame(raw_frame: &str) -> Result<Option<String>> {
	let frame = serde_json::from_str::<Value>(raw_frame)
		.context("failed to parse inbound JSON-RPC frame")?;

	let method = frame.get("method").and_then(Value::as_str);
	if method == Some("notifications/initialized") {
		return Ok(None);
	}

	let Some(id) = frame.get("id").cloned() else {
		return Ok(None);
	};

	let jsonrpc = frame
		.get("jsonrpc")
		.and_then(Value::as_str)
		.unwrap_or(JSONRPC_VERSION);

	match method {
		Some("initialize") => build_initialize_response(jsonrpc, id).map(Some),
		Some("tools/list") => build_tools_list_response(jsonrpc, id).map(Some),
		Some(other) => build_method_not_implemented_response(jsonrpc, id, other).map(Some),
		None => Ok(None),
	}
}

fn normalize_workspace_folders(raw_folders: &[Value]) -> Value {
	let folders = raw_folders
		.iter()
		.filter_map(Value::as_object)
		.map(|folder| {
			json!({
				"uri": folder.get("uri").and_then(Value::as_str).unwrap_or_default(),
				"name": folder.get("name").and_then(Value::as_str).unwrap_or_default(),
			})
		})
		.collect::<Vec<_>>();

	Value::Array(folders)
}

fn build_initialize_response(jsonrpc: &str, id: Value) -> Result<String> {
	let response = json!({
		"jsonrpc": jsonrpc,
		"id": id,
		"result": {
			"protocolVersion": MCP_PROTOCOL_VERSION,
			"capabilities": {
				"tools": {
					"listChanged": false,
				},
			},
			"serverInfo": {
				"name": env!("CARGO_PKG_NAME"),
				"version": env!("CARGO_PKG_VERSION"),
			},
		},
	});

	serde_json::to_string(&response).context("failed to serialize initialize response")
}

fn build_tools_list_response(jsonrpc: &str, id: Value) -> Result<String> {
	let tools = ToolRegistry::new()
		.tool_names()
		.into_iter()
		.map(|name| {
			json!({
				"name": name,
				"description": format!("Alfred tool: {name}"),
				"inputSchema": {
					"type": "object",
					"additionalProperties": true,
				},
			})
		})
		.collect::<Vec<_>>();

	let response = json!({
		"jsonrpc": jsonrpc,
		"id": id,
		"result": {
			"tools": tools,
		},
	});

	serde_json::to_string(&response).context("failed to serialize tools/list response")
}

fn build_method_not_implemented_response(jsonrpc: &str, id: Value, method: &str) -> Result<String> {
	let response = json!({
		"jsonrpc": jsonrpc,
		"id": id,
		"error": {
			"code": -32601,
			"message": format!("method not implemented in skeleton: {method}"),
		},
	});

	serde_json::to_string(&response).context("failed to serialize method-not-implemented response")
}
