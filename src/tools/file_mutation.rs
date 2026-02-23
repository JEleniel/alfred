//! File mutation tool group.

/// Workspace mutation tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileMutationTools;

impl FileMutationTools {
	pub const NAMES: &'static [&'static str] = &[
		"file_create",
		"file_append",
		"file_patch",
		"multi_file_patch",
		"file_delete",
		"dir_create",
		"dir_delete",
		"file_create_bytes",
		"file_append_bytes",
		"path_move",
		"path_copy",
		"path_delete",
	];
}
