//! Patch tool.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::services::file_ops::PatchRequest;

/// Text patch tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct PatchTools;

impl PatchTools {
	pub const NAMES: &'static [&'static str] = &["patch"];
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PatchOperation {
	Apply,
	Revert,
}

#[derive(Debug, Deserialize)]
struct PatchFileInput {
	path: String,
	patch: String,
}

#[derive(Debug, Deserialize)]
struct PatchArgs {
	operation: Option<PatchOperation>,
	patches: Option<Vec<PatchFileInput>>,
	dry_run: Option<bool>,
}

/// Handles patch tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"patch" => handle_patch(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_patch(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<PatchArgs>(args)?;
	let dry_run = args.dry_run.unwrap_or(true);

	match args.operation.unwrap_or(PatchOperation::Apply) {
		PatchOperation::Apply => {
			handle_patch_apply(args.patches.unwrap_or_default(), dry_run, services)
		}
		PatchOperation::Revert => handle_patch_revert(dry_run, services),
	}
}

fn handle_patch_apply(
	patches: Vec<PatchFileInput>,
	dry_run: bool,
	services: &ServiceContainer,
) -> Result<Value, AlfredError> {
	if patches.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"patches must contain at least one item".to_string(),
		));
	}

	let requests = patches
		.into_iter()
		.map(|item| PatchRequest {
			path: item.path,
			patch: item.patch,
		})
		.collect::<Vec<_>>();
	let results = services.file_ops.apply_patches(requests, dry_run)?;

	let files = results
		.into_iter()
		.map(|result| {
			json!({
				"path": result.path,
				"patched": result.patched,
				"bytes_written": result.bytes_written,
				"conflicts": result.conflicts,
				"warnings": result.warnings,
			})
		})
		.collect::<Vec<_>>();

	Ok(json!({
		"operation": "apply",
		"dry_run": dry_run,
		"files": files,
	}))
}

fn handle_patch_revert(dry_run: bool, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let results = services.file_ops.revert_last_patch(dry_run)?;
	let files = results
		.into_iter()
		.map(|result| {
			json!({
				"path": result.path,
				"reverted": result.reverted,
				"bytes_written": result.bytes_written,
				"conflicts": result.conflicts,
			})
		})
		.collect::<Vec<_>>();

	Ok(json!({
		"operation": "revert",
		"dry_run": dry_run,
		"files": files,
	}))
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}
