//! Capability discovery tool group.

/// Capability discovery tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct CapabilityTools;

impl CapabilityTools {
	pub const NAMES: &'static [&'static str] = &["capabilities"];
}
