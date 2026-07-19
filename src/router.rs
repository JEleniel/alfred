//! Tool routing boundary for Alfred.

mod route;

use crate::capabilities::CapabilityRegistry;
use crate::protocol::ToolDescriptor;
use thiserror::Error;

pub(crate) use crate::protocol::ToolRoute;
pub(crate) use route::RoutedTool;

/// Resolves tool names against the canonical capability registry.
pub(crate) struct ToolRouter {
	capabilities: CapabilityRegistry,
}

impl ToolRouter {
	/// Creates a router backed by the canonical capability registry.
	pub(crate) fn new() -> Self {
		Self {
			capabilities: CapabilityRegistry::new(),
		}
	}

	/// Returns the advertised public tools.
	pub(crate) fn capabilities(&self) -> &[ToolDescriptor<'static>] {
		self.capabilities.tools()
	}

	/// Finds a single advertised tool by name.
	pub(crate) fn capability(&self, name: &str) -> Option<&ToolDescriptor<'static>> {
		self.capabilities().iter().find(|tool| tool.name == name)
	}

	/// Dispatches a tool name to its canonical route and descriptor.
	pub(crate) fn dispatch(&self, name: &str) -> Result<RoutedTool<'_>, ToolRouterError> {
		let descriptor = self
			.capability(name)
			.ok_or_else(|| ToolRouterError::UnknownTool {
				name: name.to_owned(),
			})?;
		let route = self
			.capabilities
			.route_for(descriptor.name)
			.ok_or_else(|| ToolRouterError::UnknownTool {
				name: name.to_owned(),
			})?;
		Ok(RoutedTool::new(route, descriptor))
	}
}

/// Dispatch failure for the router boundary.
#[derive(Debug, Error, Eq, PartialEq)]
pub(crate) enum ToolRouterError {
	/// The requested tool is not part of the canonical surface.
	#[error("unknown tool `{name}`.")]
	UnknownTool { name: String },
}

#[cfg(test)]
mod tests {
	#[path = "router_tests.rs"]
	mod router_tests;
}
