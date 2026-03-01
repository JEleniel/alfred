//! Log tool group.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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
	pub const NAMES: &'static [&'static str] = &["logs"];
}

#[derive(Debug, Deserialize)]
struct LogsArgs {
	operation: String,
	#[serde(default)]
	args: Value,
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

#[derive(Debug, Deserialize)]
struct LogTailArgs {
	path: Option<String>,
	level_min: Option<String>,
	source_prefix: Option<String>,
	cursor: Option<String>,
	limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct LogFollowArgs {
	#[serde(default)]
	stop: bool,
	path: Option<String>,
	level_min: Option<String>,
	source_prefix: Option<String>,
	#[serde(default)]
	tail: usize,
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
		"logs" => handle_logs(args, services).map(Some),
		"log_search" => handle_log_search(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_logs(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<LogsArgs>(args)?;
	let operation = args.operation.trim().to_string();
	if operation.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"operation must not be empty".to_string(),
		));
	}

	let result = match operation.as_str() {
		"search" => handle_log_search(args.args, services)?,
		"tail" => handle_log_tail(args.args, services)?,
		"follow" => handle_log_follow(args.args, services)?,
		other => {
			return Err(AlfredError::InvalidArgument(format!(
				"operation must be one of search, tail, follow: {other}"
			)));
		}
	};

	Ok(json!({
		"operation": operation,
		"result": result,
	}))
}

fn handle_log_search(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<LogSearchArgs>(args)?;
	let level_filter = normalize_level_exact(args.level.as_deref())?;
	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;
	let path = resolve_log_path(args.path.as_deref(), services)?;
	let query = args.query.to_lowercase();

	let matches = read_search_matches(
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

fn handle_log_tail(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<LogTailArgs>(args)?;
	let level_min = normalize_level_min(args.level_min.as_deref())?;
	let (offset_from_end, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;
	let path = resolve_log_path(args.path.as_deref(), services)?;

	let records = read_tail_records(
		path.as_path(),
		level_min.as_deref(),
		args.source_prefix.as_deref(),
	)?;
	let (records, next_cursor) = paginate_from_end(records, offset_from_end, limit);

	Ok(json!({
		"records": records,
		"next_cursor": next_cursor,
	}))
}

fn handle_log_follow(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<LogFollowArgs>(args)?;
	if args.stop {
		services.logs.stop_follow()?;
		return Ok(json!({"records": [], "stopped": true}));
	}

	services.logs.try_start_follow()?;
	let level_min = normalize_level_min(args.level_min.as_deref())?;
	let path = resolve_log_path(args.path.as_deref(), services)?;

	let mut records = read_tail_records(
		path.as_path(),
		level_min.as_deref(),
		args.source_prefix.as_deref(),
	)?;
	if args.tail > 0 && records.len() > args.tail {
		records = records.split_off(records.len() - args.tail);
	}

	Ok(json!({"records": records}))
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

	let normalized = crate::path_encoding::normalize_inbound_separators(raw);
	if is_absolute_path(normalized.as_str()) {
		return Err(AlfredError::InvalidArgument(format!(
			"absolute paths are not allowed for logs.path: {raw}"
		)));
	}
	if !is_workspace_relative_path(normalized.as_str()) {
		return Err(AlfredError::WorkspaceBoundaryViolation(format!(
			"path must be workspace-relative and must not contain '..': {raw}"
		)));
	}

	let candidate = services.config.workspace_root.join(normalized);
	let resolved = crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
		services.config.workspace_root.as_path(),
		candidate.as_path(),
	)?;
	Ok(resolved.unwrap_or(candidate))
}

fn read_records(path: &Path) -> Result<Vec<LogRecord>, AlfredError> {
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

	let mut records = Vec::new();
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
		records.push(record);
	}

	Ok(records)
}

fn read_search_matches(
	path: &Path,
	query: &str,
	level: Option<&str>,
	source_prefix: Option<&str>,
) -> Result<Vec<Value>, AlfredError> {
	let records = read_records(path)?;
	let mut matches = Vec::new();
	for record in records {
		if !record_matches_query(&record, query, level, source_prefix) {
			continue;
		}

		let value = serde_json::to_value(record).map_err(|error| {
			AlfredError::Internal(format!("failed to serialize log record: {error}"))
		})?;
		matches.push(value);
	}
	Ok(matches)
}

fn read_tail_records(
	path: &Path,
	level_min: Option<&str>,
	source_prefix: Option<&str>,
) -> Result<Vec<Value>, AlfredError> {
	let level_min = level_min.and_then(level_rank);
	let records = read_records(path)?;
	let mut matches = Vec::new();
	for record in records {
		if !record_matches_tail(&record, level_min, source_prefix) {
			continue;
		}

		let value = serde_json::to_value(record).map_err(|error| {
			AlfredError::Internal(format!("failed to serialize log record: {error}"))
		})?;
		matches.push(value);
	}
	Ok(matches)
}

fn record_matches_query(
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

fn record_matches_tail(
	record: &LogRecord,
	level_min: Option<usize>,
	source_prefix: Option<&str>,
) -> bool {
	if let Some(prefix) = source_prefix
		&& !record.source.starts_with(prefix)
	{
		return false;
	}

	if let Some(level_min) = level_min {
		let Some(rank) = level_rank(record.level.as_str()) else {
			return false;
		};
		if rank < level_min {
			return false;
		}
	}

	true
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

fn paginate_from_end(
	items: Vec<Value>,
	offset_from_end: usize,
	limit: usize,
) -> (Vec<Value>, Option<String>) {
	if items.is_empty() {
		return (Vec::new(), None);
	}
	if offset_from_end >= items.len() {
		return (Vec::new(), None);
	}

	let end = items.len() - offset_from_end;
	let start = end.saturating_sub(limit);
	let next_cursor = if start > 0 {
		Some((offset_from_end + (end - start)).to_string())
	} else {
		None
	};
	(items[start..end].to_vec(), next_cursor)
}

fn normalize_level_exact(level: Option<&str>) -> Result<Option<String>, AlfredError> {
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

fn normalize_level_min(level: Option<&str>) -> Result<Option<String>, AlfredError> {
	normalize_level_exact(level)
}

fn level_rank(level: &str) -> Option<usize> {
	let normalized = level.trim().to_ascii_uppercase();
	ALLOWED_LEVELS
		.iter()
		.position(|candidate| candidate.eq(&normalized.as_str()))
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
