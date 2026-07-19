//! Canonical tool execution for the currently implemented Alfred tools.

use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sysinfo::System;

use crate::design::{SERVER_VERSION, TOOL_SCHEMA_VERSION};
use crate::envelope::{DeterministicError, EnvelopeStatus, ToolEnvelope, ToolMeta, ToolWarning};
use crate::protocol::{ToolDescriptor, ToolRoute};

pub(crate) fn execute_tool(
	descriptor: &ToolDescriptor<'static>,
	route: ToolRoute,
	arguments: Option<&Value>,
	tools: &[ToolDescriptor<'static>],
) -> ToolEnvelope<Value> {
	let started = Instant::now();

	match execute_tool_payload(route, descriptor.name, arguments, tools) {
		Ok(payload) => ToolEnvelope {
			status: EnvelopeStatus::Ok,
			data: Some(payload.data),
			warnings: payload.warnings,
			errors: Vec::new(),
			meta: meta(descriptor.name, started),
		},
		Err(error) => ToolEnvelope {
			status: EnvelopeStatus::Error,
			data: None,
			warnings: Vec::new(),
			errors: vec![error],
			meta: meta(descriptor.name, started),
		},
	}
}

pub(crate) fn unknown_tool_response(name: &str) -> ToolEnvelope<Value> {
	let started = Instant::now();

	ToolEnvelope {
		status: EnvelopeStatus::Error,
		data: None,
		warnings: Vec::new(),
		errors: vec![invalid_argument_error(
			"tool",
			"The requested tool name is not part of Alfred's advertised tool surface.",
			Some(json!({ "reason": "unknown_tool", "tool": name })),
		)],
		meta: meta(name, started),
	}
}

fn execute_tool_payload(
	route: ToolRoute,
	tool_name: &str,
	arguments: Option<&Value>,
	tools: &[ToolDescriptor<'static>],
) -> Result<ToolPayload, DeterministicError> {
	match route {
		ToolRoute::Capabilities => capabilities_payload(tool_name, arguments, tools),
		ToolRoute::WorkspaceDir => workspace_dir_payload(tool_name, arguments),
		ToolRoute::Status => status_payload(tool_name, arguments),
		_ => Err(tool_unavailable_error(tool_name)),
	}
}

fn capabilities_payload(
	tool_name: &str,
	arguments: Option<&Value>,
	tools: &[ToolDescriptor<'static>],
) -> Result<ToolPayload, DeterministicError> {
	let _: NoArguments = parse_arguments(tool_name, arguments)?;
	let data = serialize_payload(CapabilitiesPayload {
		tools: tools.to_vec(),
	})?;

	Ok(ToolPayload {
		data,
		warnings: Vec::new(),
	})
}

fn workspace_dir_payload(
	tool_name: &str,
	arguments: Option<&Value>,
) -> Result<ToolPayload, DeterministicError> {
	let _: NoArguments = parse_arguments(tool_name, arguments)?;
	let workspace_root = resolve_workspace_root(tool_name)?;
	let data = serialize_payload(WorkspaceDirPayload {
		root: workspace_root.rendered,
	})?;

	Ok(ToolPayload {
		data,
		warnings: workspace_root.warnings,
	})
}

fn status_payload(
	tool_name: &str,
	arguments: Option<&Value>,
) -> Result<ToolPayload, DeterministicError> {
	let status_arguments: StatusArguments = parse_arguments(tool_name, arguments)?;
	let _verbose = status_arguments.verbose;
	let workspace_root = resolve_workspace_root(tool_name)?;
	let plan_path = render_protocol_path(default_plan_path(&workspace_root.actual));
	let log_path = render_protocol_path(default_runtime_logs_path(&workspace_root.actual));
	let mut system = System::new();
	let mut warnings = workspace_root.warnings;
	warnings.extend(plan_path.warnings);
	warnings.extend(log_path.warnings);
	system.refresh_memory();

	let data = serialize_payload(StatusPayload {
		version: SERVER_VERSION.to_owned(),
		total_memory: system.total_memory(),
		workspace_root: workspace_root.rendered,
		paths: StatusPaths {
			plan: plan_path.rendered,
			alfred_logs: log_path.rendered,
		},
		index: StatusIndex {
			ready: false,
			indexed_files: 0,
			indexed_directories: 0,
		},
		memory: StatusMemory {
			ready: false,
			indexed_memories: 0,
		},
	})?;

	Ok(ToolPayload { data, warnings })
}

fn parse_arguments<T>(tool_name: &str, arguments: Option<&Value>) -> Result<T, DeterministicError>
where
	T: DeserializeOwned + Default,
{
	match arguments {
		None => Ok(T::default()),
		Some(Value::Null) => Ok(T::default()),
		Some(value) => serde_json::from_value(value.clone()).map_err(|_| {
			invalid_argument_error(
				tool_name,
				"The supplied tool arguments do not match the canonical contract.",
				Some(json!({ "reason": "invalid_arguments", "tool": tool_name })),
			)
		}),
	}
}

fn resolve_workspace_root(tool_name: &str) -> Result<RenderedPath, DeterministicError> {
	let workspace_root = env::current_dir().map_err(|_| {
		io_error(
			tool_name,
			"Alfred could not resolve the current workspace root from the process working directory.",
		)
	})?;

	Ok(render_protocol_path(workspace_root))
}

fn render_protocol_path(path: PathBuf) -> RenderedPath {
	if let Some(rendered) = path.to_str() {
		let rendered = rendered.to_owned();

		return RenderedPath {
			actual: path,
			rendered,
			warnings: Vec::new(),
		};
	}

	let rendered = match path.to_string_lossy() {
		Cow::Borrowed(text) => text.to_owned(),
		Cow::Owned(text) => text,
	};

	RenderedPath {
		actual: path,
		rendered: percent_encode_utf8(&rendered),
		warnings: vec![ToolWarning {
			kind: "path_encoded".to_owned(),
			message: Some(
				"A protocol path required deterministic percent-encoding to remain valid UTF-8."
					.to_owned(),
			),
		}],
	}
}

fn percent_encode_utf8(text: &str) -> String {
	let mut encoded = String::new();

	for byte in text.bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/' | b'~' | b' ' => {
				encoded.push(char::from(byte));
			}
			_ => encoded.push_str(&format!("%{byte:02X}")),
		}
	}

	encoded
}

fn default_plan_path(workspace_root: &Path) -> PathBuf {
	let design_plan = workspace_root.join("docs/design/ProjectPlan.md");

	if design_plan.exists() {
		return design_plan;
	}

	workspace_root.join("ProjectPlan.md")
}

fn default_runtime_logs_path(workspace_root: &Path) -> PathBuf {
	workspace_root.join(".alfred/logs")
}

fn serialize_payload<T: Serialize>(payload: T) -> Result<Value, DeterministicError> {
	serde_json::to_value(payload).map_err(|_| {
		internal_error(
			"tool",
			"Alfred could not serialize the tool result payload into JSON.",
		)
	})
}

fn meta(tool_name: &str, started: Instant) -> ToolMeta {
	ToolMeta {
		tool: tool_name.to_owned(),
		schema_version: TOOL_SCHEMA_VERSION.to_owned(),
		duration_ms: Some(started.elapsed().as_millis() as u64),
		transport_equivalent: None,
	}
}

fn invalid_argument_error(
	tool_name: &str,
	message: &str,
	details: Option<Value>,
) -> DeterministicError {
	DeterministicError {
		kind: "invalid_argument".to_owned(),
		message: message.to_owned(),
		retryable: false,
		details: details.or_else(|| Some(json!({ "tool": tool_name }))),
	}
}

fn io_error(tool_name: &str, message: &str) -> DeterministicError {
	DeterministicError {
		kind: "io_error".to_owned(),
		message: message.to_owned(),
		retryable: true,
		details: Some(json!({ "tool": tool_name })),
	}
}

fn internal_error(tool_name: &str, message: &str) -> DeterministicError {
	DeterministicError {
		kind: "internal".to_owned(),
		message: message.to_owned(),
		retryable: true,
		details: Some(json!({ "tool": tool_name })),
	}
}

fn tool_unavailable_error(tool_name: &str) -> DeterministicError {
	DeterministicError {
		kind: "tool_unavailable".to_owned(),
		message: "The requested tool is advertised but is not implemented yet.".to_owned(),
		retryable: true,
		details: Some(json!({ "reason": "not_implemented", "tool": tool_name })),
	}
}

struct ToolPayload {
	data: Value,
	warnings: Vec<ToolWarning>,
}

struct RenderedPath {
	actual: PathBuf,
	rendered: String,
	warnings: Vec<ToolWarning>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct NoArguments {}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusArguments {
	verbose: bool,
}

#[derive(Serialize)]
struct CapabilitiesPayload {
	tools: Vec<ToolDescriptor<'static>>,
}

#[derive(Serialize)]
struct WorkspaceDirPayload {
	root: String,
}

#[derive(Serialize)]
struct StatusPayload {
	version: String,
	total_memory: u64,
	workspace_root: String,
	paths: StatusPaths,
	index: StatusIndex,
	memory: StatusMemory,
}

#[derive(Serialize)]
struct StatusPaths {
	plan: String,
	alfred_logs: String,
}

#[derive(Serialize)]
struct StatusIndex {
	ready: bool,
	indexed_files: u64,
	indexed_directories: u64,
}

#[derive(Serialize)]
struct StatusMemory {
	ready: bool,
	indexed_memories: u64,
}
