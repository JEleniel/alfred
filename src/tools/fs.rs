//! Consolidated filesystem operations tool.

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::services::job_manager::{FsBulkOperation, FsBulkState};
use crate::tools::ToolCallResult;
use crate::tools::workspace_query;

/// Consolidated filesystem operations tool.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsTools;

impl FsTools {
	pub const NAMES: &'static [&'static str] = &["fs"];
}

#[derive(Debug, Deserialize)]
struct FsArgs {
	operation: String,
	#[serde(default)]
	args: Value,
	dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CreateFileArgs {
	path: String,
	content: String,
}

#[derive(Debug, Deserialize)]
struct AppendFileArgs {
	path: String,
	content: String,
}

#[derive(Debug, Deserialize)]
struct DeletePathArgs {
	path: String,
}

#[derive(Debug, Deserialize)]
struct CreateDirArgs {
	path: String,
	parents: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SearchArgs {
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
struct StatArgs {
	path: String,
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

/// Handles filesystem tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<ToolCallResult>, AlfredError> {
	match name {
		"fs" => handle_fs(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_fs(args: Value, services: &ServiceContainer) -> Result<ToolCallResult, AlfredError> {
	let args = parse_args::<FsArgs>(args)?;
	let operation = args.operation.trim();
	if operation.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"operation must not be empty".to_string(),
		));
	}

	let dry_run = args.dry_run.unwrap_or(true);
	let result = match operation {
		"search" => handle_search(args.args, services)?,
		"read_range" => handle_read_range(args.args, services)?,
		"stat" => handle_stat(args.args, services)?,
		"diff" => handle_diff(args.args, services)?,
		"create_file" => handle_create_file(args.args, dry_run, services)?,
		"append_file" => handle_append_file(args.args, dry_run, services)?,
		"delete_file" => handle_delete_file(args.args, dry_run, services)?,
		"create_dir" => handle_create_dir(args.args, dry_run, services)?,
		"delete_dir" => handle_delete_dir(args.args, dry_run, services)?,
		"bulk" => {
			return handle_bulk(args.args, dry_run, services);
		}
		other => {
			return Err(AlfredError::InvalidArgument(format!(
				"unsupported fs operation: {other}"
			)));
		}
	};

	Ok(ToolCallResult::ok(json!({
		"operation": operation,
		"result": result,
	})))
}

#[derive(Debug, Deserialize)]
struct BulkArgs {
	mode: String,
	operation_id: Option<String>,
	run_in_background: Option<bool>,
	operations: Option<Vec<BulkOperationArgs>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BulkOperationArgs {
	Move {
		from: String,
		to: String,
		overwrite: Option<bool>,
		create_parents: Option<bool>,
	},
	Copy {
		from: String,
		to: String,
		overwrite: Option<bool>,
		create_parents: Option<bool>,
	},
	Delete {
		path: String,
		recursive: Option<bool>,
	},
}

fn handle_bulk(
	args: Value,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<ToolCallResult, AlfredError> {
	let args = parse_args::<BulkArgs>(args)?;
	let mode = args.mode.trim();
	if mode.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"fs.bulk.args.mode must not be empty".to_string(),
		));
	}

	if args.run_in_background.is_some() && mode != "execute" {
		return Err(AlfredError::InvalidArgument(
			"fs.bulk.args.run_in_background is only valid with mode=execute".to_string(),
		));
	}

	match mode {
		"execute" => handle_bulk_execute(args, dry_run, services),
		"status" => handle_bulk_status(args, services),
		"cancel" => handle_bulk_cancel(args, services),
		other => Err(AlfredError::InvalidArgument(format!(
			"unsupported fs.bulk mode: {other}"
		))),
	}
}

fn handle_bulk_execute(
	args: BulkArgs,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<ToolCallResult, AlfredError> {
	let run_in_background = args.run_in_background.unwrap_or(false);
	if run_in_background && dry_run {
		return Err(AlfredError::InvalidArgument(
			"fs.bulk execute with run_in_background requires dry_run=false".to_string(),
		));
	}

	let raw_operations = args.operations.ok_or_else(|| {
		AlfredError::InvalidArgument("fs.bulk.operations is required for mode=execute".to_string())
	})?;
	if raw_operations.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"fs.bulk.operations must not be empty".to_string(),
		));
	}

	let operations = raw_operations
		.into_iter()
		.map(parse_bulk_operation)
		.collect::<Result<Vec<_>, _>>()?;

	let workspace_root = services.config.workspace_root.clone();
	if run_in_background {
		let status = services
			.jobs
			.start_fs_bulk_background(workspace_root, operations)?;
		let state = match status.state {
			FsBulkState::Queued => "queued",
			FsBulkState::Running => "running",
			_ => "queued",
		};
		return Ok(ToolCallResult::pending(
			status.operation_id.clone(),
			json!({
				"operation_id": status.operation_id,
				"state": state,
				"poll_with": "fs",
			}),
		));
	}

	let status =
		services
			.jobs
			.execute_fs_bulk_sync(workspace_root.as_path(), operations, dry_run)?;
	Ok(ToolCallResult::ok(json!({
		"operation": "bulk",
		"result": status,
	})))
}

fn handle_bulk_status(
	args: BulkArgs,
	services: &ServiceContainer,
) -> Result<ToolCallResult, AlfredError> {
	let operation_id = args.operation_id.as_deref().unwrap_or("").trim();
	if operation_id.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"fs.bulk.operation_id is required for mode=status".to_string(),
		));
	}
	let status = services.jobs.fs_bulk_status(operation_id)?;
	Ok(ToolCallResult::ok(json!({
		"operation": "bulk",
		"result": status,
	})))
}

fn handle_bulk_cancel(
	args: BulkArgs,
	services: &ServiceContainer,
) -> Result<ToolCallResult, AlfredError> {
	let operation_id = args.operation_id.as_deref().unwrap_or("").trim();
	if operation_id.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"fs.bulk.operation_id is required for mode=cancel".to_string(),
		));
	}
	let status = services.jobs.fs_bulk_cancel(operation_id)?;
	Ok(ToolCallResult::ok(json!({
		"operation": "bulk",
		"result": status,
	})))
}

fn parse_bulk_operation(operation: BulkOperationArgs) -> Result<FsBulkOperation, AlfredError> {
	match operation {
		BulkOperationArgs::Move {
			from,
			to,
			overwrite,
			create_parents,
		} => Ok(FsBulkOperation::Move {
			from: workspace_query::normalize_required_path(from.as_str())?,
			to: workspace_query::normalize_required_path(to.as_str())?,
			overwrite: overwrite.unwrap_or(false),
			create_parents: create_parents.unwrap_or(false),
		}),
		BulkOperationArgs::Copy {
			from,
			to,
			overwrite,
			create_parents,
		} => Ok(FsBulkOperation::Copy {
			from: workspace_query::normalize_required_path(from.as_str())?,
			to: workspace_query::normalize_required_path(to.as_str())?,
			overwrite: overwrite.unwrap_or(false),
			create_parents: create_parents.unwrap_or(false),
		}),
		BulkOperationArgs::Delete { path, recursive } => Ok(FsBulkOperation::Delete {
			path: workspace_query::normalize_required_path(path.as_str())?,
			recursive: recursive.unwrap_or(false),
		}),
	}
}

fn handle_search(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<SearchArgs>(args)?;
	let base_raw = args.path.as_deref().unwrap_or(".");
	let base = workspace_query::normalize_workspace_path(base_raw)?;
	let recursive = args.recursive.unwrap_or(false);
	let include_hidden = args.include_hidden.unwrap_or(false);
	let include_dirs = args.include_dirs.unwrap_or(true);
	let include_files = args.include_files.unwrap_or(true);
	let (offset, limit) = workspace_query::parse_pagination(args.cursor.as_deref(), args.limit)?;

	let workspace_root = services.config.workspace_root.as_path();
	let absolute_base =
		crate::path_encoding::resolve_workspace_relative_path(workspace_root, &base);
	let resolved_base = crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
		workspace_root,
		absolute_base.as_path(),
	)?
	.ok_or_else(|| AlfredError::NotFound(format!("path not found: {}", display_rooted(&base))))?;
	let metadata = fs::symlink_metadata(resolved_base.as_path())
		.map_err(|_| AlfredError::NotFound(format!("path not found: {}", display_rooted(&base))))?;
	if !metadata.is_dir() {
		return Err(AlfredError::InvalidArgument(format!(
			"path is not a directory: {}",
			display_rooted(&base)
		)));
	}

	let mut files = Vec::new();
	let mut directories = Vec::new();
	let mut context = SearchCollectContext {
		recursive,
		include_hidden,
		include_dirs,
		include_files,
		directories: &mut directories,
		files: &mut files,
	};
	collect_search_entries(resolved_base.as_path(), base.as_str(), &mut context)?;

	sort_paths_case_insensitive(&mut files);
	sort_paths_case_insensitive(&mut directories);

	let (files, file_cursor) = workspace_query::paginate(files, offset, limit);
	let (directories, directory_cursor) = workspace_query::paginate(directories, offset, limit);
	let next_cursor = directory_cursor.or(file_cursor);

	Ok(json!({
		"files": files,
		"directories": directories,
		"next_cursor": next_cursor,
	}))
}

struct SearchCollectContext<'a> {
	recursive: bool,
	include_hidden: bool,
	include_dirs: bool,
	include_files: bool,
	directories: &'a mut Vec<String>,
	files: &'a mut Vec<String>,
}

fn collect_search_entries(
	absolute_dir: &Path,
	relative_dir: &str,
	context: &mut SearchCollectContext<'_>,
) -> Result<(), AlfredError> {
	let entries = fs::read_dir(absolute_dir).map_err(|error| {
		AlfredError::IoError(format!(
			"failed to read directory {}: {error}",
			absolute_dir.display()
		))
	})?;

	for entry in entries {
		let entry = entry.map_err(|error| {
			AlfredError::IoError(format!(
				"failed to read directory entry {}: {error}",
				absolute_dir.display()
			))
		})?;
		let rendered_name = crate::path_encoding::render_component(entry.file_name().as_os_str());
		if !context.include_hidden && rendered_name.text.starts_with('.') {
			continue;
		}
		let child_relative = join_relative(relative_dir, rendered_name.text.as_str());
		let child_path = entry.path();
		let file_type = entry.file_type().map_err(|error| {
			AlfredError::IoError(format!(
				"failed to stat directory entry {}: {error}",
				child_path.display()
			))
		})?;

		if file_type.is_dir() {
			if context.include_dirs {
				context.directories.push(child_relative.clone());
			}
			if context.recursive && !file_type.is_symlink() {
				collect_search_entries(child_path.as_path(), child_relative.as_str(), context)?;
			}
		} else if context.include_files {
			context.files.push(child_relative);
		}
	}

	Ok(())
}

fn handle_read_range(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<ReadRangeArgs>(args)?;
	let path = workspace_query::normalize_required_path(args.path.as_str())?;
	let text = read_range_from_disk(path.as_str(), args.start_line, args.end_line, services)?;

	Ok(json!({
		"path": path,
		"start_line": args.start_line,
		"end_line": args.end_line,
		"text": text,
	}))
}

fn read_range_from_disk(
	path: &str,
	start_line: usize,
	end_line: usize,
	services: &ServiceContainer,
) -> Result<String, AlfredError> {
	if start_line == 0 || end_line == 0 {
		return Err(AlfredError::InvalidArgument(
			"line numbers must be 1-indexed".to_string(),
		));
	}
	if end_line < start_line {
		return Err(AlfredError::InvalidArgument(
			"end_line must be greater than or equal to start_line".to_string(),
		));
	}

	let workspace_root = services.config.workspace_root.as_path();
	let absolute_path = crate::path_encoding::resolve_workspace_relative_path(workspace_root, path);
	let absolute_path =
		match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
			workspace_root,
			absolute_path.as_path(),
		) {
			Ok(Some(resolved)) => resolved,
			Ok(None) => absolute_path,
			Err(error) => return Err(error),
		};

	let file = File::open(absolute_path.as_path()).map_err(|error| {
		if error.kind() == std::io::ErrorKind::NotFound {
			return AlfredError::NotFound(format!("file not found: {path}"));
		}
		AlfredError::IoError(format!("io error reading file: {path}"))
	})?;
	let mut reader = BufReader::new(file);
	let mut output = String::new();
	let mut buffer = String::new();
	let mut current_line = 0usize;
	while current_line < end_line {
		buffer.clear();
		match reader.read_line(&mut buffer) {
			Ok(0) => break,
			Ok(_) => {}
			Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
				return Err(AlfredError::InvalidArgument(format!(
					"file is not available as UTF-8 text: {path}"
				)));
			}
			Err(_) => {
				return Err(AlfredError::IoError(format!(
					"io error reading file: {path}"
				)));
			}
		}

		current_line += 1;
		if current_line < start_line {
			continue;
		}
		push_line(&mut output, buffer.trim_end_matches(['\n', '\r']));
	}

	if end_line > current_line {
		return Err(AlfredError::InvalidArgument(format!(
			"requested line range is out of bounds for file: {path}"
		)));
	}

	Ok(output)
}

fn push_line(output: &mut String, line: &str) {
	if !output.is_empty() {
		output.push('\n');
	}
	output.push_str(line);
}

fn handle_stat(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<StatArgs>(args)?;
	let path = workspace_query::normalize_required_path(args.path.as_str())?;

	let workspace_root = services.config.workspace_root.as_path();
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, &path);
	let metadata = fs::symlink_metadata(candidate.as_path())
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

	match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	) {
		Ok(_) => {}
		Err(error) => {
			if matches!(error, AlfredError::PermissionDenied(_)) {
				return Err(error);
			}
			if !file_type.is_symlink() {
				return Err(error);
			}
			// Broken symlinks do not permit boundary escapes; preserve the observable metadata.
		}
	}

	let size_bytes = if file_type.is_file() {
		Some(metadata.len())
	} else {
		None
	};
	let modified_at = metadata
		.modified()
		.ok()
		.map(workspace_query::rfc3339_timestamp);

	Ok(json!({
		"path": path,
		"kind": kind,
		"size_bytes": size_bytes,
		"modified_at": modified_at,
	}))
}

fn handle_diff(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<DiffArgs>(args)?;
	let left = resolve_diff_side(args.a, services)?;
	let right = resolve_diff_side(args.b, services)?;
	let diff = unified_diff(left.as_str(), right.as_str());
	Ok(json!({"diff": diff}))
}

fn resolve_diff_side(input: DiffInput, services: &ServiceContainer) -> Result<String, AlfredError> {
	match input {
		DiffInput::Text(text) => Ok(text),
		DiffInput::Range { path, from, to } => {
			let path = workspace_query::normalize_required_path(path.as_str())?;
			read_range_from_disk(path.as_str(), from, to, services)
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

fn sort_paths_case_insensitive(values: &mut [String]) {
	values.sort_by(|left, right| {
		left.to_lowercase()
			.cmp(&right.to_lowercase())
			.then_with(|| left.cmp(right))
	});
}

fn join_relative(base: &str, name: &str) -> String {
	if base.is_empty() {
		name.to_string()
	} else {
		format!("{base}/{name}")
	}
}

fn display_rooted(path: &str) -> &str {
	if path.is_empty() { "." } else { path }
}

fn handle_create_file(
	args: Value,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<Value, AlfredError> {
	let args = parse_args::<CreateFileArgs>(args)?;
	let path = workspace_query::normalize_required_path(args.path.as_str())?;
	let bytes_written = args.content.len() as u64;

	let workspace_root = services.config.workspace_root.as_path();
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, &path);
	let write_target = crate::workspace_boundary::resolve_write_target_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	)?;

	match fs::symlink_metadata(write_target.as_path()) {
		Ok(metadata) => {
			if metadata.is_dir() {
				return Err(AlfredError::Conflict(format!(
					"path already exists as a directory: {path}"
				)));
			}
			return Err(AlfredError::Conflict(format!(
				"file already exists: {path}"
			)));
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
		Err(error) => {
			return Err(AlfredError::IoError(format!(
				"failed to stat {path}: {error}"
			)));
		}
	}

	if !dry_run {
		let parent = write_target.parent().ok_or_else(|| {
			AlfredError::InvalidArgument(format!("path has no parent: {}", write_target.display()))
		})?;
		fs::create_dir_all(parent).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to create parent directory {}: {error}",
				parent.display()
			))
		})?;
		fs::write(write_target.as_path(), args.content.as_bytes())
			.map_err(|error| AlfredError::IoError(format!("failed to write {path}: {error}")))?;
	}

	Ok(json!({
		"path": path,
		"bytes_written": bytes_written,
	}))
}

fn handle_append_file(
	args: Value,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<Value, AlfredError> {
	let args = parse_args::<AppendFileArgs>(args)?;
	let path = workspace_query::normalize_required_path(args.path.as_str())?;
	let bytes_written = args.content.len() as u64;

	let workspace_root = services.config.workspace_root.as_path();
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, &path);
	let write_target = crate::workspace_boundary::resolve_write_target_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	)?;

	if let Ok(metadata) = fs::symlink_metadata(write_target.as_path())
		&& metadata.is_dir()
	{
		return Err(AlfredError::Conflict(format!(
			"path already exists as a directory: {path}"
		)));
	}

	if !dry_run {
		let parent = write_target.parent().ok_or_else(|| {
			AlfredError::InvalidArgument(format!("path has no parent: {}", write_target.display()))
		})?;
		fs::create_dir_all(parent).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to create parent directory {}: {error}",
				parent.display()
			))
		})?;

		use std::io::Write;
		let mut file = fs::OpenOptions::new()
			.create(true)
			.append(true)
			.open(write_target.as_path())
			.map_err(|error| AlfredError::IoError(format!("failed to open {path}: {error}")))?;
		file.write_all(args.content.as_bytes())
			.map_err(|error| AlfredError::IoError(format!("failed to append {path}: {error}")))?;
	}

	Ok(json!({
		"path": path,
		"bytes_written": bytes_written,
	}))
}

fn handle_delete_file(
	args: Value,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<Value, AlfredError> {
	let args = parse_args::<DeletePathArgs>(args)?;
	let path = workspace_query::normalize_required_path(args.path.as_str())?;

	let workspace_root = services.config.workspace_root.as_path();
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, &path);
	let resolved = crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	)?;
	let Some(target) = resolved.or(Some(candidate)) else {
		return Ok(json!({"deleted": false}));
	};

	match fs::symlink_metadata(target.as_path()) {
		Ok(metadata) => {
			if metadata.is_dir() {
				return Err(AlfredError::InvalidArgument(format!(
					"path is a directory: {path}"
				)));
			}
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			return Ok(json!({"deleted": false}));
		}
		Err(error) => {
			return Err(AlfredError::IoError(format!(
				"failed to stat {path}: {error}"
			)));
		}
	}

	if !dry_run {
		match fs::remove_file(target.as_path()) {
			Ok(()) => {}
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(json!({"deleted": false}));
			}
			Err(error) => {
				return Err(AlfredError::IoError(format!(
					"failed to delete file {path}: {error}"
				)));
			}
		}
	}

	Ok(json!({"deleted": true}))
}

fn handle_create_dir(
	args: Value,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<Value, AlfredError> {
	let args = parse_args::<CreateDirArgs>(args)?;
	let path = workspace_query::normalize_required_path(args.path.as_str())?;
	let parents = args.parents.unwrap_or(false);

	let workspace_root = services.config.workspace_root.as_path();
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, &path);
	let write_target = crate::workspace_boundary::resolve_write_target_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	)?;

	match fs::symlink_metadata(write_target.as_path()) {
		Ok(metadata) => {
			if metadata.is_dir() {
				return Ok(json!({"created": false}));
			}
			return Err(AlfredError::Conflict(format!(
				"path already exists as a file: {path}"
			)));
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
		Err(error) => {
			return Err(AlfredError::IoError(format!(
				"failed to stat {path}: {error}"
			)));
		}
	}

	if !dry_run {
		let create = if parents {
			fs::create_dir_all(write_target.as_path())
		} else {
			fs::create_dir(write_target.as_path())
		};
		create.map_err(|error| {
			AlfredError::IoError(format!("failed to create directory {path}: {error}"))
		})?;
	}

	Ok(json!({"created": true}))
}

fn handle_delete_dir(
	args: Value,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<Value, AlfredError> {
	let args = parse_args::<DeletePathArgs>(args)?;
	let path = workspace_query::normalize_required_path(args.path.as_str())?;

	let workspace_root = services.config.workspace_root.as_path();
	let candidate = crate::path_encoding::resolve_workspace_relative_path(workspace_root, &path);
	let resolved = crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
		workspace_root,
		candidate.as_path(),
	)?;
	let Some(target) = resolved.or(Some(candidate)) else {
		return Ok(json!({"deleted": false}));
	};

	match fs::symlink_metadata(target.as_path()) {
		Ok(metadata) => {
			if !metadata.is_dir() {
				return Err(AlfredError::InvalidArgument(format!(
					"path is not a directory: {path}"
				)));
			}
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			return Ok(json!({"deleted": false}));
		}
		Err(error) => {
			return Err(AlfredError::IoError(format!(
				"failed to stat {path}: {error}"
			)));
		}
	}

	if !dry_run {
		match fs::remove_dir(target.as_path()) {
			Ok(()) => {}
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(json!({"deleted": false}));
			}
			Err(error) => {
				return Err(AlfredError::IoError(format!(
					"failed to delete directory {path}: {error}"
				)));
			}
		}
	}

	Ok(json!({"deleted": true}))
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}
