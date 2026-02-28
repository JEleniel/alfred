//! Tool group registration and discovery skeleton.

pub mod capabilities;
pub mod chain;
pub mod context;
pub mod environment;
pub mod file_mutation;
pub mod jobs;
pub mod logs;
pub mod memory;
pub mod patch;
pub mod plan;
pub mod session;
pub mod status;
pub mod task_execution;
pub mod workspace_query;

use serde_json::Value;

use crate::configuration::AppConfig;
use crate::errors::AlfredError;
use crate::services::ServiceContainer;

use self::capabilities::CapabilityTools;
use self::chain::ChainTools;
use self::context::ContextTools;
use self::environment::EnvironmentTools;
use self::file_mutation::FileMutationTools;
use self::jobs::JobTools;
use self::logs::LogTools;
use self::memory::MemoryTools;
use self::patch::PatchTools;
use self::plan::PlanTools;
use self::session::SessionTools;
use self::status::StatusTools;
use self::task_execution::TaskExecutionTools;
use self::workspace_query::WorkspaceQueryTools;

/// Registry of exposed tool groups.
#[derive(Debug, Clone, Copy, Default)]
pub struct ToolRegistry {
	pub context: ContextTools,
	pub workspace_query: WorkspaceQueryTools,
	pub file_mutation: FileMutationTools,
	pub patch: PatchTools,
	pub status: StatusTools,
	pub task_execution: TaskExecutionTools,
	pub jobs: JobTools,
	pub session: SessionTools,
	pub logs: LogTools,
	pub plan: PlanTools,
	pub capabilities: CapabilityTools,
	pub chain: ChainTools,
	pub environment: EnvironmentTools,
	pub memory: MemoryTools,
}

impl ToolRegistry {
	/// Creates the default tool registry.
	pub fn new() -> Self {
		Self::default()
	}

	/// Returns a stable, lexicographically sorted list of tool names.
	pub fn tool_names(&self) -> Vec<&'static str> {
		let mut names = Vec::new();
		names.extend_from_slice(ContextTools::NAMES);
		names.extend_from_slice(WorkspaceQueryTools::NAMES);
		names.extend_from_slice(LogTools::NAMES);
		names.extend_from_slice(PatchTools::NAMES);
		names.extend_from_slice(StatusTools::NAMES);
		names.extend_from_slice(PlanTools::NAMES);
		names.extend_from_slice(MemoryTools::NAMES);
		names.extend_from_slice(CapabilityTools::NAMES);
		names.sort_unstable();
		names
	}

	/// Returns the total number of registered tools.
	pub fn tool_count(&self) -> usize {
		self.tool_names().len()
	}

	/// Returns policy-filtered tool names for a given configuration.
	pub fn tool_names_for_config(&self, config: &AppConfig) -> Vec<&'static str> {
		self.tool_names()
			.into_iter()
			.filter(|name| config.is_tool_enabled(name))
			.collect()
	}
}

/// Dispatches a single tool call to the appropriate tool group implementation.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Value, AlfredError> {
	if !services.config.is_tool_enabled(name) {
		return Err(AlfredError::InvalidArgument(format!(
			"tool disabled by policy: {name}"
		)));
	}

	if let Some(data) = context::dispatch_tool_call(name, services) {
		return Ok(data);
	}

	if let Some(data) = capabilities::dispatch_tool_call(name, services) {
		return Ok(data);
	}

	if let Some(data) = logs::dispatch_tool_call(name, args.clone(), services)? {
		return Ok(data);
	}

	if let Some(data) = plan::dispatch_tool_call(name, args.clone(), services)? {
		return Ok(data);
	}

	if let Some(data) = memory::dispatch_tool_call(name, args.clone(), services)? {
		return Ok(data);
	}

	if let Some(data) = patch::dispatch_tool_call(name, args.clone(), services)? {
		return Ok(data);
	}

	if let Some(data) = status::dispatch_tool_call(name, args.clone(), services)? {
		return Ok(data);
	}

	if let Some(data) = workspace_query::dispatch_tool_call(name, args, services)? {
		return Ok(data);
	}

	Err(AlfredError::InvalidArgument(format!(
		"tool not implemented: {name}"
	)))
}
