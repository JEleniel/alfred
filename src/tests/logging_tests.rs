use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use chrono::{Days, Utc};
use uuid::Uuid;
use zip::ZipArchive;

use crate::logging::rotate_runtime_log;

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

	rotate_runtime_log(&runtime_path, 7).expect("runtime log should rotate");
	assert!(!runtime_path.exists());

	let archive_paths = fs::read_dir(&test_dir.path)
		.expect("test directory should be readable")
		.filter_map(|entry| entry.ok().map(|item| item.path()))
		.filter(|path| {
			path.file_name()
				.and_then(|name| name.to_str())
				.map(|name| name.starts_with("runtime-") && name.ends_with(".ndjson.zip"))
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
	let runtime_path = test_dir.path.join("runtime.ndjson");
	let today = Utc::now().date_naive();
	let old_date = today
		.checked_sub_days(Days::new(10))
		.expect("old date should be calculable");
	let recent_date = today
		.checked_sub_days(Days::new(2))
		.expect("recent date should be calculable");

	let old_archive = test_dir.path.join(format!(
		"runtime-{}T000000Z-{}.ndjson.zip",
		old_date.format("%Y%m%d"),
		Uuid::new_v4().simple()
	));
	let recent_archive = test_dir.path.join(format!(
		"runtime-{}T000000Z-{}.ndjson.zip",
		recent_date.format("%Y%m%d"),
		Uuid::new_v4().simple()
	));

	fs::write(&old_archive, b"old").expect("old archive fixture should be written");
	fs::write(&recent_archive, b"recent").expect("recent archive fixture should be written");

	rotate_runtime_log(&runtime_path, 7).expect("retention pruning should succeed");
	assert!(!old_archive.exists());
	assert!(recent_archive.exists());
}
