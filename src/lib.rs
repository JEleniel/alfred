//! Alfred library entry point.
//!
//! This crate exposes the skeleton application shell and canonical tool surface.

pub mod app;
pub mod capabilities;
pub mod design;
pub mod envelope;
pub mod protocol;
pub mod workspace_boundary;

pub(crate) mod router;
pub(crate) mod tool_execution;

#[cfg(test)]
#[path = "tests/integration_skeleton.rs"]
mod integration_skeleton;

use anyhow::Result;

/// Builds the Alfred skeleton and returns success.
pub fn run() -> Result<()> {
	let app = app::Alfred::new();
	debug_assert!(matches!(
		app.dispatch_tool("capabilities"),
		Ok(route) if route.route() == crate::router::ToolRoute::Capabilities
			&& route.descriptor().name == "capabilities"
	));
	Ok(())
}
