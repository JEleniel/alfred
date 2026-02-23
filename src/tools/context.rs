//! Context tool group.

/// `workspace_dir` tool group.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContextTools;

impl ContextTools {
	pub const NAMES: &'static [&'static str] = &["workspace_dir"];
}
