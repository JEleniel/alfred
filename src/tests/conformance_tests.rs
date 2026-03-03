use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use uuid::Uuid;

use crate::configuration::AppConfig;
use crate::protocol::handle_runtime_frame;
use crate::services::ServiceContainer;

#[cfg(all(unix, not(target_os = "macos")))]
use std::ffi::OsString;

#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::ffi::OsStringExt;

struct TestDir {
	path: PathBuf,
}

impl TestDir {
	fn new(prefix: &str) -> Self {
		let path = std::env::current_dir()
			.expect("workspace current dir should resolve")
			.join("tmp")
			.join(format!("{prefix}-{}", Uuid::new_v4()));
		fs::create_dir_all(&path).expect("test directory should be created");
		Self { path }
	}
}

impl Drop for TestDir {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.path);
	}
}

fn fixture_workspace() -> PathBuf {
	std::env::current_dir()
		.expect("workspace current dir should resolve")
		.join("src")
		.join("testdata")
		.join("indexer")
}

fn copy_fixture_tree(source: &Path, destination: &Path) {
	for entry in fs::read_dir(source).expect("fixture directory should be readable") {
		let entry = entry.expect("fixture entry should be valid");
		let source_path = entry.path();
		let destination_path = destination.join(entry.file_name());
		if source_path.is_dir() {
			fs::create_dir_all(&destination_path).expect("destination directory should be created");
			copy_fixture_tree(&source_path, &destination_path);
		} else {
			fs::copy(&source_path, &destination_path).expect("fixture file should copy");
		}
	}
}

fn build_services_for_fixture(with_index: bool) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("conformance-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	let services = ServiceContainer::new(config).expect("service container should build");
	if with_index {
		services
			.indexer
			.rebuild()
			.expect("fixture index should build");
	}

	(services, workspace)
}

fn build_services_with_disabled_tools(disabled_tools: Vec<String>) -> (ServiceContainer, TestDir) {
	let workspace = TestDir::new("conformance-policy-tests");
	copy_fixture_tree(fixture_workspace().as_path(), workspace.path.as_path());

	let mut config = AppConfig::load_from_paths(
		workspace.path.clone(),
		workspace.path.join("missing-user-config.json"),
		workspace.path.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");
	config.disabled_tools.extend(disabled_tools);
	config.disabled_tools.sort_unstable();
	config.disabled_tools.dedup();

	let services = ServiceContainer::new(config).expect("service container should build");
	(services, workspace)
}

fn runtime_tools_call(id: usize, name: &str, args: Value, services: &ServiceContainer) -> Value {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": id,
		"method": "tools/call",
		"params": {
			"name": name,
			"arguments": args,
		}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), services)
		.expect("runtime frame should parse")
		.expect("request should return response");
	serde_json::from_str(&response).expect("response should be valid JSON")
}

fn runtime_tools_list(id: usize, services: &ServiceContainer) -> Value {
	let frame = json!({
		"jsonrpc": "2.0",
		"id": id,
		"method": "tools/list",
		"params": {}
	});

	let response = handle_runtime_frame(frame.to_string().as_str(), services)
		.expect("runtime frame should parse")
		.expect("request should return response");
	serde_json::from_str(&response).expect("response should be valid JSON")
}

#[cfg(all(unix, not(target_os = "macos")))]
fn write_non_utf8_fixture_file(workspace_root: &Path) -> String {
	let mut bytes = b"bad-".to_vec();
	bytes.push(0xFF);
	bytes.extend_from_slice(b".txt");
	let filename = OsString::from_vec(bytes);
	let path = workspace_root.join(filename);
	fs::write(&path, "hello\n").expect("non-utf8 fixture file should write");
	"bad-\\xFF.txt".to_string()
}

#[test]
fn conformance_envelopes_cover_ok_error_and_pending() {
	let (services, _workspace) = build_services_for_fixture(true);

	let ok = runtime_tools_call(1, "workspace_dir", json!({}), &services);
	assert_eq!(ok["result"]["isError"], json!(false));
	assert_eq!(ok["result"]["structuredContent"]["status"], json!("ok"));

	let error = runtime_tools_call(2, "does_not_exist", json!({}), &services);
	assert_eq!(error["result"]["isError"], json!(true));
	assert_eq!(
		error["result"]["structuredContent"]["status"],
		json!("error")
	);
	assert_eq!(
		error["result"]["structuredContent"]["error"]["kind"],
		json!("invalid_argument")
	);

	let pending = runtime_tools_call(
		3,
		"fs",
		json!({
			"operation": "bulk",
			"dry_run": false,
			"args": {
				"mode": "execute",
				"run_in_background": true,
				"operations": [
					{"kind": "copy", "from": "alpha.txt", "to": "alpha-pending-copy.txt", "overwrite": false, "create_parents": false}
				]
			}
		}),
		&services,
	);
	assert_eq!(pending["result"]["isError"], json!(false));
	assert_eq!(
		pending["result"]["structuredContent"]["status"],
		json!("pending")
	);
	assert_eq!(
		pending["result"]["structuredContent"]["meta"]["transport_equivalent"]["http_status"],
		json!(202)
	);
}

#[test]
fn conformance_policy_gating_omits_and_refuses_disabled_tools() {
	let (services, _workspace) = build_services_with_disabled_tools(vec!["search".to_string()]);

	let list_payload = runtime_tools_list(10, &services);
	let names = list_payload["result"]["tools"]
		.as_array()
		.expect("tools/list should return tools")
		.iter()
		.filter_map(|entry| entry.get("name").and_then(Value::as_str))
		.collect::<Vec<_>>();
	assert!(!names.contains(&"search"));

	let capabilities = runtime_tools_call(11, "capabilities", json!({}), &services);
	let tool_names = capabilities["result"]["structuredContent"]["data"]["tools"]
		.as_array()
		.expect("capabilities should return tools")
		.iter()
		.filter_map(|entry| entry.get("name").and_then(Value::as_str))
		.collect::<Vec<_>>();
	assert!(!tool_names.contains(&"search"));

	let call = runtime_tools_call(12, "search", json!({"query": "hello"}), &services);
	assert_eq!(call["result"]["isError"], json!(true));
	assert_eq!(
		call["result"]["structuredContent"]["error"]["kind"],
		json!("invalid_argument")
	);
	assert_eq!(
		call["result"]["structuredContent"]["error"]["details"]["reason"],
		json!("tool_disabled")
	);
}

#[test]
fn conformance_search_reports_index_not_ready_and_index_disabled() {
	let (services_not_ready, _workspace) = build_services_for_fixture(false);
	let not_ready =
		runtime_tools_call(20, "search", json!({"query": "hello"}), &services_not_ready);
	assert_eq!(not_ready["result"]["isError"], json!(true));
	assert_eq!(
		not_ready["result"]["structuredContent"]["error"]["kind"],
		json!("tool_unavailable")
	);
	assert_eq!(
		not_ready["result"]["structuredContent"]["error"]["details"]["reason"],
		json!("index_not_ready")
	);

	let (mut services_disabled, _workspace) = build_services_for_fixture(true);
	services_disabled.config.index_enabled = false;
	let disabled = runtime_tools_call(21, "search", json!({"query": "hello"}), &services_disabled);
	assert_eq!(disabled["result"]["isError"], json!(true));
	assert_eq!(
		disabled["result"]["structuredContent"]["error"]["kind"],
		json!("tool_unavailable")
	);
	assert_eq!(
		disabled["result"]["structuredContent"]["error"]["details"]["reason"],
		json!("index_disabled")
	);
}

#[test]
fn conformance_logs_search_pagination_is_stable() {
	let (services, workspace) = build_services_for_fixture(false);
	let log_path = workspace.path.join(".alfred/logs/runtime.ndjson");
	if let Some(parent) = log_path.parent() {
		fs::create_dir_all(parent).expect("log parent should exist");
	}
	let content = [
		json!({"timestamp": "2026-02-23T10:00:00Z", "level": "INFO", "message": "build one", "source": "alfred::task", "extra": {}}),
		json!({"timestamp": "2026-02-23T10:01:00Z", "level": "INFO", "message": "build two", "source": "alfred::task", "extra": {}}),
		json!({"timestamp": "2026-02-23T10:02:00Z", "level": "INFO", "message": "build three", "source": "alfred::task", "extra": {}}),
	]
	.into_iter()
	.map(|record| serde_json::to_string(&record).expect("log record should serialize"))
	.collect::<Vec<_>>()
	.join("\n")
		+ "\n";
	fs::write(&log_path, content).expect("log file should write");

	let first = runtime_tools_call(
		30,
		"logs",
		json!({"operation": "search", "args": {"path": ".alfred/logs/runtime.ndjson", "query": "build", "limit": 2}}),
		&services,
	);
	let matches = first["result"]["structuredContent"]["data"]["result"]["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(matches.len(), 2);
	assert_eq!(
		first["result"]["structuredContent"]["data"]["result"]["next_cursor"],
		json!("2")
	);

	let second = runtime_tools_call(
		31,
		"logs",
		json!({"operation": "search", "args": {"path": ".alfred/logs/runtime.ndjson", "query": "build", "limit": 2, "cursor": "2"}}),
		&services,
	);
	let second_matches = second["result"]["structuredContent"]["data"]["result"]["matches"]
		.as_array()
		.expect("matches should be an array");
	assert_eq!(second_matches.len(), 1);
	assert!(second["result"]["structuredContent"]["data"]["result"]["next_cursor"].is_null());
}

#[test]
fn conformance_redaction_emits_warning_and_masks_output() {
	let (services, workspace) = build_services_for_fixture(false);
	fs::write(workspace.path.join("secret.txt"), "password=supersecret\n")
		.expect("secret fixture should write");

	let payload = runtime_tools_call(
		40,
		"fs",
		json!({"operation": "read_range", "args": {"path": "secret.txt", "start_line": 1, "end_line": 1}}),
		&services,
	);

	let warnings = payload["result"]["structuredContent"]["meta"]["warnings"]
		.as_array()
		.expect("warnings should be an array");
	assert!(
		warnings
			.iter()
			.any(|warning| warning["kind"] == json!("redaction"))
	);

	let text = payload["result"]["structuredContent"]["data"]["result"]["text"]
		.as_str()
		.expect("text should be present");
	assert!(!text.contains("supersecret"));
}

#[test]
#[cfg(all(unix, not(target_os = "macos")))]
fn conformance_path_encoding_emits_warning() {
	let (services, workspace) = build_services_for_fixture(false);
	let encoded_name = write_non_utf8_fixture_file(workspace.path.as_path());

	let payload = runtime_tools_call(
		50,
		"fs",
		json!({"operation": "search", "args": {"path": ".", "recursive": false}}),
		&services,
	);

	let files = payload["result"]["structuredContent"]["data"]["result"]["files"]
		.as_array()
		.expect("files should be an array")
		.iter()
		.filter_map(Value::as_str)
		.collect::<Vec<_>>();
	assert!(files.contains(&encoded_name.as_str()));

	let warnings = payload["result"]["structuredContent"]["meta"]["warnings"]
		.as_array()
		.expect("warnings should be an array");
	assert!(
		warnings
			.iter()
			.any(|warning| warning["kind"] == json!("path_encoded"))
	);
}
