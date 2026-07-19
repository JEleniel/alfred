use crate::router::{ToolRoute, ToolRouter};

#[test]
fn dispatch_returns_the_expected_route_for_known_tools() {
	let router = ToolRouter::new();
	let routed = router.dispatch("workspace_dir").expect("tool should route");

	assert!(matches!(routed.route(), ToolRoute::WorkspaceDir));
	assert_eq!(routed.descriptor().name, "workspace_dir");
}

#[test]
fn dispatch_rejects_unknown_tools() {
	let router = ToolRouter::new();

	assert!(router.dispatch("not-a-tool").is_err());
}
