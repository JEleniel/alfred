use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use chrono::{Days, Utc};
use uuid::Uuid;
use zip::ZipArchive;

use crate::configuration::AppConfig;
use crate::logging::{prepare_runtime_log_directory, select_runtime_log_path};

struct TestDir {
	path: PathBuf,
}

impl TestDir {
	fn new() -> Self {
		let path = std::env::current_dir()
			.expect("workspace current_dir should resolve")
			.join("tmp")
			.join(format!("logging-tests-{}", Uuid::new_v4()));
		fs::create_dir_all(&path).expect("test directory should be created");
		Self { path }
	}
}

impl Drop for TestDir {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.path);
	}
}

#[test]
fn rotates_existing_runtime_log_into_zip_archive() {
	let test_dir = TestDir::new();
	let runtime_path = test_dir.path.join("runtime.ndjson");
	fs::write(&runtime_path, "first line\nsecond line\n")
		.expect("runtime log fixture should be written");

	prepare_runtime_log_directory(test_dir.path.as_path(), 7)
		.expect("runtime log directory should rotate");
	assert!(!runtime_path.exists());

	let archive_paths = fs::read_dir(&test_dir.path)
		.expect("test directory should be readable")
		.filter_map(|entry| entry.ok().map(|item| item.path()))
		.filter(|path| {
			path.file_name()
				.and_then(|name| name.to_str())
				.map(|name| name == "runtime.ndjson.zip")
				.unwrap_or(false)
		})
		.collect::<Vec<_>>();
	assert_eq!(archive_paths.len(), 1);

	let file = File::open(&archive_paths[0]).expect("archive should open");
	let mut archive = ZipArchive::new(file).expect("archive should be valid zip");
	let mut entry = archive
		.by_name("runtime.ndjson")
		.expect("zip should contain runtime.ndjson entry");
	let mut content = String::new();
	entry
		.read_to_string(&mut content)
		.expect("zip entry should be readable as text");
	assert_eq!(content, "first line\nsecond line\n");
}

#[test]
fn prunes_archives_older_than_retention_window() {
	let test_dir = TestDir::new();
	let today = Utc::now().date_naive();
	let old_date = today
		.checked_sub_days(Days::new(10))
		.expect("old date should be calculable");
	let recent_date = today
		.checked_sub_days(Days::new(2))
		.expect("recent date should be calculable");

	let old_archive = test_dir.path.join(format!(
		"alfred-{}T000000Z.ndjson.zip",
		old_date.format("%Y%m%d"),
	));
	let recent_archive = test_dir.path.join(format!(
		"alfred-{}T000000Z.ndjson.zip",
		recent_date.format("%Y%m%d"),
	));

	fs::write(&old_archive, b"old").expect("old archive fixture should be written");
	fs::write(&recent_archive, b"recent").expect("recent archive fixture should be written");

	prepare_runtime_log_directory(test_dir.path.as_path(), 7)
		.expect("retention pruning should succeed");
	assert!(!old_archive.exists());
	assert!(recent_archive.exists());
}

#[test]
fn startup_creates_new_timestamped_runtime_log_file() {
	let test_dir = TestDir::new();
	let workspace_root = test_dir.path.join("workspace");
	fs::create_dir_all(workspace_root.join(".alfred")).expect("workspace should be created");

	let override_dir = test_dir.path.join("runtime-logs");
	fs::write(
		workspace_root.join(".alfred").join("config.json"),
		format!(
			r#"{{"logging":{{"runtime":{{"path":"{}"}}}}}}"#,
			override_dir.to_string_lossy()
		),
	)
	.expect("workspace config should be written");

	let config = AppConfig::load_from_paths(
		workspace_root.clone(),
		workspace_root.join("missing-user-config.json"),
		workspace_root.join(".alfred").join("config.json"),
	)
	.expect("config should load from explicit paths");

	let runtime_log_path =
		select_runtime_log_path(&config).expect("runtime log path selection should succeed");
	assert!(runtime_log_path.starts_with(override_dir.as_path()));
	assert!(runtime_log_path.exists());

	let name = runtime_log_path
		.file_name()
		.and_then(|value| value.to_str())
		.expect("runtime log filename should be utf-8");
	assert!(name.starts_with("alfred-"));
	assert!(name.ends_with(".ndjson"));
	assert!(name.contains('T'));
	assert!(name.contains('Z'));
	assert!(!name.contains(':'));
}
