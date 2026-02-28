//! Logging configuration with deterministic redaction.

use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use chrono::{Days, NaiveDate, Utc};
use fern::Dispatch;
use log::{LevelFilter, trace};
use serde_json::json;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::configuration::AppConfig;
use crate::redaction::Redactor;

static LOGGER_READY: OnceLock<()> = OnceLock::new();

/// Initializes process-wide TRACE logging with NPI redaction.
pub fn init_logging(config: &AppConfig) -> Result<()> {
	if LOGGER_READY.get().is_some() {
		return Ok(());
	}

	let runtime_log_path = config.effective_runtime_log_path();

	configure_trace_logger(
		runtime_log_path.as_path(),
		config.runtime_log_retention_days,
	)?;
	let _ = LOGGER_READY.set(());
	Ok(())
}

/// Emits a TRACE record for an inbound MCP wire frame.
pub fn trace_mcp_inbound(kind: &str, raw_frame: &str) {
	trace!(target: "alfred::mcp::wire", "direction=in kind={kind} frame={raw_frame}");
}

/// Emits a TRACE record for an outbound MCP wire frame.
pub fn trace_mcp_outbound(kind: &str, raw_frame: &str) {
	trace!(target: "alfred::mcp::wire", "direction=out kind={kind} frame={raw_frame}");
}

fn configure_trace_logger(runtime_log_path: &Path, retention_days: u64) -> Result<()> {
	if let Some(parent) = runtime_log_path.parent() {
		fs::create_dir_all(parent).context("failed to create runtime log directory")?;
	}

	rotate_runtime_log(runtime_log_path, retention_days)?;

	let redactor = Redactor::try_default()?;
	let file_dispatch = build_file_dispatch(redactor, runtime_log_path)?;

	Dispatch::new()
		.level(LevelFilter::Trace)
		.chain(file_dispatch)
		.apply()
		.context("failed to install global logger")
}

fn build_file_dispatch(redactor: Redactor, runtime_log_path: &Path) -> Result<Dispatch> {
	let file = fern::log_file(runtime_log_path).with_context(|| {
		format!(
			"failed to open runtime log file at {}",
			runtime_log_path.display()
		)
	})?;

	Ok(Dispatch::new()
		.format(move |out, message, record| {
			let timestamp = Utc::now().to_rfc3339();
			let redacted = redactor.redact_text(message.to_string().as_str());
			let line = json!({
				"timestamp": timestamp,
				"level": record.level().to_string(),
				"message": redacted,
				"source": record.target(),
				"extra": {},
			});
			out.finish(format_args!("{}", line));
		})
		.chain(file))
}

pub(crate) fn rotate_runtime_log(runtime_log_path: &Path, retention_days: u64) -> Result<()> {
	if runtime_log_path.exists() {
		archive_runtime_log(runtime_log_path)?;
	}

	prune_expired_archives(runtime_log_path, retention_days)?;
	Ok(())
}

fn archive_runtime_log(runtime_log_path: &Path) -> Result<()> {
	let archive_path = next_archive_path(runtime_log_path)?;
	let source_name = runtime_log_path
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("runtime.ndjson")
		.to_string();

	let source_file = File::open(runtime_log_path).with_context(|| {
		format!(
			"failed to open runtime log file at {}",
			runtime_log_path.display()
		)
	})?;
	let archive_file = File::create(&archive_path).with_context(|| {
		format!(
			"failed to create runtime archive at {}",
			archive_path.display()
		)
	})?;

	let mut zip = ZipWriter::new(archive_file);
	let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
	zip.start_file(source_name, options)
		.context("failed to start runtime log zip entry")?;

	let mut reader = BufReader::new(source_file);
	io::copy(&mut reader, &mut zip).context("failed to write runtime log zip entry")?;
	zip.finish().context("failed to finalize runtime log zip")?;

	fs::remove_file(runtime_log_path).with_context(|| {
		format!(
			"failed to remove rotated runtime log at {}",
			runtime_log_path.display()
		)
	})
}

fn next_archive_path(runtime_log_path: &Path) -> Result<std::path::PathBuf> {
	let parent = runtime_log_path
		.parent()
		.context("runtime log path missing parent directory")?;
	let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
	let archive_name = format!("runtime-{timestamp}-{}.ndjson.zip", Uuid::new_v4().simple());
	Ok(parent.join(archive_name))
}

fn prune_expired_archives(runtime_log_path: &Path, retention_days: u64) -> Result<()> {
	let Some(parent) = runtime_log_path.parent() else {
		return Ok(());
	};

	let keep_days = retention_days.max(1);
	let today = Utc::now().date_naive();
	let oldest_keep_date = today
		.checked_sub_days(Days::new(keep_days.saturating_sub(1)))
		.unwrap_or(today);

	for entry in fs::read_dir(parent).with_context(|| {
		format!(
			"failed to read runtime log directory at {}",
			parent.display()
		)
	})? {
		let entry = entry.context("failed to inspect runtime log directory entry")?;
		let path = entry.path();
		if !path.is_file() {
			continue;
		}

		let Some(name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
			continue;
		};
		let Some(archive_date) = parse_archive_date(name) else {
			continue;
		};

		if archive_date < oldest_keep_date {
			fs::remove_file(&path).with_context(|| {
				format!(
					"failed to remove expired runtime archive at {}",
					path.display()
				)
			})?;
		}
	}

	Ok(())
}

fn parse_archive_date(file_name: &str) -> Option<NaiveDate> {
	if !file_name.starts_with("runtime-") || !file_name.ends_with(".ndjson.zip") {
		return None;
	}

	let suffix = file_name.strip_prefix("runtime-")?;
	let date_token = suffix.get(0..8)?;
	NaiveDate::parse_from_str(date_token, "%Y%m%d").ok()
}
