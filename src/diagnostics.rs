//! Diagnostics models aligned to design schema.

use serde::{Deserialize, Serialize};

/// Severity levels for normalized diagnostics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
	Error,
	Warning,
	Info,
}

/// 1-indexed source range for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DiagnosticRange {
	pub start_line: usize,
	pub start_col: usize,
	pub end_line: usize,
	pub end_col: usize,
}

/// A single normalized diagnostic item.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DiagnosticItem {
	pub severity: DiagnosticSeverity,
	pub message: String,
	pub fingerprint: String,
	pub tool: Option<String>,
	pub task: Option<String>,
	pub path: Option<String>,
	pub range: Option<DiagnosticRange>,
	pub code: Option<String>,
}

/// Aggregate diagnostic summary.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DiagnosticSummary {
	pub errors: usize,
	pub warnings: usize,
	pub infos: usize,
}

/// Full diagnostics report container.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DiagnosticsReport {
	pub schema_version: String,
	pub diagnostics: Vec<DiagnosticItem>,
	pub summary: DiagnosticSummary,
}
