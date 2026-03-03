//! Capability discovery tool group.

use serde_json::{Value, json};

use crate::services::ServiceContainer;
use crate::tools::ToolRegistry;

/// Maximum UTF-8 bytes accepted in inline tool-call arguments.
pub const MAX_INLINE_UTF8_BYTES: usize = 1_048_576;
/// Maximum number of patch file entries accepted by `patch` per call.
pub const MAX_PATCH_FILES_PER_CALL: usize = 128;
/// Maximum number of bulk operations accepted by `fs` execute mode per call.
pub const MAX_BULK_OPERATIONS_PER_CALL: usize = 256;
/// Maximum number of log records returned per `logs` call.
pub const MAX_LOG_RECORDS_PER_CALL: usize = 1_000;

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
	let limits = match name {
		"patch" => json!({
			"max_inline_utf8_bytes": MAX_INLINE_UTF8_BYTES,
			"max_patch_files_per_call": MAX_PATCH_FILES_PER_CALL,
		}),
		"fs" => json!({
			"max_inline_utf8_bytes": MAX_INLINE_UTF8_BYTES,
			"max_bulk_operations_per_call": MAX_BULK_OPERATIONS_PER_CALL,
		}),
		"logs" => json!({
			"max_inline_utf8_bytes": MAX_INLINE_UTF8_BYTES,
			"max_log_records_per_call": MAX_LOG_RECORDS_PER_CALL,
		}),
		_ => json!({
			"max_inline_utf8_bytes": MAX_INLINE_UTF8_BYTES,
		}),
	};

	let descriptor = json!({
		"name": name,
		"version": env!("CARGO_PKG_VERSION"),
		"schema_version": env!("CARGO_PKG_VERSION"),
		"execution_modes": execution_modes,
		"limits": limits,
	});

	descriptor
}
