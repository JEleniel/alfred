use std::path::Path;

use crate::configuration::default_runtime_log_path;

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
