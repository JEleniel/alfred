//! Tool group registration and discovery skeleton.

pub mod capabilities;
pub mod chain;
pub mod context;
pub mod environment;
pub mod file_mutation;
pub mod jobs;
pub mod logs;
pub mod memory;
pub mod plan;
pub mod session;
pub mod task_execution;
pub mod workspace_query;

use self::capabilities::CapabilityTools;
use self::chain::ChainTools;
use self::context::ContextTools;
use self::environment::EnvironmentTools;
use self::file_mutation::FileMutationTools;
use self::jobs::JobTools;
use self::logs::LogTools;
use self::memory::MemoryTools;
use self::plan::PlanTools;
use self::session::SessionTools;
use self::task_execution::TaskExecutionTools;
use self::workspace_query::WorkspaceQueryTools;

/// Registry of exposed tool groups.
#[derive(Debug, Clone, Copy, Default)]
pub struct ToolRegistry {
	pub context: ContextTools,
	pub workspace_query: WorkspaceQueryTools,
	pub file_mutation: FileMutationTools,
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
		names.extend_from_slice(FileMutationTools::NAMES);
		names.extend_from_slice(TaskExecutionTools::NAMES);
		names.extend_from_slice(JobTools::NAMES);
		names.extend_from_slice(SessionTools::NAMES);
		names.extend_from_slice(LogTools::NAMES);
		names.extend_from_slice(PlanTools::NAMES);
		names.extend_from_slice(CapabilityTools::NAMES);
		names.extend_from_slice(ChainTools::NAMES);
		names.extend_from_slice(EnvironmentTools::NAMES);
		names.extend_from_slice(MemoryTools::NAMES);
		names.sort_unstable();
		names
	}

	/// Returns the total number of registered tools.
	pub fn tool_count(&self) -> usize {
		self.tool_names().len()
	}
}
