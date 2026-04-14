//! Alfred application shell.

use serde_json::Value;

use crate::envelope::ToolEnvelope;
use crate::protocol::ToolDescriptor;
use crate::router::{RoutedTool, ToolRouter, ToolRouterError};
use crate::tool_execution::{execute_tool, unknown_tool_response};

/// Alfred application state.
pub struct Alfred {
	router: ToolRouter,
}

impl Alfred {
	/// Creates a new Alfred application shell.
	pub fn new() -> Self {
		Self {
			router: ToolRouter::new(),
		}
	}

	/// Returns the canonical public capability list.
	pub fn capabilities(&self) -> &[ToolDescriptor<'static>] {
		self.router.capabilities()
	}

	/// Returns the canonical capability metadata for a tool name.
	pub fn capability(&self, name: &str) -> Option<&ToolDescriptor<'static>> {
		self.router.capability(name)
	}

	/// Executes a tool call against the currently implemented handlers.
	pub fn invoke_tool(&self, name: &str, arguments: Option<&Value>) -> ToolEnvelope<Value> {
		match self.dispatch_tool(name) {
			Ok(routed) => self.execute_routed_tool(routed, arguments),
			Err(_) => unknown_tool_response(name),
		}
	}

	/// Dispatches a tool name to the canonical router boundary.
	pub(crate) fn dispatch_tool(&self, name: &str) -> Result<RoutedTool<'_>, ToolRouterError> {
		self.router.dispatch(name)
	}

	fn execute_routed_tool(
		&self,
		routed: RoutedTool<'_>,
		arguments: Option<&Value>,
	) -> ToolEnvelope<Value> {
		execute_tool(
			routed.descriptor(),
			routed.route(),
			arguments,
			self.capabilities(),
		)
	}
}

impl Default for Alfred {
	fn default() -> Self {
		Self::new()
	}
}
