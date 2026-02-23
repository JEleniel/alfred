//! Log tool group.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::configuration::is_workspace_relative_path;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;

const DEFAULT_LIMIT: usize = 100;
const ALLOWED_LEVELS: &[&str] = &["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];

/// Log tailing and search tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct LogTools;

impl LogTools {
	pub const NAMES: &'static [&'static str] = &["log_search"];
}

#[derive(Debug, Deserialize)]
struct LogSearchArgs {
	path: Option<String>,
	query: String,
	level: Option<String>,
	source_prefix: Option<String>,
	cursor: Option<String>,
	limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogRecord {
	timestamp: String,
	level: String,
	message: String,
	source: String,
	#[serde(default)]
	extra: BTreeMap<String, String>,
}

/// Handles log tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"log_search" => handle_log_search(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_log_search(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<LogSearchArgs>(args)?;
	let level_filter = normalize_level(args.level.as_deref())?;
	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;
	let path = resolve_log_path(args.path.as_deref(), services)?;
	let query = args.query.to_lowercase();

	let matches = read_filtered_records(
		path.as_path(),
		query.as_str(),
		level_filter.as_deref(),
		args.source_prefix.as_deref(),
	)?;
	let (matches, next_cursor) = paginate(matches, offset, limit);

	Ok(json!({
		"matches": matches,
		"next_cursor": next_cursor,
	}))
}

fn resolve_log_path(
	path: Option<&str>,
	services: &ServiceContainer,
) -> Result<PathBuf, AlfredError> {
	let default_path = services.logs.log_path().to_path_buf();
	let Some(raw) = path else {
		return Ok(default_path);
	};

	let raw = raw.trim();
	if raw.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"path must not be empty".to_string(),
		));
	}

	let normalized = raw.replace('\\', "/");
	if is_absolute_path(normalized.as_str()) {
		let absolute = PathBuf::from(&normalized);
		if absolute
			.components()
			.any(|component| matches!(component, Component::ParentDir))
		{
			return Err(AlfredError::PermissionDenied(format!(
				"absolute log path cannot contain parent traversal: {raw}"
			)));
		}

		let allowed_root = default_path.parent().unwrap_or_else(|| Path::new(""));
		if !absolute.starts_with(allowed_root) {
			return Err(AlfredError::PermissionDenied(format!(
				"absolute log path is outside Alfred log directory: {raw}"
			)));
		}

		return Ok(absolute);
	}

	let relative = normalize_workspace_relative_path(normalized.as_str())?;
	Ok(services.config.workspace_root.join(relative))
}

fn normalize_workspace_relative_path(raw_path: &str) -> Result<String, AlfredError> {
	let raw = raw_path.trim().replace('\\', "/");
	if raw.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"path must not be empty".to_string(),
		));
	}

	let mut segments = Vec::new();
	for segment in raw.split('/') {
		if segment.is_empty() || segment == "." {
			continue;
		}
		if segment == ".." {
			if segments.pop().is_none() {
				return Err(AlfredError::WorkspaceBoundaryViolation(format!(
					"path escapes workspace boundary: {raw_path}"
				)));
			}
			continue;
		}
		segments.push(segment);
	}

	let normalized = segments.join("/");
	if normalized.is_empty() || !is_workspace_relative_path(normalized.as_str()) {
		return Err(AlfredError::WorkspaceBoundaryViolation(format!(
			"path escapes workspace boundary: {raw_path}"
		)));
	}

	Ok(normalized)
}

fn read_filtered_records(
	path: &Path,
	query: &str,
	level: Option<&str>,
	source_prefix: Option<&str>,
) -> Result<Vec<Value>, AlfredError> {
	let file = File::open(path).map_err(|error| {
		if error.kind() == std::io::ErrorKind::NotFound {
			return AlfredError::NotFound(format!("log file not found: {}", path.display()));
		}

		AlfredError::IoError(format!(
			"failed to open log file {}: {error}",
			path.display()
		))
	})?;
	let reader = BufReader::new(file);

	let mut matches = Vec::new();
	for (line_number, line) in reader.lines().enumerate() {
		let line = line.map_err(|error| {
			AlfredError::IoError(format!(
				"failed to read log file {}: {error}",
				path.display()
			))
		})?;
		if line.trim().is_empty() {
			continue;
		}

		let record = serde_json::from_str::<LogRecord>(line.as_str()).map_err(|error| {
			AlfredError::InvalidArgument(format!(
				"log file contains invalid NDJSON at line {}: {error}",
				line_number + 1
			))
		})?;

		if !record_matches(&record, query, level, source_prefix) {
			continue;
		}

		let value = serde_json::to_value(record).map_err(|error| {
			AlfredError::Internal(format!("failed to serialize log record: {error}"))
		})?;
		matches.push(value);
	}

	Ok(matches)
}

fn record_matches(
	record: &LogRecord,
	query: &str,
	level: Option<&str>,
	source_prefix: Option<&str>,
) -> bool {
	if let Some(level_filter) = level
		&& !record.level.eq_ignore_ascii_case(level_filter)
	{
		return false;
	}

	if let Some(prefix) = source_prefix
		&& !record.source.starts_with(prefix)
	{
		return false;
	}

	if query.is_empty() {
		return true;
	}

	let timestamp = record.timestamp.to_lowercase();
	let level = record.level.to_lowercase();
	let message = record.message.to_lowercase();
	let source = record.source.to_lowercase();
	if timestamp.contains(query)
		|| level.contains(query)
		|| message.contains(query)
		|| source.contains(query)
	{
		return true;
	}

	record.extra.iter().any(|(key, value)| {
		key.to_lowercase().contains(query) || value.to_lowercase().contains(query)
	})
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}

fn parse_pagination(
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

fn paginate<T>(items: Vec<T>, offset: usize, limit: usize) -> (Vec<T>, Option<String>) {
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

fn normalize_level(level: Option<&str>) -> Result<Option<String>, AlfredError> {
	let Some(level) = level else {
		return Ok(None);
	};

	let normalized = level.trim().to_ascii_uppercase();
	if normalized.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"level must not be empty".to_string(),
		));
	}
	if !ALLOWED_LEVELS.contains(&normalized.as_str()) {
		return Err(AlfredError::InvalidArgument(format!(
			"level must be one of TRACE, DEBUG, INFO, WARN, ERROR: {level}"
		)));
	}

	Ok(Some(normalized))
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
