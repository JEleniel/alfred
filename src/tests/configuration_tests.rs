use std::path::Path;

use crate::configuration::{
	default_runtime_log_path, default_workspace_index_root, default_workspace_local_index_root,
};

#[test]
fn runtime_log_path_uses_user_data_alfred_subfolder() {
	let workspace_root = Path::new("/workspace-root");
	let runtime_path = default_runtime_log_path(workspace_root);

	assert!(runtime_path.ends_with(Path::new("alfred/logs/runtime.ndjson")));
}

#[test]
fn runtime_log_path_matches_data_dir_or_workspace_fallback() {
	let workspace_root = Path::new("/workspace-root");
	let runtime_path = default_runtime_log_path(workspace_root);

	if let Some(data_dir) = dirs::data_dir() {
		assert_eq!(
			runtime_path,
			data_dir.join("alfred").join("logs").join("runtime.ndjson")
		);
	} else {
		assert_eq!(
			runtime_path,
			workspace_root
				.join(".agents")
				.join("alfred")
				.join("logs")
				.join("runtime.ndjson")
		);
	}
}

#[test]
fn workspace_index_root_uses_user_data_alfred_subfolder() {
	let workspace_root = Path::new("/workspace-root");
	let index_root = default_workspace_index_root(workspace_root);

	assert!(index_root.to_string_lossy().contains("alfred/index/"));
}

#[test]
fn workspace_local_index_root_uses_agents_alfred_index() {
	let workspace_root = Path::new("/workspace-root");
	let index_root = default_workspace_local_index_root(workspace_root);

	assert_eq!(
		index_root,
		workspace_root.join(".agents").join("alfred").join("index")
	);
}
