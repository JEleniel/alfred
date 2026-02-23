//! Background job tool group.

/// Background job tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct JobTools;

impl JobTools {
	pub const NAMES: &'static [&'static str] = &[
		"job_status",
		"job_statuses",
		"job_cancel",
		"job_list",
		"job_read",
	];
}
