//! Workspace query tool group.

/// Index-backed workspace query tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct WorkspaceQueryTools;

impl WorkspaceQueryTools {
	pub const NAMES: &'static [&'static str] = &[
		"ls",
		"read_range",
		"file_stat",
		"file_read_bytes",
		"grep",
		"search",
		"diff",
	];
}
