//! Protocol envelope models for tool responses.

use std::time::Instant;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::configuration::AppConfig;
use crate::errors::ToolError;
use crate::services::ServiceContainer;
use crate::tools::ToolRegistry;
use crate::tools::dispatch_tool_call;

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
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub warnings: Vec<ToolWarning>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub transport_equivalent: Option<TransportEquivalent>,
}

/// Transport-equivalent metadata for stdio responses.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransportEquivalent {
	pub http_status: u16,
}

impl TransportEquivalent {
	fn accepted() -> Self {
		Self { http_status: 202 }
	}
}

/// Deterministic result envelope used by all tools.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ToolResponse<T> {
	Ok {
		data: T,
		meta: ToolMeta,
	},
	Error {
		error: ToolError,
		meta: ToolMeta,
	},
	Pending {
		job_id: String,
		data: T,
		meta: ToolMeta,
	},
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
	pub fn pending(job_id: String, data: T, mut meta: ToolMeta) -> Self {
		meta.transport_equivalent = Some(TransportEquivalent::accepted());
		Self::Pending { job_id, data, meta }
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
		"prompts/list" => "prompts_list_request",
		"prompts/get" => "prompts_get_request",
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
		Some("tools/list") => {
			let config = AppConfig::load_default().context("failed to load configuration")?;
			let tool_names = ToolRegistry::new().tool_names_for_config(&config);
			build_tools_list_response(jsonrpc, id, tool_names).map(Some)
		}
		Some("prompts/list") => build_prompts_list_response(jsonrpc, id).map(Some),
		Some("prompts/get") => {
			let config = AppConfig::load_default().context("failed to load configuration")?;
			build_prompts_get_response(jsonrpc, id, frame.get("params"), config.workspace_root)
				.map(Some)
		}
		Some(other) => build_method_not_implemented_response(jsonrpc, id, other).map(Some),
		None => Ok(None),
	}
}

/// Handles runtime frames, including tool invocation calls.
pub fn handle_runtime_frame(
	raw_frame: &str,
	services: &ServiceContainer,
) -> Result<Option<String>> {
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
		Some("tools/list") => {
			let tool_names = ToolRegistry::new().tool_names_for_config(&services.config);
			build_tools_list_response(jsonrpc, id, tool_names).map(Some)
		}
		Some("tools/call") => {
			build_tools_call_response(jsonrpc, id, frame.get("params"), services).map(Some)
		}
		Some("prompts/list") => build_prompts_list_response(jsonrpc, id).map(Some),
		Some("prompts/get") => build_prompts_get_response(
			jsonrpc,
			id,
			frame.get("params"),
			services.indexer.workspace_root().to_path_buf(),
		)
		.map(Some),
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
				"prompts": {
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

fn build_tools_list_response(jsonrpc: &str, id: Value, tool_names: Vec<&str>) -> Result<String> {
	let tools = tool_names
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

fn build_prompts_list_response(jsonrpc: &str, id: Value) -> Result<String> {
	let prompts = vec![json!({
		"name": "alfred_agent",
		"description": "Instructions for using Alfred MCP tools effectively.",
		"arguments": [],
	})];

	let response = json!({
		"jsonrpc": jsonrpc,
		"id": id,
		"result": {
			"prompts": prompts,
		},
	});

	serde_json::to_string(&response).context("failed to serialize prompts/list response")
}

fn build_prompts_get_response(
	jsonrpc: &str,
	id: Value,
	params: Option<&Value>,
	workspace_root: std::path::PathBuf,
) -> Result<String> {
	let prompt_name = match parse_prompts_get_params(params) {
		Ok(name) => name,
		Err(message) => return build_invalid_params_response(jsonrpc, id, message.as_str()),
	};

	let workspace_root = workspace_root.to_string_lossy().to_string();
	let (description, text) = match prompt_name.as_str() {
		"alfred_agent" => (
			"Alfred MCP usage prompt".to_string(),
			format!(
				"You are operating against an Alfred MCP server for a Rust workspace at: {workspace_root}.\n\nUse tools conservatively and deterministically:\n- Use `capabilities` or `tools/list` to discover tool names.\n- Prefer patch-based edits via the `patch` tool (dry-run by default) and only then apply.\n- Use `patch` with operation `revert` only to undo the latest applied patch batch.\n- Use `status` to check index readiness and memory usage before expensive queries.\n\nWhen modifying files: keep changes minimal, avoid rewriting entire files, and respect workspace boundaries."
			),
		),
		_ => {
			let message = format!("prompt not found: {prompt_name}");
			return build_invalid_params_response(jsonrpc, id, message.as_str());
		}
	};

	let response = json!({
		"jsonrpc": jsonrpc,
		"id": id,
		"result": {
			"description": description,
			"messages": [
				{
					"role": "user",
					"content": [
						{
							"type": "text",
							"text": text,
						}
					]
				}
			]
		}
	});

	serde_json::to_string(&response).context("failed to serialize prompts/get response")
}

fn parse_prompts_get_params(params: Option<&Value>) -> std::result::Result<String, String> {
	let Some(params) = params.and_then(Value::as_object) else {
		return Err("prompts/get params must be an object".to_string());
	};

	let Some(name) = params.get("name").and_then(Value::as_str) else {
		return Err("prompts/get params.name must be a string".to_string());
	};

	Ok(name.to_string())
}

fn build_tools_call_response(
	jsonrpc: &str,
	id: Value,
	params: Option<&Value>,
	services: &ServiceContainer,
) -> Result<String> {
	let (tool_name, arguments) = match parse_tool_call_params(params) {
		Ok(values) => values,
		Err(message) => return build_invalid_params_response(jsonrpc, id, message.as_str()),
	};

	let started = Instant::now();
	let (response_body, is_error) =
		match dispatch_tool_call(tool_name.as_str(), arguments, services) {
			Ok(data) => (
				ToolResponse::ok(data, tool_meta(tool_name.as_str(), started)),
				false,
			),
			Err(error) => (
				ToolResponse::<Value>::error(error.into(), tool_meta(tool_name.as_str(), started)),
				true,
			),
		};

	let structured_content = serde_json::to_value(response_body)
		.context("failed to serialize tool-call response envelope")?;
	let text_content = serde_json::to_string(&structured_content)
		.context("failed to serialize tool-call response text content")?;
	let response = json!({
		"jsonrpc": jsonrpc,
		"id": id,
		"result": {
			"content": [
				{
					"type": "text",
					"text": text_content,
				}
			],
			"structuredContent": structured_content,
			"isError": is_error,
		},
	});

	serde_json::to_string(&response).context("failed to serialize tools/call response")
}

fn parse_tool_call_params(params: Option<&Value>) -> std::result::Result<(String, Value), String> {
	let Some(params) = params.and_then(Value::as_object) else {
		return Err("tools/call params must be an object".to_string());
	};

	let Some(name) = params.get("name").and_then(Value::as_str) else {
		return Err("tools/call params.name must be a string".to_string());
	};

	let arguments = params
		.get("arguments")
		.cloned()
		.unwrap_or_else(|| json!({}));
	if !arguments.is_object() {
		return Err("tools/call params.arguments must be an object".to_string());
	}

	Ok((name.to_string(), arguments))
}

fn build_invalid_params_response(jsonrpc: &str, id: Value, message: &str) -> Result<String> {
	let response = json!({
		"jsonrpc": jsonrpc,
		"id": id,
		"error": {
			"code": -32602,
			"message": message,
		},
	});

	serde_json::to_string(&response).context("failed to serialize invalid-params response")
}

fn tool_meta(name: &str, started: Instant) -> ToolMeta {
	ToolMeta {
		tool: name.to_string(),
		schema_version: env!("CARGO_PKG_VERSION").to_string(),
		duration_ms: Some(started.elapsed().as_millis()),
		warnings: Vec::new(),
		transport_equivalent: None,
	}
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
