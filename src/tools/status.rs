//! Status tool.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AlfredError;
use crate::services::ServiceContainer;

/// Runtime status tool.
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusTools;

impl StatusTools {
	pub const NAMES: &'static [&'static str] = &["status"];
}

#[derive(Debug, Deserialize, Default)]
struct StatusArgs {
	verbose: Option<bool>,
}

/// Handles status tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"status" => handle_status(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_status(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<StatusArgs>(args)?;
	let verbose = args.verbose.unwrap_or(false);

	let workspace_root = services
		.indexer
		.workspace_root()
		.to_string_lossy()
		.to_string();
	let plan_path = services
		.plan_store
		.plan_path()
		.to_string_lossy()
		.to_string();
	let log_path = services.logs.log_path().to_string_lossy().to_string();
	let index_root = services
		.config
		.effective_workspace_index_root()
		.to_string_lossy()
		.to_string();

	let index_ready = services.indexer.is_ready();
	let index_stats = services.indexer.snapshot_stats();

	let memory = memory_status(verbose)?;

	Ok(json!({
		"version": env!("CARGO_PKG_VERSION"),
		"pid": std::process::id(),
		"workspace_root": workspace_root,
		"paths": {
			"plan": plan_path,
			"runtime_log": log_path,
			"workspace_index_root": index_root,
		},
		"index": {
			"ready": index_ready,
			"indexed_files": index_stats.indexed_files,
			"indexed_directories": index_stats.indexed_directories,
			"indexed_text_files": index_stats.indexed_text_files,
		},
		"memory": memory,
	}))
}

fn memory_status(verbose: bool) -> Result<Value, AlfredError> {
	use sysinfo::{Pid, System};

	let pid = Pid::from_u32(std::process::id());
	let mut system = System::new_all();
	system.refresh_all();

	let system_total_bytes = system.total_memory();
	let system_used_bytes = system.used_memory();
	let system_free_bytes = system.free_memory();
	let system_available_bytes = system.available_memory();

	let process = system.process(pid);
	let (rss_bytes, virtual_bytes) = match process {
		Some(process) => (process.memory(), process.virtual_memory()),
		None => (0, 0),
	};

	let mut status = json!({
		"unit": "bytes",
		"system": {
			"total": system_total_bytes,
			"used": system_used_bytes,
			"free": system_free_bytes,
			"available": system_available_bytes,
		},
		"process": {
			"rss": rss_bytes,
			"virtual": virtual_bytes,
		},
	});

	if verbose {
		let load_average = System::load_average();
		let cpu_count = system.cpus().len();
		let status_object = status.as_object_mut().ok_or_else(|| {
			AlfredError::Internal("status memory payload not an object".to_string())
		})?;
		status_object.insert(
			"system_load".to_string(),
			json!({
				"one": load_average.one,
				"five": load_average.five,
				"fifteen": load_average.fifteen,
			}),
		);
		status_object.insert("cpu_count".to_string(), json!(cpu_count));
	}

	Ok(status)
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}
