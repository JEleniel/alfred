//! Alfred crate root.

pub mod app;
pub mod configuration;
pub mod diagnostics;
pub mod errors;
pub mod logging;
pub mod protocol;
pub mod redaction;
pub mod services;
pub mod tools;

/// Bootstraps and runs the Alfred server process.
pub async fn run() -> anyhow::Result<()> {
	let app = app::AlfredApp::bootstrap().await?;
	app.run().await
}

#[cfg(test)]
#[path = "tests/lib_tests.rs"]
mod lib_tests;

#[cfg(test)]
#[path = "tests/redaction_tests.rs"]
mod redaction_tests;

#[cfg(test)]
#[path = "tests/protocol_tests.rs"]
mod protocol_tests;

#[cfg(test)]
#[path = "tests/configuration_tests.rs"]
mod configuration_tests;

#[cfg(test)]
#[path = "tests/logging_tests.rs"]
mod logging_tests;

#[cfg(test)]
#[path = "tests/indexer_tests.rs"]
mod indexer_tests;

#[cfg(test)]
#[path = "tests/workspace_query_tests.rs"]
mod workspace_query_tests;
