//! Workspace query tool group.

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::configuration::is_workspace_relative_path;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::services::indexer::SearchMatch;

const DEFAULT_LIMIT: usize = 100;

/// Index-backed workspace query tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct WorkspaceQueryTools;

impl WorkspaceQueryTools {
	pub const NAMES: &'static [&'static str] = &["search"];
}

/// Handles workspace-query tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"search" => handle_search(args, services).map(Some),
		_ => Ok(None),
	}
}

#[derive(Debug, Deserialize)]
struct SearchArgs {
	query: String,
	mode: Option<SearchMode>,
	case_sensitive: Option<bool>,
	full_text: Option<bool>,
	include_pattern: Option<String>,
	exclude_pattern: Option<String>,
	cursor: Option<String>,
	limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SearchMode {
	Literal,
	Regex,
}

fn handle_search(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	if !services.config.index_enabled {
		return Err(AlfredError::ToolUnavailable {
			message: "workspace index is disabled".to_string(),
			details: Some(json!({"reason": "index_disabled"})),
		});
	}
	if !services.indexer.is_ready() {
		return Err(AlfredError::ToolUnavailable {
			message: "workspace index is not ready".to_string(),
			details: Some(json!({"reason": "index_not_ready"})),
		});
	}

	let args = parse_args::<SearchArgs>(args)?;
	let _ = args.full_text.unwrap_or(false);
	let case_sensitive = args.case_sensitive.unwrap_or(false);
	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;
	let mode = args.mode.unwrap_or(SearchMode::Literal);

	let matches = match mode {
		SearchMode::Literal => services
			.indexer
			.grep_literal(args.query.as_str(), case_sensitive),
		SearchMode::Regex => services
			.indexer
			.search_regex(args.query.as_str(), case_sensitive)
			.map_err(map_indexer_error)?,
	};

	let matches = filter_matches(
		matches,
		args.include_pattern.as_deref(),
		args.exclude_pattern.as_deref(),
	)?;
	let (matches, next_cursor) = paginate(matches, offset, limit);

	Ok(json!({
		"matches": matches,
		"next_cursor": next_cursor,
	}))
}
fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}

pub(crate) fn normalize_required_path(raw_path: &str) -> Result<String, AlfredError> {
	let path = normalize_workspace_path(raw_path)?;
	if path.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"path must not be empty".to_string(),
		));
	}

	Ok(path)
}

pub(crate) fn normalize_workspace_path(raw_path: &str) -> Result<String, AlfredError> {
	let raw = crate::path_encoding::normalize_inbound_separators(raw_path.trim());
	if raw.is_empty() {
		return Ok(String::new());
	}

	if is_absolute_path(raw.as_str()) {
		return Err(AlfredError::WorkspaceBoundaryViolation(format!(
			"path escapes workspace boundary: {raw_path}"
		)));
	}

	let mut normalized_segments = Vec::new();
	for segment in raw.split('/') {
		if segment.is_empty() || segment == "." {
			continue;
		}

		if segment == ".." {
			if normalized_segments.pop().is_none() {
				return Err(AlfredError::WorkspaceBoundaryViolation(format!(
					"path escapes workspace boundary: {raw_path}"
				)));
			}
			continue;
		}

		normalized_segments.push(segment);
	}

	let normalized = normalized_segments.join("/");
	if normalized.is_empty() {
		return Ok(String::new());
	}

	if !is_workspace_relative_path(normalized.as_str()) {
		return Err(AlfredError::WorkspaceBoundaryViolation(format!(
			"path escapes workspace boundary: {raw_path}"
		)));
	}

	Ok(normalized)
}

fn is_absolute_path(path: &str) -> bool {
	if path.starts_with('/') {
		return true;
	}

	if path.len() >= 3 {
		let bytes = path.as_bytes();
		if bytes[1] == b':' && bytes[2] == b'/' {
			return true;
		}
	}

	false
}

pub(crate) fn parse_pagination(
	cursor: Option<&str>,
	limit: Option<usize>,
) -> Result<(usize, usize), AlfredError> {
	let offset = match cursor {
		Some(token) => token.parse::<usize>().map_err(|_| {
			AlfredError::InvalidArgument(format!("cursor is not a valid index: {token}"))
		})?,
		None => 0,
	};

	let limit = limit.unwrap_or(DEFAULT_LIMIT);
	if limit == 0 {
		return Err(AlfredError::InvalidArgument(
			"limit must be greater than 0".to_string(),
		));
	}

	Ok((offset, limit))
}

pub(crate) fn paginate<T>(items: Vec<T>, offset: usize, limit: usize) -> (Vec<T>, Option<String>) {
	if offset >= items.len() {
		return (Vec::new(), None);
	}

	let end = std::cmp::min(items.len(), offset + limit);
	let next_cursor = if end < items.len() {
		Some(end.to_string())
	} else {
		None
	};

	(
		items.into_iter().skip(offset).take(limit).collect(),
		next_cursor,
	)
}

fn filter_matches(
	matches: Vec<SearchMatch>,
	include_pattern: Option<&str>,
	exclude_pattern: Option<&str>,
) -> Result<Vec<SearchMatch>, AlfredError> {
	let include = compile_glob_pattern(include_pattern)?;
	let exclude = compile_glob_pattern(exclude_pattern)?;

	Ok(matches
		.into_iter()
		.filter(|item| {
			let included = include
				.as_ref()
				.map(|pattern| pattern.is_match(item.path.as_str()))
				.unwrap_or(true);
			let excluded = exclude
				.as_ref()
				.map(|pattern| pattern.is_match(item.path.as_str()))
				.unwrap_or(false);
			included && !excluded
		})
		.collect())
}

fn compile_glob_pattern(pattern: Option<&str>) -> Result<Option<Regex>, AlfredError> {
	let Some(pattern) = pattern else {
		return Ok(None);
	};

	let regex_pattern = pattern
		.chars()
		.map(|ch| match ch {
			'*' => ".*".to_string(),
			'?' => ".".to_string(),
			other => regex::escape(other.to_string().as_str()),
		})
		.collect::<String>();

	Regex::new(format!("^{regex_pattern}$").as_str())
		.map(Some)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid glob pattern: {error}")))
}

fn map_indexer_error(error: anyhow::Error) -> AlfredError {
	let message = error.to_string();
	if message.contains(crate::workspace_boundary::SYMLINK_JUNCTION_ESCAPE_MESSAGE) {
		return AlfredError::PermissionDenied(
			crate::workspace_boundary::SYMLINK_JUNCTION_ESCAPE_MESSAGE.to_string(),
		);
	}
	if message.contains("invalid regex")
		|| message.contains("line numbers")
		|| message.contains("end_line must be greater than or equal to start_line")
		|| message.contains("requested line range is out of bounds")
		|| message.contains("file is not available as UTF-8 text")
	{
		return AlfredError::InvalidArgument(message);
	}
	if message.contains("not found") {
		return AlfredError::NotFound(message);
	}
	if message.contains("io error") || message.contains("I/O") {
		return AlfredError::IoError(message);
	}

	AlfredError::Internal(message)
}

pub(crate) fn rfc3339_timestamp(time: std::time::SystemTime) -> String {
	DateTime::<Utc>::from(time).to_rfc3339()
}
