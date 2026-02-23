use std::process::ExitCode;

use alfred::run;

#[tokio::main]
async fn main() -> ExitCode {
	match run().await {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("{} startup failed: {error:#}", env!("CARGO_PKG_NAME"));
			ExitCode::FAILURE
		}
	}
}
