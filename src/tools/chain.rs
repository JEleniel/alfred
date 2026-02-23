//! Action chaining tool group.

/// Action chaining tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChainTools;

impl ChainTools {
	pub const NAMES: &'static [&'static str] = &["chain"];
}
