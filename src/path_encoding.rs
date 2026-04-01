//! Deterministic rendering for filesystem paths that may not be valid Unicode.
//!
//! See `docs/design/Protocol.md`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenderedPath {
	pub text: String,
	pub was_encoded: bool,
}

/// Normalizes inbound path separators by treating `\` as `/` **except** when the backslash
/// appears to begin an encoded-path escape sequence (`\\`, `\xNN`, `\u{...}`).
pub fn normalize_inbound_separators(raw: &str) -> String {
	let mut output = String::with_capacity(raw.len());
	let mut chars = raw.chars().peekable();
	while let Some(ch) = chars.next() {
		if ch != '\\' {
			output.push(ch);
			continue;
		}

		match chars.peek().copied() {
			Some('\\' | 'x' | 'u') => output.push('\\'),
			_ => output.push('/'),
		}
	}

	output
}

/// Renders a path as a POSIX-separated string for protocol/tool output.
///
/// This does **not** enforce workspace-relative semantics; callers should validate separately.
pub fn render_relative_path(path: &Path) -> RenderedPath {
	let mut rendered = Vec::new();
	let mut was_encoded = false;

	for component in path.components() {
		use std::path::Component;
		match component {
			Component::CurDir => continue,
			Component::ParentDir => rendered.push("..".to_string()),
			Component::Normal(name) => {
				let part = render_component(name);
				was_encoded |= part.was_encoded;
				rendered.push(part.text);
			}
			Component::RootDir => {}
			Component::Prefix(prefix) => {
				let part = render_component(prefix.as_os_str());
				was_encoded |= part.was_encoded;
				rendered.push(part.text);
			}
		}
	}

	RenderedPath {
		text: rendered.join("/"),
		was_encoded,
	}
}

/// Renders an arbitrary path (absolute or relative) using POSIX separators.
///
/// Absolute Unix paths retain their leading `/`.
pub fn render_path(path: &Path) -> RenderedPath {
	let mut rendered = Vec::new();
	let mut was_encoded = false;
	let mut is_absolute = false;

	for component in path.components() {
		use std::path::Component;
		match component {
			Component::CurDir => continue,
			Component::ParentDir => rendered.push("..".to_string()),
			Component::Normal(name) => {
				let part = render_component(name);
				was_encoded |= part.was_encoded;
				rendered.push(part.text);
			}
			Component::RootDir => is_absolute = true,
			Component::Prefix(prefix) => {
				let part = render_component(prefix.as_os_str());
				was_encoded |= part.was_encoded;
				rendered.push(part.text);
			}
		}
	}

	let joined = rendered.join("/");
	let text = if is_absolute {
		if joined.is_empty() {
			"/".to_string()
		} else {
			format!("/{joined}")
		}
	} else {
		joined
	};

	RenderedPath { text, was_encoded }
}

pub fn render_component(component: &OsStr) -> RenderedPath {
	#[cfg(unix)]
	{
		use std::os::unix::ffi::OsStrExt;
		if let Some(text) = component.to_str() {
			return RenderedPath {
				text: text.to_string(),
				was_encoded: false,
			};
		}

		RenderedPath {
			text: encode_unix_bytes(component.as_bytes()),
			was_encoded: true,
		}
	}

	#[cfg(windows)]
	{
		use std::os::windows::ffi::OsStrExt;
		if let Some(text) = component.to_str() {
			return RenderedPath {
				text: text.to_string(),
				was_encoded: false,
			};
		}

		let mut output = String::new();
		let mut was_encoded = false;
		for unit in std::char::decode_utf16(component.encode_wide()) {
			match unit {
				Ok(ch) => output.push(ch),
				Err(error) => {
					was_encoded = true;
					output.push_str(&format!("\\u{{{:04X}}}", error.unpaired_surrogate()));
				}
			}
		}

		RenderedPath {
			text: output,
			was_encoded,
		}
	}

	#[cfg(not(any(unix, windows)))]
	{
		RenderedPath {
			text: component.to_string_lossy().to_string(),
			was_encoded: true,
		}
	}
}

#[cfg(unix)]
fn encode_unix_bytes(bytes: &[u8]) -> String {
	let mut output = String::new();
	let mut remaining = bytes;

	while !remaining.is_empty() {
		match std::str::from_utf8(remaining) {
			Ok(text) => {
				push_escaped_backslashes(&mut output, text);
				break;
			}
			Err(error) => {
				let valid = error.valid_up_to();
				if valid > 0 {
					if let Ok(prefix) = std::str::from_utf8(&remaining[..valid]) {
						push_escaped_backslashes(&mut output, prefix);
					} else {
						remaining = &remaining[valid..];
						continue;
					}
					remaining = &remaining[valid..];
					continue;
				}

				let invalid_len = error.error_len().unwrap_or(remaining.len()).max(1);
				let invalid_len = invalid_len.min(remaining.len());
				for byte in &remaining[..invalid_len] {
					if *byte == b'\\' {
						output.push_str("\\\\");
					} else {
						output.push_str(&format!("\\x{:02X}", byte));
					}
				}
				remaining = &remaining[invalid_len..];
			}
		}
	}

	output
}

#[cfg(unix)]
fn push_escaped_backslashes(output: &mut String, text: &str) {
	for ch in text.chars() {
		if ch == '\\' {
			output.push_str("\\\\");
		} else {
			output.push(ch);
		}
	}
}

/// Resolves a workspace-relative protocol path to an absolute filesystem path.
///
/// This is intentionally forgiving: we first try the literal path, and only decode encoded
/// escape sequences if the literal path does not exist.
pub fn resolve_workspace_relative_path(workspace_root: &Path, relative: &str) -> PathBuf {
	let direct = workspace_root.join(relative);
	if std::fs::symlink_metadata(&direct).is_ok() {
		return direct;
	}

	let decoded = decode_relative_path(relative);
	workspace_root.join(decoded)
}

pub fn decode_relative_path(relative: &str) -> PathBuf {
	let mut path = PathBuf::new();
	for segment in relative.split('/') {
		if segment.is_empty() || segment == "." {
			continue;
		}
		if segment == ".." {
			path.pop();
			continue;
		}

		path.push(decode_segment(segment));
	}
	path
}

fn decode_segment(segment: &str) -> std::ffi::OsString {
	#[cfg(unix)]
	{
		use std::os::unix::ffi::OsStringExt;
		std::ffi::OsString::from_vec(decode_unix_segment_bytes(segment.as_bytes()))
	}

	#[cfg(windows)]
	{
		use std::os::windows::ffi::OsStringExt;
		std::ffi::OsString::from_wide(&decode_windows_segment_units(segment))
	}

	#[cfg(not(any(unix, windows)))]
	{
		std::ffi::OsString::from(segment)
	}
}

#[cfg(unix)]
fn decode_unix_segment_bytes(bytes: &[u8]) -> Vec<u8> {
	let mut output = Vec::with_capacity(bytes.len());
	let mut index = 0usize;
	while index < bytes.len() {
		if bytes[index] != b'\\' {
			output.push(bytes[index]);
			index += 1;
			continue;
		}

		if index + 1 < bytes.len() && bytes[index + 1] == b'\\' {
			output.push(b'\\');
			index += 2;
			continue;
		}

		if index + 3 < bytes.len()
			&& bytes[index + 1] == b'x'
			&& let (Some(hi), Some(lo)) = (hex_value(bytes[index + 2]), hex_value(bytes[index + 3]))
		{
			output.push((hi << 4) | lo);
			index += 4;
			continue;
		}

		output.push(b'\\');
		index += 1;
	}
	output
}

#[cfg(unix)]
fn hex_value(byte: u8) -> Option<u8> {
	match byte {
		b'0'..=b'9' => Some(byte - b'0'),
		b'a'..=b'f' => Some(10 + (byte - b'a')),
		b'A'..=b'F' => Some(10 + (byte - b'A')),
		_ => None,
	}
}

#[cfg(windows)]
fn decode_windows_segment_units(segment: &str) -> Vec<u16> {
	let mut output = Vec::new();
	let bytes = segment.as_bytes();
	let mut index = 0usize;
	while index < bytes.len() {
		if bytes[index] == b'\\'
			&& index + 3 < bytes.len()
			&& bytes[index + 1] == b'u'
			&& bytes[index + 2] == b'{'
		{
			if let Some(end) = segment[index + 3..].find('}') {
				let hex = &segment[index + 3..index + 3 + end];
				if let Ok(unit) = u16::from_str_radix(hex, 16) {
					output.push(unit);
					index += 3 + end + 1;
					continue;
				}
			}
		}

		let ch = segment[index..].chars().next().unwrap();
		let mut buf = [0u16; 2];
		let encoded = ch.encode_utf16(&mut buf);
		output.extend_from_slice(encoded);
		index += ch.len_utf8();
	}
	output
}
