use std::path::Path;

#[cfg(unix)]
use std::ffi::OsString;

#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use crate::path_encoding::{
	decode_relative_path, render_component, resolve_workspace_relative_path,
};

#[test]
#[cfg(unix)]
fn renders_non_utf8_bytes_as_xnn_and_round_trips() {
	let mut bytes = b"bad-".to_vec();
	bytes.push(0xFF);
	bytes.extend_from_slice(b".txt");
	let name = OsString::from_vec(bytes.clone());

	let rendered = render_component(name.as_os_str());
	assert!(rendered.was_encoded);
	assert_eq!(rendered.text, "bad-\\xFF.txt");

	let decoded = decode_relative_path(rendered.text.as_str());
	let decoded_name = decoded
		.file_name()
		.expect("decoded path should have a single segment");
	assert_eq!(decoded_name.as_bytes(), bytes.as_slice());
}

#[test]
fn resolve_prefers_direct_path_when_present() {
	let temp = std::env::current_dir()
		.expect("workspace current dir should resolve")
		.join("tmp")
		.join(format!("path-encoding-direct-{}", uuid::Uuid::new_v4()));
	std::fs::create_dir_all(&temp).expect("temp directory should exist");

	let filename = "bad-\\xFF.txt";
	let direct = temp.join(filename);
	std::fs::write(&direct, "hello\n").expect("file should write");

	let resolved = resolve_workspace_relative_path(temp.as_path(), filename);
	assert_eq!(resolved, direct);
	assert!(resolved.is_file());

	let _ = std::fs::remove_dir_all(&temp);
}

#[test]
fn inbound_separator_normalization_preserves_escape_sequences() {
	let input = r"dir\file";
	let normalized = crate::path_encoding::normalize_inbound_separators(input);
	assert_eq!(normalized, "dir/file");

	let input = r"dir\xFF\file";
	let normalized = crate::path_encoding::normalize_inbound_separators(input);
	assert_eq!(normalized, r"dir\xFF/file");

	let input = r"dir\u{D800}\file";
	let normalized = crate::path_encoding::normalize_inbound_separators(input);
	assert_eq!(normalized, r"dir\u{D800}/file");

	let input = r"..\secrets.txt";
	let normalized = crate::path_encoding::normalize_inbound_separators(input);
	assert_eq!(normalized, "../secrets.txt");

	assert!(crate::configuration::is_workspace_relative_path(
		"docs/design/ProjectPlan.md"
	));
	assert!(!crate::configuration::is_workspace_relative_path(
		"..\\secrets.txt"
	));
	assert!(!crate::configuration::is_workspace_relative_path(
		"../secrets.txt"
	));
}

#[test]
fn render_path_keeps_absolute_root() {
	let rendered = crate::path_encoding::render_path(Path::new("/"));
	assert_eq!(rendered.text, "/");
}
