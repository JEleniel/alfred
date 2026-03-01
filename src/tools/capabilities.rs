//! Capability discovery tool group.

use serde_json::{Value, json};

use crate::services::ServiceContainer;
use crate::tools::ToolRegistry;

/// Capability discovery tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct CapabilityTools;

impl CapabilityTools {
	pub const NAMES: &'static [&'static str] = &["capabilities"];
}

/// Handles capability-discovery tool calls.
pub fn dispatch_tool_call(name: &str, services: &ServiceContainer) -> Option<Value> {
	match name {
		"capabilities" => Some(capabilities(services)),
		_ => None,
	}
}

fn capabilities(services: &ServiceContainer) -> Value {
	let tools = ToolRegistry::new()
		.tool_names_for_config(&services.config)
		.into_iter()
		.map(tool_descriptor)
		.collect::<Vec<_>>();

	json!({
		"tools": tools,
	})
}

fn tool_descriptor(name: &str) -> Value {
	let execution_modes = match name {
		"logs" => json!(["sync", "stream"]),
		"fs" => json!(["sync", "background"]),
		_ => json!(["sync"]),
	};

	let descriptor = json!({
		"name": name,
		"version": env!("CARGO_PKG_VERSION"),
		"schema_version": env!("CARGO_PKG_VERSION"),
		"execution_modes": execution_modes,
	});

	descriptor
}
