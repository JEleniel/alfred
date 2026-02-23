//! Log tool group.

/// Log tailing and search tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct LogTools;

impl LogTools {
	pub const NAMES: &'static [&'static str] = &["log_tail", "log_search"];
}
