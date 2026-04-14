use crate::app::Alfred;
use crate::design::{SERVER_VERSION, TOOL_SCHEMA_VERSION};
use crate::envelope::{
	DeterministicError, EnvelopeStatus, ToolEnvelope, ToolMeta, ToolWarning, TransportEquivalent,
};
use crate::protocol::ExecutionMode;
use crate::router::ToolRoute;

#[test]
fn capabilities_follow_the_canonical_public_tool_surface() {
	let app = Alfred::new();
	let names: Vec<&str> = app.capabilities().iter().map(|tool| tool.name).collect();
	let expected_names = [
		"capabilities",
		"fs",
		"logs",
		"memory",
		"patch",
		"plan",
		"search",
		"status",
		"workspace_dir",
	];

	assert_eq!(names, expected_names);
	assert!(app.capability("capabilities").is_some());
	assert!(app.capability("not-a-tool").is_none());
	assert!(
		app.capabilities().iter().all(
			|tool| tool.version == SERVER_VERSION && tool.schema_version == TOOL_SCHEMA_VERSION
		)
	);
}

#[test]
fn tool_envelope_serializes_the_canonical_shape() {
	let envelope = ToolEnvelope {
		status: EnvelopeStatus::Ok,
		data: Some(serde_json::json!({"root": "/workspace"})),
		warnings: vec![ToolWarning {
			kind: "path_encoded".to_string(),
			message: Some("workspace path was percent-encoded".to_string()),
		}],
		errors: vec![DeterministicError {
			kind: "internal".to_string(),
			message: "unexpected envelope state".to_string(),
			retryable: true,
			details: None,
		}],
		meta: ToolMeta {
			tool: "workspace_dir".to_string(),
			schema_version: TOOL_SCHEMA_VERSION.to_string(),
			duration_ms: Some(12),
			transport_equivalent: Some(TransportEquivalent {
				http_status: Some(200),
			}),
		},
	};

	let serialized = serde_json::to_value(envelope).expect("envelope should serialize");
	assert_eq!(serialized["status"], "ok");
	assert_eq!(serialized["meta"]["tool"], "workspace_dir");
	assert_eq!(serialized["meta"]["schema_version"], TOOL_SCHEMA_VERSION);
	assert_eq!(serialized["warnings"][0]["kind"], "path_encoded");
}

#[test]
fn execution_mode_serializes_as_its_variant_name() {
	let serialized = serde_json::to_value(ExecutionMode::Sync).expect("mode should serialize");

	assert_eq!(serialized, "Sync");
}

#[test]
fn router_dispatches_known_tools_and_rejects_unknown_names() {
	let app = Alfred::new();
	let routed = app
		.dispatch_tool("workspace_dir")
		.expect("tool should route");

	assert!(matches!(routed.route(), ToolRoute::WorkspaceDir));
	assert_eq!(routed.descriptor().name, "workspace_dir");
	assert!(app.dispatch_tool("not-a-tool").is_err());
}

#[test]
fn invoke_tool_returns_the_capabilities_payload() {
	let app = Alfred::new();
	let envelope = app.invoke_tool("capabilities", None);
	let tool_names: Vec<String> = envelope.data.expect("payload should exist")["tools"]
		.as_array()
		.expect("tools should be an array")
		.iter()
		.map(|tool| tool["name"].as_str().expect("name should exist").to_owned())
		.collect();

	assert_eq!(envelope.status, EnvelopeStatus::Ok);
	assert_eq!(tool_names.len(), app.capabilities().len());
	assert_eq!(tool_names.first().map(String::as_str), Some("capabilities"));
	assert!(envelope.errors.is_empty());
}

#[test]
fn invoke_tool_returns_the_workspace_root_payload() {
	let app = Alfred::new();
	let expected_root = std::env::current_dir().expect("cwd should resolve");
	let envelope = app.invoke_tool("workspace_dir", None);

	assert_eq!(envelope.status, EnvelopeStatus::Ok);
	assert_eq!(
		envelope.data.expect("payload should exist")["root"],
		serde_json::Value::String(expected_root.to_string_lossy().into_owned())
	);
	assert!(envelope.errors.is_empty());
}

#[test]
fn invoke_tool_returns_the_status_payload() {
	let app = Alfred::new();
	let envelope = app.invoke_tool("status", Some(&serde_json::json!({ "verbose": true })));
	let payload = envelope.data.expect("payload should exist");

	assert_eq!(envelope.status, EnvelopeStatus::Ok);
	assert_eq!(
		payload["version"],
		serde_json::Value::String(SERVER_VERSION.to_owned())
	);
	assert_eq!(payload["index"]["ready"], serde_json::Value::Bool(false));
	assert_eq!(
		payload["memory"]["indexed_memories"],
		serde_json::Value::Number(0.into())
	);
	assert!(payload["paths"]["plan"].as_str().is_some());
	assert!(payload["paths"]["alfred_logs"].as_str().is_some());
}

#[test]
fn invoke_tool_reports_unimplemented_and_unknown_tools_deterministically() {
	let app = Alfred::new();
	let unimplemented = app.invoke_tool("search", None);
	let unknown = app.invoke_tool("not-a-tool", None);

	assert_eq!(unimplemented.status, EnvelopeStatus::Error);
	assert_eq!(unimplemented.errors[0].kind, "tool_unavailable");
	assert_eq!(unknown.status, EnvelopeStatus::Error);
	assert_eq!(unknown.errors[0].kind, "invalid_argument");
	assert_eq!(
		unknown.errors[0].details.as_ref().expect("details")["reason"],
		"unknown_tool"
	);
}
