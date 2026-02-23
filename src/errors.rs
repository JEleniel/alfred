//! Deterministic error taxonomy types.

use thiserror::Error;

/// Canonical error kind values used in tool envelopes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
	InvalidArgument,
	PermissionDenied,
	Conflict,
	NotFound,
	ToolUnavailable,
	ResourceExhausted,
	Timeout,
	Internal,
}

/// Structured tool-level error payload.
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolError {
	pub kind: ErrorKind,
	pub message: String,
	pub retryable: bool,
}

/// Internal crate-level error enum.
#[derive(Debug, Error)]
pub enum AlfredError {
	#[error("invalid argument: {0}")]
	InvalidArgument(String),
	#[error("permission denied: {0}")]
	PermissionDenied(String),
	#[error("conflict: {0}")]
	Conflict(String),
	#[error("not found: {0}")]
	NotFound(String),
	#[error("tool unavailable: {0}")]
	ToolUnavailable(String),
	#[error("resource exhausted: {0}")]
	ResourceExhausted(String),
	#[error("timeout: {0}")]
	Timeout(String),
	#[error("internal: {0}")]
	Internal(String),
}

impl From<AlfredError> for ToolError {
	fn from(value: AlfredError) -> Self {
		match value {
			AlfredError::InvalidArgument(message) => Self {
				kind: ErrorKind::InvalidArgument,
				message,
				retryable: false,
			},
			AlfredError::PermissionDenied(message) => Self {
				kind: ErrorKind::PermissionDenied,
				message,
				retryable: false,
			},
			AlfredError::Conflict(message) => Self {
				kind: ErrorKind::Conflict,
				message,
				retryable: true,
			},
			AlfredError::NotFound(message) => Self {
				kind: ErrorKind::NotFound,
				message,
				retryable: false,
			},
			AlfredError::ToolUnavailable(message) => Self {
				kind: ErrorKind::ToolUnavailable,
				message,
				retryable: true,
			},
			AlfredError::ResourceExhausted(message) => Self {
				kind: ErrorKind::ResourceExhausted,
				message,
				retryable: true,
			},
			AlfredError::Timeout(message) => Self {
				kind: ErrorKind::Timeout,
				message,
				retryable: true,
			},
			AlfredError::Internal(message) => Self {
				kind: ErrorKind::Internal,
				message,
				retryable: false,
			},
		}
	}
}
