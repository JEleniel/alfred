//! Canonical capability registry for Alfred.

use crate::design::{
	MAX_BULK_OPERATIONS_PER_CALL, MAX_INLINE_UTF8_BYTES, MAX_LOG_RECORDS_PER_CALL,
	MAX_PATCH_FILES_PER_CALL, SERVER_VERSION, TOOL_SCHEMA_VERSION,
};
use crate::protocol::{ExecutionMode, ToolDescriptor, ToolLimits, ToolRoute};

const SYNC_MODES: &[ExecutionMode] = &[ExecutionMode::Sync];
const SYNC_BACKGROUND_MODES: &[ExecutionMode] = &[ExecutionMode::Sync, ExecutionMode::Background];
const SYNC_STREAM_MODES: &[ExecutionMode] = &[ExecutionMode::Sync, ExecutionMode::Stream];

const FS_LIMITS: ToolLimits = ToolLimits {
	max_inline_utf8_bytes: Some(MAX_INLINE_UTF8_BYTES),
	max_patch_files_per_call: None,
	max_bulk_operations_per_call: Some(MAX_BULK_OPERATIONS_PER_CALL),
	max_log_records_per_call: None,
};

const PATCH_LIMITS: ToolLimits = ToolLimits {
	max_inline_utf8_bytes: Some(MAX_INLINE_UTF8_BYTES),
	max_patch_files_per_call: Some(MAX_PATCH_FILES_PER_CALL),
	max_bulk_operations_per_call: None,
	max_log_records_per_call: None,
};

const LOG_LIMITS: ToolLimits = ToolLimits {
	max_inline_utf8_bytes: None,
	max_patch_files_per_call: None,
	max_bulk_operations_per_call: None,
	max_log_records_per_call: Some(MAX_LOG_RECORDS_PER_CALL),
};

struct CapabilityDefinition {
	route: ToolRoute,
	descriptor: ToolDescriptor<'static>,
}

const fn capability(
	name: &'static str,
	route: ToolRoute,
	execution_modes: &'static [ExecutionMode],
	limits: Option<ToolLimits>,
) -> CapabilityDefinition {
	CapabilityDefinition {
		route,
		descriptor: ToolDescriptor {
			name,
			version: SERVER_VERSION,
			schema_version: TOOL_SCHEMA_VERSION,
			execution_modes,
			limits,
		},
	}
}

const CAPABILITY_DEFINITIONS: &[CapabilityDefinition] = &[
	capability("capabilities", ToolRoute::Capabilities, SYNC_MODES, None),
	capability("fs", ToolRoute::Fs, SYNC_BACKGROUND_MODES, Some(FS_LIMITS)),
	capability("logs", ToolRoute::Logs, SYNC_STREAM_MODES, Some(LOG_LIMITS)),
	capability("memory", ToolRoute::Memory, SYNC_MODES, None),
	capability("patch", ToolRoute::Patch, SYNC_MODES, Some(PATCH_LIMITS)),
	capability("plan", ToolRoute::Plan, SYNC_MODES, None),
	capability("search", ToolRoute::Search, SYNC_MODES, None),
	capability("status", ToolRoute::Status, SYNC_MODES, None),
	capability("workspace_dir", ToolRoute::WorkspaceDir, SYNC_MODES, None),
];

/// Capability registry with a single source of truth for the public tools.
pub struct CapabilityRegistry {
	tools: Vec<ToolDescriptor<'static>>,
}

impl CapabilityRegistry {
	/// Creates the canonical registry.
	pub fn new() -> Self {
		Self {
			tools: CAPABILITY_DEFINITIONS
				.iter()
				.map(|definition| definition.descriptor.clone())
				.collect(),
		}
	}

	/// Returns the advertised public tools.
	pub fn tools(&self) -> &[ToolDescriptor<'static>] {
		&self.tools
	}

	/// Returns the canonical route for an advertised tool.
	pub(crate) fn route_for(&self, name: &str) -> Option<ToolRoute> {
		CAPABILITY_DEFINITIONS
			.iter()
			.find(|definition| definition.descriptor.name == name)
			.map(|definition| definition.route)
	}
}

impl Default for CapabilityRegistry {
	fn default() -> Self {
		Self::new()
	}
}
