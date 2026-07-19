//! Canonical design constants for Alfred.
//!
//! This module centralizes the public tool surface and runtime limits so
//! other modules do not duplicate design truth.

/// Alfred package version.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Alfred tool schema version.
pub const TOOL_SCHEMA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum inline UTF-8 payload size in bytes.
pub const MAX_INLINE_UTF8_BYTES: usize = 1_048_576;

/// Maximum patch file count per call.
pub const MAX_PATCH_FILES_PER_CALL: usize = 128;

/// Maximum bulk operations per call.
pub const MAX_BULK_OPERATIONS_PER_CALL: usize = 256;

/// Maximum log records returned per call.
pub const MAX_LOG_RECORDS_PER_CALL: usize = 1_000;
