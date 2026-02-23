use crate::configuration::is_workspace_relative_path;
use crate::tools::ToolRegistry;

#[test]
fn accepts_workspace_relative_paths() {
	assert!(is_workspace_relative_path("docs/design/ProjectPlan.md"));
}

#[test]
fn rejects_absolute_paths() {
	assert!(!is_workspace_relative_path("/etc/passwd"));
	assert!(!is_workspace_relative_path("C:/Windows/System32"));
}

#[test]
fn rejects_path_traversal_inputs() {
	assert!(!is_workspace_relative_path("../secrets.txt"));
	assert!(!is_workspace_relative_path("..\\secrets.txt"));
}

#[test]
fn registry_contains_memory_tools() {
	let registry = ToolRegistry::new();
	let names = registry.tool_names();

	assert!(names.contains(&"memory_put"));
	assert!(names.contains(&"memory_search"));
	assert!(registry.tool_count() >= names.len());
}
