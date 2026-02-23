//! Plan tool group.

/// Project plan CRUD tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlanTools;

impl PlanTools {
	pub const NAMES: &'static [&'static str] = &[
		"plan_get",
		"plan_update",
		"plan_edit",
		"plan_add",
		"plan_delete",
	];
}
