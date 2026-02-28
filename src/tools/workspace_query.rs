//! Workspace query tool group.

use std::fs;

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
	pub const NAMES: &'static [&'static str] =
		&["ls", "read_range", "file_stat", "grep", "search", "diff"];
}

/// Handles workspace-query tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"ls" => handle_ls(args, services).map(Some),
		"read_range" => handle_read_range(args, services).map(Some),
		"file_stat" => handle_file_stat(args, services).map(Some),
		"grep" => handle_grep(args, services).map(Some),
		"search" => handle_search(args, services).map(Some),
		"diff" => handle_diff(args, services).map(Some),
		_ => Ok(None),
	}
}

#[derive(Debug, Deserialize)]
struct LsArgs {
	path: Option<String>,
	recursive: Option<bool>,
	include_hidden: Option<bool>,
	include_dirs: Option<bool>,
	include_files: Option<bool>,
	cursor: Option<String>,
	limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ReadRangeArgs {
	path: String,
	start_line: usize,
	end_line: usize,
}

#[derive(Debug, Deserialize)]
struct FilePathArgs {
	path: String,
}

#[derive(Debug, Deserialize)]
struct GrepArgs {
	query: String,
	case_sensitive: Option<bool>,
	include_pattern: Option<String>,
	cursor: Option<String>,
	limit: Option<usize>,
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

#[derive(Debug, Deserialize)]
struct DiffArgs {
	a: DiffInput,
	b: DiffInput,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DiffInput {
	Text(String),
	Range {
		path: String,
		from: usize,
		to: usize,
	},
}

fn handle_ls(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
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

	let args = parse_args::<LsArgs>(args)?;
	let base = normalize_ls_base(args.path.as_deref().unwrap_or("."))?;
	let recursive = args.recursive.unwrap_or(false);
	let include_hidden = args.include_hidden.unwrap_or(false);
	let include_dirs = args.include_dirs.unwrap_or(true);
	let include_files = args.include_files.unwrap_or(true);
	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;

	let files = if include_files {
		services
			.indexer
			.list_files()
			.into_iter()
			.filter(|path| ls_path_allowed(path, base.as_str(), recursive, include_hidden, true))
			.collect::<Vec<_>>()
	} else {
		Vec::new()
	};

	let directories = if include_dirs {
		services
			.indexer
			.list_directories()
			.into_iter()
			.filter(|path| ls_path_allowed(path, base.as_str(), recursive, include_hidden, false))
			.collect::<Vec<_>>()
	} else {
		Vec::new()
	};

	let (files, file_cursor) = paginate(files, offset, limit);
	let (directories, directory_cursor) = paginate(directories, offset, limit);
	let next_cursor = directory_cursor.or(file_cursor);

	Ok(json!({
		"files": files,
		"directories": directories,
		"next_cursor": next_cursor,
	}))
}

fn handle_read_range(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<ReadRangeArgs>(args)?;
	let path = normalize_required_path(args.path.as_str())?;
	let text = services
		.indexer
		.read_range(path.as_str(), args.start_line, args.end_line)
		.map_err(map_indexer_error)?;

	Ok(json!({
		"path": path,
		"start_line": args.start_line,
		"end_line": args.end_line,
		"text": text,
	}))
}

fn handle_file_stat(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<FilePathArgs>(args)?;
	let path = normalize_required_path(args.path.as_str())?;
	let absolute_path = services.indexer.workspace_root().join(path.as_str());
	let metadata = fs::symlink_metadata(&absolute_path)
		.map_err(|_| AlfredError::NotFound(format!("path not found: {path}")))?;
	let file_type = metadata.file_type();

	let kind = if file_type.is_file() {
		"file"
	} else if file_type.is_dir() {
		"dir"
	} else if file_type.is_symlink() {
		"symlink"
	} else {
		"other"
	};

	let size_bytes = if file_type.is_file() {
		Some(metadata.len())
	} else {
		None
	};
	let modified_at = metadata.modified().ok().map(rfc3339_timestamp);

	Ok(json!({
		"path": path,
		"kind": kind,
		"size_bytes": size_bytes,
		"modified_at": modified_at,
	}))
}

fn handle_grep(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
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

	let args = parse_args::<GrepArgs>(args)?;
	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;

	let matches = services
		.indexer
		.grep_literal(args.query.as_str(), args.case_sensitive.unwrap_or(false));
	let matches = filter_matches(matches, args.include_pattern.as_deref(), None)?;
	let (matches, next_cursor) = paginate(matches, offset, limit);

	Ok(json!({
		"matches": matches,
		"next_cursor": next_cursor,
	}))
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

fn handle_diff(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<DiffArgs>(args)?;
	let left = resolve_diff_side(args.a, services)?;
	let right = resolve_diff_side(args.b, services)?;
	let diff = unified_diff(left.as_str(), right.as_str());

	Ok(json!({"diff": diff}))
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}

fn normalize_required_path(raw_path: &str) -> Result<String, AlfredError> {
	let path = normalize_workspace_path(raw_path)?;
	if path.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"path must not be empty".to_string(),
		));
	}

	Ok(path)
}

fn normalize_ls_base(raw_path: &str) -> Result<String, AlfredError> {
	if raw_path == "." {
		return Ok(String::new());
	}

	normalize_workspace_path(raw_path)
}

fn normalize_workspace_path(raw_path: &str) -> Result<String, AlfredError> {
	let raw = raw_path.trim().replace('\\', "/");
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

fn ls_path_allowed(
	path: &str,
	base: &str,
	recursive: bool,
	include_hidden: bool,
	include_file_exact: bool,
) -> bool {
	if !is_under_base(path, base) {
		return false;
	}
	if !include_hidden && is_hidden_path(path) {
		return false;
	}
	if recursive {
		return path != base;
	}

	if include_file_exact && path == base {
		return true;
	}
	is_direct_child(path, base)
}

fn is_under_base(path: &str, base: &str) -> bool {
	if base.is_empty() {
		return true;
	}

	path == base
		|| path
			.strip_prefix(base)
			.is_some_and(|suffix| suffix.starts_with('/'))
}

fn is_direct_child(path: &str, base: &str) -> bool {
	if base.is_empty() {
		return !path.contains('/');
	}

	path.strip_prefix(format!("{base}/").as_str())
		.is_some_and(|suffix| !suffix.contains('/'))
}

fn is_hidden_path(path: &str) -> bool {
	path.split('/').any(|segment| segment.starts_with('.'))
}

fn resolve_diff_side(input: DiffInput, services: &ServiceContainer) -> Result<String, AlfredError> {
	match input {
		DiffInput::Text(text) => Ok(text),
		DiffInput::Range { path, from, to } => {
			let path = normalize_required_path(path.as_str())?;
			services
				.indexer
				.read_range(path.as_str(), from, to)
				.map_err(map_indexer_error)
		}
	}
}

fn unified_diff(left: &str, right: &str) -> String {
	if left == right {
		return String::new();
	}

	let left_lines = left.lines().collect::<Vec<_>>();
	let right_lines = right.lines().collect::<Vec<_>>();
	let mut output = String::new();
	output.push_str("--- a\n+++ b\n");
	output.push_str(format!("@@ -1,{} +1,{} @@\n", left_lines.len(), right_lines.len()).as_str());
	for line in left_lines {
		output.push('-');
		output.push_str(line);
		output.push('\n');
	}
	for line in right_lines {
		output.push('+');
		output.push_str(line);
		output.push('\n');
	}

	output
}

fn map_indexer_error(error: anyhow::Error) -> AlfredError {
	let message = error.to_string();
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

fn rfc3339_timestamp(time: std::time::SystemTime) -> String {
	DateTime::<Utc>::from(time).to_rfc3339()
}
