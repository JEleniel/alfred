//! Canonical dispatch route types for Alfred.

use crate::protocol::{ToolDescriptor, ToolRoute};

/// A routed tool with its canonical descriptor.
pub(crate) struct RoutedTool<'a> {
	route: ToolRoute,
	descriptor: &'a ToolDescriptor<'static>,
}

impl<'a> RoutedTool<'a> {
	/// Creates a routed tool from a canonical route and descriptor.
	pub(crate) fn new(route: ToolRoute, descriptor: &'a ToolDescriptor<'static>) -> Self {
		Self { route, descriptor }
	}

	/// Returns the routed tool kind.
	pub(crate) fn route(&self) -> ToolRoute {
		self.route
	}

	/// Returns the canonical tool descriptor associated with the route.
	pub(crate) fn descriptor(&self) -> &'a ToolDescriptor<'static> {
		self.descriptor
	}
}
