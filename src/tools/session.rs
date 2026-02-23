//! Session introspection tool group.

/// Session inspection tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct SessionTools;

impl SessionTools {
	pub const NAMES: &'static [&'static str] = &["session_recent"];
}
