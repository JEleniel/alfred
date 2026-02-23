//! Capability discovery tool group.

use serde_json::{Value, json};

use crate::services::ServiceContainer;
use crate::tools::ToolRegistry;

use super::workspace_query::MAX_FILE_CHUNK_BYTES;

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
	let mut descriptor = json!({
		"name": name,
		"version": env!("CARGO_PKG_VERSION"),
		"schema_version": env!("CARGO_PKG_VERSION"),
		"execution_modes": ["sync"],
	});

	if name == "file_read_bytes" {
		descriptor["limits"] = json!({
			"max_file_chunk_bytes": MAX_FILE_CHUNK_BYTES,
		});
	}

	descriptor
}
