//! Deterministic error taxonomy types.

use serde_json::Value;
use thiserror::Error;

/// Canonical error kind values used in tool envelopes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
	InvalidArgument,
	PermissionDenied,
	WorkspaceBoundaryViolation,
	Conflict,
	NotFound,
	ToolUnavailable,
	ResourceExhausted,
	Timeout,
	Canceled,
	IoError,
	Internal,
}

/// Structured tool-level error payload.
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolError {
	pub kind: ErrorKind,
	pub message: String,
	pub retryable: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub details: Option<Value>,
}

/// Internal crate-level error enum.
#[derive(Debug, Error)]
pub enum AlfredError {
	#[error("invalid argument: {0}")]
	InvalidArgument(String),
	#[error("invalid argument: {message}")]
	InvalidArgumentWithDetails {
		message: String,
		details: Option<Value>,
	},
	#[error("permission denied: {0}")]
	PermissionDenied(String),
	#[error("workspace boundary violation: {0}")]
	WorkspaceBoundaryViolation(String),
	#[error("conflict: {0}")]
	Conflict(String),
	#[error("conflict: {message}")]
	ConflictWithDetails {
		message: String,
		details: Option<Value>,
	},
	#[error("not found: {0}")]
	NotFound(String),
	#[error("tool unavailable: {message}")]
	ToolUnavailable {
		message: String,
		details: Option<Value>,
	},
	#[error("resource exhausted: {0}")]
	ResourceExhausted(String),
	#[error("timeout: {0}")]
	Timeout(String),
	#[error("canceled: {0}")]
	Canceled(String),
	#[error("io error: {0}")]
	IoError(String),
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
				details: None,
			},
			AlfredError::InvalidArgumentWithDetails { message, details } => Self {
				kind: ErrorKind::InvalidArgument,
				message,
				retryable: false,
				details,
			},
			AlfredError::PermissionDenied(message) => Self {
				kind: ErrorKind::PermissionDenied,
				message,
				retryable: false,
				details: None,
			},
			AlfredError::WorkspaceBoundaryViolation(message) => Self {
				kind: ErrorKind::WorkspaceBoundaryViolation,
				message,
				retryable: false,
				details: None,
			},
			AlfredError::Conflict(message) => Self {
				kind: ErrorKind::Conflict,
				message,
				retryable: false,
				details: None,
			},
			AlfredError::ConflictWithDetails { message, details } => Self {
				kind: ErrorKind::Conflict,
				message,
				retryable: false,
				details,
			},
			AlfredError::NotFound(message) => Self {
				kind: ErrorKind::NotFound,
				message,
				retryable: false,
				details: None,
			},
			AlfredError::ToolUnavailable { message, details } => Self {
				kind: ErrorKind::ToolUnavailable,
				message,
				retryable: true,
				details,
			},
			AlfredError::ResourceExhausted(message) => Self {
				kind: ErrorKind::ResourceExhausted,
				message,
				retryable: true,
				details: None,
			},
			AlfredError::Timeout(message) => Self {
				kind: ErrorKind::Timeout,
				message,
				retryable: true,
				details: None,
			},
			AlfredError::Canceled(message) => Self {
				kind: ErrorKind::Canceled,
				message,
				retryable: true,
				details: None,
			},
			AlfredError::IoError(message) => Self {
				kind: ErrorKind::IoError,
				message,
				retryable: true,
				details: None,
			},
			AlfredError::Internal(message) => Self {
				kind: ErrorKind::Internal,
				message,
				retryable: true,
				details: None,
			},
		}
	}
}
