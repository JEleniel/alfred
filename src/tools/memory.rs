//! Local memory tool group.

/// Local memory CRUD/search tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryTools;

impl MemoryTools {
	pub const NAMES: &'static [&'static str] = &[
		"memory_put",
		"memory_get",
		"memory_delete",
		"memory_list",
		"memory_search",
	];
}
