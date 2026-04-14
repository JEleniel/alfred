//! Canonical tool result envelope for Alfred.

use serde::{Deserialize, Serialize};

/// Envelope state for a tool result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvelopeStatus {
	/// The tool completed successfully.
	Ok,
	/// The tool completed with deterministic errors.
	Error,
	/// The tool was accepted for background execution.
	Pending,
}

/// Common metadata shared by tool result envelopes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolMeta {
	/// Tool name.
	pub tool: String,
	/// Tool result schema version.
	pub schema_version: String,
	/// Observed duration in milliseconds.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub duration_ms: Option<u64>,
	/// Transport-specific metadata.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub transport_equivalent: Option<TransportEquivalent>,
}

/// Transport-specific metadata preserved in the envelope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransportEquivalent {
	/// Expected HTTP status for the envelope when bridged from stdio.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub http_status: Option<u16>,
}

/// Deterministic warning emitted alongside a result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolWarning {
	/// Stable warning kind.
	pub kind: String,
	/// Human-readable warning message.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub message: Option<String>,
}

/// Deterministic error emitted by Alfred.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeterministicError {
	/// Stable error kind.
	pub kind: String,
	/// Stable human-readable message.
	pub message: String,
	/// Whether the client may retry the operation.
	pub retryable: bool,
	/// Optional structured details.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub details: Option<serde_json::Value>,
}

/// Generic Alfred tool result envelope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolEnvelope<T> {
	/// Result status.
	pub status: EnvelopeStatus,
	/// Tool-specific payload.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<T>,
	/// Non-fatal warnings.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub warnings: Vec<ToolWarning>,
	/// Deterministic errors.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub errors: Vec<DeterministicError>,
	/// Common metadata.
	pub meta: ToolMeta,
}
