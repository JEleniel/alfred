//! Task execution tool group.

/// Task execution tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskExecutionTools;

impl TaskExecutionTools {
	pub const NAMES: &'static [&'static str] = &["list_tasks", "task_run"];
}
