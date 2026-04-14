use std::fs;
use std::path::PathBuf;

use crate::workspace_boundary::{
	WorkspaceBoundaryError, normalize_workspace_relative_path,
	resolve_write_target_within_workspace_root, try_resolve_existing_path_within_workspace_root,
};

#[test]
fn normalize_workspace_relative_path_collapses_separators_and_dots() {
	let normalized = normalize_workspace_relative_path("folder/./nested/./file.txt")
		.expect("path should normalize");

	assert_eq!(normalized, PathBuf::from("folder/nested/file.txt"));
}

#[test]
fn normalize_workspace_relative_path_accepts_absolute_paths() {
	let absolute_path =
		std::env::temp_dir().join(format!("alfred-boundary-normalize-{}", std::process::id()));

	let normalized =
		normalize_workspace_relative_path(absolute_path.to_str().expect("path should be utf-8"))
			.expect("absolute path should normalize");

	assert_eq!(normalized, absolute_path);
}

#[test]
fn normalize_workspace_relative_path_collapses_parent_dirs_within_workspace() {
	let normalized =
		normalize_workspace_relative_path("folder/../file.txt").expect("path should normalize");

	assert_eq!(normalized, PathBuf::from("file.txt"));
}

#[test]
fn normalize_workspace_relative_path_rejects_traversal_outside_workspace() {
	assert!(matches!(
		normalize_workspace_relative_path("../escape"),
		Err(WorkspaceBoundaryError::BoundaryViolation { .. })
	));
}

#[cfg(windows)]
#[test]
fn normalize_workspace_relative_path_rejects_windows_reserved_and_forbidden_names() {
	assert!(matches!(
		normalize_workspace_relative_path("CON"),
		Err(WorkspaceBoundaryError::InvalidPath { .. })
	));
	assert!(matches!(
		normalize_workspace_relative_path("bad<name>.txt"),
		Err(WorkspaceBoundaryError::InvalidPath { .. })
	));
}

#[cfg(not(windows))]
#[test]
fn normalize_workspace_relative_path_accepts_windows_style_inputs_on_non_windows() {
	assert_eq!(
		normalize_workspace_relative_path("\\\\server\\share\\file.txt")
			.expect("path should normalize"),
		PathBuf::from("\\\\server\\share\\file.txt")
	);
	assert_eq!(
		normalize_workspace_relative_path("C:\\workspace\\file.txt")
			.expect("path should normalize"),
		PathBuf::from("C:\\workspace\\file.txt")
	);
	assert_eq!(
		normalize_workspace_relative_path("CON").expect("path should normalize"),
		PathBuf::from("CON")
	);
}

#[test]
fn resolve_write_target_within_workspace_root_joins_the_normalized_path() {
	let root = PathBuf::from("/workspace/root");
	let resolved = resolve_write_target_within_workspace_root(&root, "folder/file.txt")
		.expect("target should resolve");

	assert_eq!(resolved, root.join("folder/file.txt"));
}

#[test]
fn resolve_write_target_within_workspace_root_collapses_parent_dirs() {
	let root = PathBuf::from("/workspace/root");
	let resolved = resolve_write_target_within_workspace_root(&root, "folder/../file.txt")
		.expect("target should resolve");

	assert_eq!(resolved, root.join("file.txt"));
}

#[test]
fn resolve_write_target_within_workspace_root_accepts_nested_parent_dirs_inside_workspace() {
	let root = PathBuf::from("/workspace");
	let resolved = resolve_write_target_within_workspace_root(
		&root,
		"../workspace/subfolder/../../workspace/docs/somefile.txt",
	)
	.expect("target should resolve");

	assert_eq!(resolved, root.join("docs/somefile.txt"));
}

#[test]
fn resolve_write_target_within_workspace_root_accepts_absolute_paths_inside_root() {
	let root = std::env::temp_dir().join(format!("alfred-boundary-root-{}", std::process::id()));
	let _ = fs::remove_dir_all(&root);
	fs::create_dir_all(&root).expect("workspace root should create");
	let absolute_target = root.join("folder/file.txt");

	let resolved = resolve_write_target_within_workspace_root(
		&root,
		absolute_target.to_str().expect("path should be utf-8"),
	)
	.expect("absolute path inside the workspace root should resolve");

	assert_eq!(resolved, absolute_target);
	let _ = fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn existing_path_resolution_rejects_symlink_escapes() {
	use std::os::unix::fs::symlink;

	let root = std::env::temp_dir().join(format!("alfred-boundary-{}", std::process::id()));
	let _ = fs::remove_dir_all(&root);
	fs::create_dir_all(&root).expect("workspace root should create");
	let outside = std::env::temp_dir().join(format!("alfred-outside-{}", std::process::id()));
	let _ = fs::remove_dir_all(&outside);
	fs::create_dir_all(&outside).expect("outside dir should create");
	let link_path = root.join("escape");
	symlink(&outside, &link_path).expect("symlink should create");

	let result = try_resolve_existing_path_within_workspace_root(&root, "escape");

	assert!(matches!(
		result,
		Err(WorkspaceBoundaryError::BoundaryViolation { .. })
	));
	let _ = fs::remove_dir_all(&root);
	let _ = fs::remove_dir_all(&outside);
}
