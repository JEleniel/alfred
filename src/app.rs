//! Application lifecycle and bootstrapping.

use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};
use log::{info, trace};

use crate::configuration::AppConfig;
use crate::logging;
use crate::protocol;
use crate::services::ServiceContainer;
use crate::tools::ToolRegistry;

/// Top-level application object for Alfred.
#[derive(Debug)]
pub struct AlfredApp {
	config: AppConfig,
	services: ServiceContainer,
	tools: ToolRegistry,
}

impl AlfredApp {
	/// Creates a new app instance using default configuration discovery.
	pub async fn bootstrap() -> Result<Self> {
		let config = AppConfig::load_default().context("failed to load configuration")?;
		logging::init_logging(&config).context("failed to initialize logging")?;
		trace!(
			"bootstrap config user={} workspace={}",
			config.user_config_path.display(),
			config.workspace_config_path.display(),
		);
		let services = ServiceContainer::new(config.clone());
		services
			.indexer
			.rebuild()
			.context("failed to build workspace index")?;
		let tools = ToolRegistry::new();

		Ok(Self {
			config,
			services,
			tools,
		})
	}

	/// Runs the Alfred process loop skeleton.
	pub async fn run(&self) -> Result<()> {
		info!(
			"booted {} at {} with {} tools",
			env!("CARGO_PKG_NAME"),
			self.config.workspace_root.display(),
			self.tools.tool_count()
		);

		let _ = &self.services;
		self.run_stdio_loop()?;
		Ok(())
	}

	fn run_stdio_loop(&self) -> Result<()> {
		let stdin = io::stdin();
		let mut stdout = io::stdout();

		for line in stdin.lock().lines() {
			let raw_line = line.context("failed to read stdio frame")?;
			let frame = raw_line.trim();
			if frame.is_empty() {
				continue;
			}

			let inbound_kind = protocol::classify_wire_frame(frame);
			logging::trace_mcp_inbound(inbound_kind, frame);
			if inbound_kind == "initialize_request" {
				self.trace_workspace_hints(frame)?;
			}

			if let Some(response) = protocol::handle_startup_frame(frame)? {
				let outbound_kind = protocol::classify_wire_frame(response.as_str());
				logging::trace_mcp_outbound(outbound_kind, response.as_str());
				writeln!(stdout, "{response}").context("failed to write stdio response")?;
				stdout.flush().context("failed to flush stdio response")?;
			}
		}

		Ok(())
	}

	fn trace_workspace_hints(&self, initialize_frame: &str) -> Result<()> {
		if let Some(hints) = protocol::extract_initialize_workspace_hints(initialize_frame)? {
			let hints_text = hints.to_string();
			logging::trace_mcp_inbound("workspace_hints", hints_text.as_str());
			return Ok(());
		}

		let fallback = format!(
			"source=process_cwd workspace_root={}",
			self.config.workspace_root.display()
		);
		logging::trace_mcp_inbound("workspace_hints", fallback.as_str());
		Ok(())
	}
}
