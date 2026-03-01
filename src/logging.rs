//! Logging configuration with deterministic redaction.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use chrono::{Days, NaiveDate, Utc};
use fern::Dispatch;
use log::{LevelFilter, trace};
use serde_json::json;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::configuration::AppConfig;
use crate::redaction::Redactor;

static LOGGER_READY: OnceLock<()> = OnceLock::new();
static RUNTIME_LOG_PATH: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Initializes process-wide TRACE logging with NPI redaction.
pub fn init_logging(config: &AppConfig) -> Result<()> {
	if LOGGER_READY.get().is_some() {
		return Ok(());
	}

	let runtime_log_path = select_runtime_log_path(config)?;
	let _ = RUNTIME_LOG_PATH.set(runtime_log_path.clone());

	let redactor = Redactor::try_from_rules(
		config.redaction_enabled,
		config.redaction_replacement_token.clone(),
		config.redaction_preserve_length,
		config.redaction_rules.clone(),
	)?;
	configure_trace_logger(runtime_log_path.as_path(), redactor)?;
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

fn configure_trace_logger(runtime_log_path: &Path, redactor: Redactor) -> Result<()> {
	if let Some(parent) = runtime_log_path.parent() {
		fs::create_dir_all(parent).context("failed to create runtime log directory")?;
	}

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
			let line = render_log_record(
				timestamp.as_str(),
				record.level().to_string().as_str(),
				redacted.as_str(),
				record.target(),
			);
			out.finish(format_args!("{line}"));
		})
		.chain(file))
}

pub(crate) fn select_runtime_log_path(config: &AppConfig) -> Result<std::path::PathBuf> {
	let pid = std::process::id();
	for candidate in config.runtime_log_dir_candidates() {
		if ensure_dir_writable(candidate.as_path(), pid).is_err() {
			continue;
		}
		prepare_runtime_log_directory(candidate.as_path(), config.runtime_log_retention_days)?;
		let path = create_new_runtime_log_file(candidate.as_path())?;
		return Ok(path);
	}

	anyhow::bail!("no writable runtime log directory candidates")
}

pub(crate) fn runtime_log_path() -> Option<std::path::PathBuf> {
	RUNTIME_LOG_PATH.get().cloned()
}

pub(crate) fn prepare_runtime_log_directory(dir: &Path, retention_days: u64) -> Result<()> {
	fs::create_dir_all(dir).context("failed to create runtime log directory")?;
	archive_existing_runtime_logs(dir)?;
	prune_expired_archives(dir, retention_days)
}

fn ensure_dir_writable(dir: &Path, pid: u32) -> Result<()> {
	fs::create_dir_all(dir).context("failed to create runtime log directory")?;
	for counter in 0..10u32 {
		let probe = dir.join(format!(".alfred-writable-{pid}-{counter}.tmp"));
		match OpenOptions::new().write(true).create_new(true).open(&probe) {
			Ok(_) => {
				let _ = fs::remove_file(&probe);
				return Ok(());
			}
			Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
			Err(error) => return Err(error).context("runtime log directory is not writable"),
		}
	}

	anyhow::bail!("runtime log directory probe collided")
}

fn create_new_runtime_log_file(dir: &Path) -> Result<std::path::PathBuf> {
	let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
	for counter in 0..100u32 {
		let suffix = if counter == 0 {
			String::new()
		} else {
			format!("-{counter}")
		};
		let candidate = dir.join(format!("alfred-{timestamp}{suffix}.ndjson"));
		match OpenOptions::new()
			.write(true)
			.create_new(true)
			.open(&candidate)
		{
			Ok(_) => return Ok(candidate),
			Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
			Err(error) => {
				return Err(error).with_context(|| {
					format!(
						"failed to create runtime log file at {}",
						candidate.display()
					)
				});
			}
		}
	}

	anyhow::bail!("failed to create unique runtime log file")
}

fn archive_existing_runtime_logs(dir: &Path) -> Result<()> {
	for entry in fs::read_dir(dir)
		.with_context(|| format!("failed to read runtime log directory at {}", dir.display()))?
	{
		let entry = entry.context("failed to inspect runtime log directory entry")?;
		let path = entry.path();
		if !path.is_file() {
			continue;
		}
		let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
			continue;
		};
		if !is_runtime_ndjson_file(name) {
			continue;
		}
		archive_runtime_log_file(path.as_path())?;
	}

	Ok(())
}

fn is_runtime_ndjson_file(file_name: &str) -> bool {
	file_name == "runtime.ndjson"
		|| (file_name.starts_with("alfred-") && file_name.ends_with(".ndjson"))
}

fn archive_runtime_log_file(runtime_log_path: &Path) -> Result<()> {
	let source_name = runtime_log_path
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("runtime.ndjson")
		.to_string();
	let archive_path = next_archive_path(runtime_log_path, source_name.as_str())?;

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

fn next_archive_path(runtime_log_path: &Path, source_name: &str) -> Result<std::path::PathBuf> {
	let parent = runtime_log_path
		.parent()
		.context("runtime log path missing parent directory")?;
	let base = format!("{source_name}.zip");
	let preferred = parent.join(base.as_str());
	if !preferred.exists() {
		return Ok(preferred);
	}
	for counter in 1..100u32 {
		let candidate = parent.join(format!("{source_name}.{counter}.zip"));
		if !candidate.exists() {
			return Ok(candidate);
		}
	}
	anyhow::bail!("failed to create unique runtime archive name")
}

fn prune_expired_archives(parent: &Path, retention_days: u64) -> Result<()> {
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
	let is_supported = (file_name.starts_with("runtime-") || file_name.starts_with("alfred-"))
		&& file_name.ends_with(".ndjson.zip");
	if !is_supported {
		return None;
	}

	let suffix = file_name
		.strip_prefix("runtime-")
		.or_else(|| file_name.strip_prefix("alfred-"))?;
	let date_token = suffix.get(0..8)?;
	NaiveDate::parse_from_str(date_token, "%Y%m%d").ok()
}

fn render_log_record(timestamp: &str, level: &str, message: &str, source: &str) -> String {
	let record = json!({
		"timestamp": timestamp,
		"level": level,
		"message": message,
		"source": source,
		"extra": {},
	});
	record.to_string()
}
