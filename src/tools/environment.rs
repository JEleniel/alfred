//! Environment variable tool group.

/// Environment variable tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct EnvironmentTools;

impl EnvironmentTools {
	pub const NAMES: &'static [&'static str] = &["env_list", "env_get", "env_set", "env_unset"];
}
