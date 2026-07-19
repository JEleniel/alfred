//! Shared protocol-facing types for Alfred.

use serde::{Deserialize, Serialize};

/// Alfred tool execution mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ExecutionMode {
	/// A single synchronous response.
	Sync,
	/// A response that may continue in the background.
	Background,
	/// A response that may stream multiple envelopes.
	Stream,
}

/// Canonical dispatch routes for Alfred's public tools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolRoute {
	Capabilities,
	WorkspaceDir,
	Status,
	Search,
	Fs,
	Patch,
	Logs,
	Plan,
	Memory,
}

/// Canonical limits advertised by a tool.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct ToolLimits {
	/// Maximum inline UTF-8 payload size.
	pub max_inline_utf8_bytes: Option<usize>,
	/// Maximum patch file count per call.
	pub max_patch_files_per_call: Option<usize>,
	/// Maximum bulk operation count per call.
	pub max_bulk_operations_per_call: Option<usize>,
	/// Maximum log record count per call.
	pub max_log_records_per_call: Option<usize>,
}

/// A single public tool descriptor.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ToolDescriptor<'a> {
	/// Stable public tool name.
	pub name: &'a str,
	/// Tool version.
	pub version: &'a str,
	/// Tool schema version.
	pub schema_version: &'a str,
	/// Supported execution modes.
	pub execution_modes: &'a [ExecutionMode],
	/// Optional capability limits.
	pub limits: Option<ToolLimits>,
}
