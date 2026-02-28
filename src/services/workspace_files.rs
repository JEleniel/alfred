//! Workspace-scoped filesystem policy and safe file I/O helpers.
//!
//! This module centralizes filesystem behavior that affects tool policy and
//! determinism (ignore rules, size caps, and UTF-8 validation). Other modules
//! should delegate disk reads/writes to these helpers so that policy and error
//! semantics stay consistent.

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};

pub use super::workspace_ignore::IndexIgnoreMatcher;

/// Files up to this size can be read directly into memory in one shot.
pub const INLINE_READ_BYTES: u64 = 4096;

/// Disk read block size used for streaming reads.
pub const IO_BLOCK_BYTES: usize = 4096;

/// Reads a UTF-8 file into a string, failing if it exceeds `max_bytes`.
pub fn read_utf8_text_capped(path: &Path, max_bytes: u64) -> Result<String> {
	let label = path.display().to_string();
	read_utf8_text_capped_labeled(path, label.as_str(), max_bytes)
}

/// Reads a UTF-8 file into a string, failing if it exceeds `max_bytes`.
///
/// Use `label` when you need deterministic, workspace-relative error messages.
pub fn read_utf8_text_capped_labeled(path: &Path, label: &str, max_bytes: u64) -> Result<String> {
	let bytes = read_bytes_capped(path, label, max_bytes)?;
	String::from_utf8(bytes).map_err(|_| anyhow!("file is not available as UTF-8 text: {label}"))
}

/// Reads an optional UTF-8 file into a string, failing if it exceeds `max_bytes`.
pub fn read_optional_utf8_text_capped(path: &Path, max_bytes: u64) -> Result<Option<String>> {
	let label = path.display().to_string();
	read_optional_utf8_text_capped_labeled(path, label.as_str(), max_bytes)
}

pub fn read_optional_utf8_text_capped_labeled(
	path: &Path,
	label: &str,
	max_bytes: u64,
) -> Result<Option<String>> {
	match fs::metadata(path) {
		Ok(metadata) => {
			if !metadata.is_file() {
				return Ok(None);
			}
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
		Err(error) => {
			return Err(error).with_context(|| format!("failed to read file metadata {label}"));
		}
	};

	read_utf8_text_capped_labeled(path, label, max_bytes).map(Some)
}

/// Best-effort read used by index snapshots: returns None when file is too large
/// or not valid UTF-8.
pub fn read_indexable_utf8_text(path: &Path, max_bytes: u64) -> Option<String> {
	let metadata = fs::metadata(path).ok()?;
	if !metadata.is_file() {
		return None;
	}
	if metadata.len() > max_bytes {
		return None;
	}

	let label = path.display().to_string();
	let bytes = read_bytes_capped(path, label.as_str(), max_bytes).ok()?;
	String::from_utf8(bytes).ok()
}

/// Reads an inclusive line range from `absolute_path` without loading the entire
/// file into memory.
///
/// Error messages are intentionally stable because tool-layer error mapping
/// depends on substrings like "file is not available as UTF-8 text".
pub fn read_text_file_range(
	absolute_path: &Path,
	normalized_path: &str,
	start_line: usize,
	end_line: usize,
) -> Result<String> {
	let metadata = indexed_file_metadata(absolute_path, normalized_path)?;
	if !metadata.is_file() {
		bail!("indexed file not found: {normalized_path}");
	}

	let file = open_indexed_file(absolute_path, normalized_path)?;
	let mut reader = BufReader::with_capacity(IO_BLOCK_BYTES, file);
	read_range_from_reader(&mut reader, normalized_path, start_line, end_line)
}

/// Streams UTF-8 lines from disk and invokes `visitor` for each line.
///
/// This is best-effort: missing files, I/O errors, or non-UTF8 content stop the scan
/// and return `Ok(())` so callers can fall back to cached/indexed content.
pub fn visit_utf8_lines_best_effort(
	absolute_path: &Path,
	mut visitor: impl FnMut(usize, &str),
) -> Result<()> {
	let file = match File::open(absolute_path) {
		Ok(file) => file,
		Err(_) => return Ok(()),
	};

	let mut reader = BufReader::with_capacity(IO_BLOCK_BYTES, file);
	let mut buffer = String::new();
	let mut current_line = 0usize;
	loop {
		buffer.clear();
		match reader.read_line(&mut buffer) {
			Ok(0) => break,
			Ok(_) => {}
			Err(_) => return Ok(()),
		}

		current_line += 1;
		let trimmed = buffer.trim_end_matches(['\n', '\r']);
		visitor(current_line, trimmed);
	}

	Ok(())
}

fn read_bytes_capped(path: &Path, label: &str, max_bytes: u64) -> Result<Vec<u8>> {
	let metadata =
		fs::metadata(path).with_context(|| format!("failed to read file metadata {label}"))?;
	validate_file_size(label, &metadata, max_bytes)?;
	if metadata.len() <= INLINE_READ_BYTES {
		return fs::read(path).with_context(|| format!("failed to read file {label}"));
	}

	let mut file = File::open(path).with_context(|| format!("failed to open file {label}"))?;
	read_file_in_blocks(label, &mut file, metadata.len(), max_bytes)
}

fn indexed_file_metadata(absolute_path: &Path, normalized_path: &str) -> Result<fs::Metadata> {
	match fs::metadata(absolute_path) {
		Ok(metadata) => Ok(metadata),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			bail!("indexed file not found: {normalized_path}");
		}
		Err(_) => {
			bail!("io error reading file metadata: {normalized_path}");
		}
	}
}

fn open_indexed_file(absolute_path: &Path, normalized_path: &str) -> Result<File> {
	match File::open(absolute_path) {
		Ok(file) => Ok(file),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			bail!("indexed file not found: {normalized_path}");
		}
		Err(_) => {
			bail!("io error reading file: {normalized_path}");
		}
	}
}

fn read_range_from_reader(
	reader: &mut BufReader<File>,
	normalized_path: &str,
	start_line: usize,
	end_line: usize,
) -> Result<String> {
	let mut output = String::new();
	let mut buffer = String::new();
	let mut current_line = 0usize;
	while current_line < end_line {
		buffer.clear();
		match reader.read_line(&mut buffer) {
			Ok(0) => break,
			Ok(_) => {}
			Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
				bail!("file is not available as UTF-8 text: {normalized_path}");
			}
			Err(_) => {
				bail!("io error reading file: {normalized_path}");
			}
		}

		current_line += 1;
		if current_line < start_line {
			continue;
		}

		push_line(&mut output, buffer.trim_end_matches(['\n', '\r']));
	}

	if end_line > current_line {
		bail!("requested line range is out of bounds for file: {normalized_path}");
	}

	Ok(output)
}

fn push_line(output: &mut String, line: &str) {
	if !output.is_empty() {
		output.push('\n');
	}
	output.push_str(line);
}

fn validate_file_size(label: &str, metadata: &fs::Metadata, max_bytes: u64) -> Result<()> {
	if !metadata.is_file() {
		bail!("not a file: {label}");
	}
	if metadata.len() > max_bytes {
		bail!("file exceeds configured size limit: {label}");
	}
	Ok(())
}

fn read_file_in_blocks(
	label: &str,
	file: &mut File,
	file_len: u64,
	max_bytes: u64,
) -> Result<Vec<u8>> {
	let mut output = Vec::with_capacity(
		std::cmp::min(file_len, max_bytes)
			.try_into()
			.unwrap_or(0usize),
	);
	let mut buffer = [0u8; IO_BLOCK_BYTES];
	let mut total = 0u64;
	loop {
		let read = file
			.read(&mut buffer)
			.with_context(|| format!("failed to read file {label}"))?;
		if read == 0 {
			break;
		}
		total = total.saturating_add(read as u64);
		if total > max_bytes {
			bail!("file exceeds configured size limit: {label}");
		}
		output.extend_from_slice(&buffer[..read]);
	}

	Ok(output)
}
