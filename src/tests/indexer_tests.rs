use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use uuid::Uuid;

use crate::redaction::Redactor;
use crate::services::indexer::{SearchMatch, WorkspaceIndexer};

struct TestDir {
	path: PathBuf,
}

impl TestDir {
	fn new() -> Self {
		let path = std::env::current_dir()
			.expect("workspace root should resolve")
			.join("tmp")
			.join(format!("indexer-watch-tests-{}", Uuid::new_v4()));
		fs::create_dir_all(&path).expect("test directory should be created");
		Self { path }
	}
}

impl Drop for TestDir {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.path);
	}
}

fn fixture_root() -> PathBuf {
	std::env::current_dir()
		.expect("workspace root should resolve")
		.join("src")
		.join("testdata")
		.join("indexer")
}

fn new_test_indexer(workspace_root: PathBuf, index_root: PathBuf) -> WorkspaceIndexer {
	let user_ignore_path = index_root.join("missing-user-alfredignore");
	let redactor = Arc::new(Redactor::try_default().expect("redactor should initialize"));
	WorkspaceIndexer::new_with_options(
		workspace_root,
		index_root,
		Duration::from_millis(120),
		Duration::from_millis(80),
		user_ignore_path,
		None,
		redactor,
	)
	.expect("indexer should construct")
}

#[test]
fn rebuild_indexes_workspace_files_and_directories() {
	let storage = TestDir::new();
	let indexer = new_test_indexer(fixture_root(), storage.path.clone());
	let stats = indexer.rebuild().expect("index rebuild should succeed");

	assert_eq!(stats.indexed_files, 3);
	assert_eq!(stats.indexed_directories, 1);
	assert_eq!(stats.indexed_text_files, 3);

	assert_eq!(
		indexer.list_files(),
		vec![
			"alpha.txt".to_string(),
			"nested/beta.md".to_string(),
			"Zeta.txt".to_string(),
		]
	);
	assert_eq!(indexer.list_directories(), vec!["nested".to_string()]);
}

#[test]
fn grep_literal_finds_matches_with_casefold_sorting() {
	let storage = TestDir::new();
	let indexer = new_test_indexer(fixture_root(), storage.path.clone());
	indexer.rebuild().expect("index rebuild should succeed");

	let matches = indexer.grep_literal("hello", false);
	assert_eq!(
		matches,
		vec![
			SearchMatch {
				path: "alpha.txt".to_string(),
				line: 1,
				text: "Hello from alpha".to_string(),
			},
			SearchMatch {
				path: "nested/beta.md".to_string(),
				line: 1,
				text: "Hello from beta".to_string(),
			},
		]
	);
}

#[test]
fn search_regex_finds_expected_lines() {
	let storage = TestDir::new();
	let indexer = new_test_indexer(fixture_root(), storage.path.clone());
	indexer.rebuild().expect("index rebuild should succeed");

	let matches = indexer
		.search_regex("search\\s+me", false)
		.expect("regex search should succeed");
	assert_eq!(matches.len(), 1);
	assert_eq!(matches[0].path, "nested/beta.md");
	assert_eq!(matches[0].line, 2);
	assert_eq!(matches[0].text, "search me please");
}

#[test]
fn read_range_returns_requested_lines() {
	let storage = TestDir::new();
	let indexer = new_test_indexer(fixture_root(), storage.path.clone());
	indexer.rebuild().expect("index rebuild should succeed");

	let text = indexer
		.read_range("alpha.txt", 1, 2)
		.expect("read_range should succeed");
	assert_eq!(text, "Hello from alpha\nSecond line");
}

#[test]
fn watch_thread_starts_only_once() {
	let storage = TestDir::new();
	let indexer = new_test_indexer(fixture_root(), storage.path.clone());
	indexer.rebuild().expect("index rebuild should succeed");

	let first = indexer
		.start_watch_thread()
		.expect("first watch-thread start should succeed");
	let second = indexer
		.start_watch_thread()
		.expect("second watch-thread start call should succeed");

	assert!(first);
	assert!(!second);
	indexer.stop_watch_thread();
}

#[test]
fn watch_thread_reindexes_on_save() {
	let test_dir = TestDir::new();
	let storage = TestDir::new();
	let file_path = test_dir.path.join("watched.txt");
	fs::write(&file_path, "one\ntwo\n").expect("fixture file should be written");

	let indexer = new_test_indexer(test_dir.path.clone(), storage.path.clone());
	indexer.rebuild().expect("index rebuild should succeed");
	indexer
		.start_watch_thread()
		.expect("watch thread should start");

	thread::sleep(Duration::from_millis(200));
	fs::write(&file_path, "updated\ntwo\n").expect("fixture file should be updated");

	let mut updated = false;
	for _ in 0..30 {
		thread::sleep(Duration::from_millis(150));
		if let Ok(text) = indexer.read_range("watched.txt", 1, 1)
			&& text == "updated"
		{
			updated = true;
			break;
		}
	}

	indexer.stop_watch_thread();
	assert!(updated, "watch thread should reindex saved file changes");
}

#[cfg(unix)]
#[test]
fn watch_thread_does_not_index_symlink_escape_paths() {
	use std::os::unix::fs::symlink;

	let workspace = TestDir::new();
	let storage = TestDir::new();
	let outside = TestDir::new();
	let outside_file = outside.path.join("outside.txt");
	fs::write(&outside_file, "outside\n").expect("outside fixture file should write");

	let indexer = new_test_indexer(workspace.path.clone(), storage.path.clone());
	indexer.rebuild().expect("index rebuild should succeed");
	indexer
		.start_watch_thread()
		.expect("watch thread should start");

	thread::sleep(Duration::from_millis(200));
	let link_path = workspace.path.join("escape.txt");
	symlink(&outside_file, &link_path).expect("symlink should be created");

	for _ in 0..25 {
		thread::sleep(Duration::from_millis(120));
		if indexer.list_files().contains(&"escape.txt".to_string()) {
			break;
		}
	}

	indexer.stop_watch_thread();
	assert!(
		!indexer.list_files().contains(&"escape.txt".to_string()),
		"symlink escape path must not be indexed"
	);

	let error = indexer
		.read_range("escape.txt", 1, 1)
		.expect_err("read_range through escape symlink must be denied");
	assert!(
		error
			.to_string()
			.contains(crate::workspace_boundary::SYMLINK_JUNCTION_ESCAPE_MESSAGE),
		"unexpected error: {error:#}"
	);
}

#[test]
fn initialize_loads_persisted_index_for_workspace() {
	let workspace = TestDir::new();
	let storage = TestDir::new();
	let file_path = workspace.path.join("persisted.txt");
	fs::write(&file_path, "persist me\n").expect("workspace file should be written");

	let first = new_test_indexer(workspace.path.clone(), storage.path.clone());
	first.rebuild().expect("first index build should succeed");
	drop(first);

	fs::remove_file(&file_path).expect("workspace file should be removed");

	let second = new_test_indexer(workspace.path.clone(), storage.path.clone());
	second
		.initialize()
		.expect("index should load from persisted storage");

	assert_eq!(second.list_files(), vec!["persisted.txt".to_string()]);
}

#[test]
fn initialize_rebuilds_when_persistence_root_changes() {
	let workspace = TestDir::new();
	let storage_a = TestDir::new();
	let storage_b = TestDir::new();
	let file_path = workspace.path.join("persisted.txt");
	fs::write(&file_path, "persist me\n").expect("workspace file should be written");

	let first = new_test_indexer(workspace.path.clone(), storage_a.path.clone());
	first.rebuild().expect("first index build should succeed");
	drop(first);

	fs::remove_file(&file_path).expect("workspace file should be removed");

	let second = new_test_indexer(workspace.path.clone(), storage_b.path.clone());
	second
		.initialize()
		.expect("index initialize should rebuild when storage is empty");

	assert!(
		second.list_files().is_empty(),
		"index should not consult persisted storage from other location"
	);
}
