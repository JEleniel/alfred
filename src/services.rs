//! Service-layer component container.

pub mod file_ops;
pub mod indexer;
pub mod job_manager;
pub mod log_manager;
pub mod memory_store;
pub mod plan_store;
pub mod task_runner;

use anyhow::Result;

use crate::configuration::{AppConfig, default_plan_path, default_runtime_log_path};

use self::file_ops::FileOperationsEngine;
use self::indexer::WorkspaceIndexer;
use self::job_manager::JobManager;
use self::log_manager::LogManager;
use self::memory_store::MemoryStore;
use self::plan_store::PlanStore;
use self::task_runner::TaskRunner;

/// Aggregates all long-lived services used by tool handlers.
#[derive(Debug)]
pub struct ServiceContainer {
	pub config: AppConfig,
	pub indexer: WorkspaceIndexer,
	pub file_ops: FileOperationsEngine,
	pub task_runner: TaskRunner,
	pub jobs: JobManager,
	pub logs: LogManager,
	pub plan_store: PlanStore,
	pub memory_store: MemoryStore,
}

impl ServiceContainer {
	/// Builds a container from effective configuration values.
	pub fn new(config: AppConfig) -> Result<Self> {
		let workspace_root = config.workspace_root.clone();
		let plan_path = default_plan_path(&workspace_root);
		let runtime_log_path = default_runtime_log_path(&workspace_root);

		Ok(Self {
			config: config.clone(),
			indexer: WorkspaceIndexer::new_with_options(
				workspace_root.clone(),
				config.effective_workspace_index_root(),
				std::time::Duration::from_secs(config.index_persist_interval_seconds),
				std::time::Duration::from_millis(config.index_watch_debounce_millis),
			)?,
			file_ops: FileOperationsEngine::new(workspace_root.clone()),
			task_runner: TaskRunner::new(workspace_root.clone()),
			jobs: JobManager::new(),
			logs: LogManager::new(runtime_log_path),
			plan_store: PlanStore::new(workspace_root.clone(), plan_path),
			memory_store: MemoryStore::new(
				config.user_config_path.with_file_name("alfred.sqlite3"),
				workspace_root.join(".agents/alfred/alfred.sqlite3"),
			),
		})
	}
}
