//! Context tool group.

use serde_json::{Value, json};

use crate::services::ServiceContainer;

/// `workspace_dir` tool group.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContextTools;

impl ContextTools {
	pub const NAMES: &'static [&'static str] = &["workspace_dir"];
}

/// Handles context tool calls.
pub fn dispatch_tool_call(name: &str, services: &ServiceContainer) -> Option<Value> {
	match name {
		"workspace_dir" => Some(workspace_dir(services)),
		_ => None,
	}
}

fn workspace_dir(services: &ServiceContainer) -> Value {
	json!({
		"root": services.indexer.workspace_root().display().to_string(),
	})
}
