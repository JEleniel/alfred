use std::path::PathBuf;

use crate::services::indexer::{SearchMatch, WorkspaceIndexer};

fn fixture_root() -> PathBuf {
	std::env::current_dir()
		.expect("workspace root should resolve")
		.join("src")
		.join("testdata")
		.join("indexer")
}

#[test]
fn rebuild_indexes_workspace_files_and_directories() {
	let indexer = WorkspaceIndexer::new(fixture_root());
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
	let indexer = WorkspaceIndexer::new(fixture_root());
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
	let indexer = WorkspaceIndexer::new(fixture_root());
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
	let indexer = WorkspaceIndexer::new(fixture_root());
	indexer.rebuild().expect("index rebuild should succeed");

	let text = indexer
		.read_range("alpha.txt", 1, 2)
		.expect("read_range should succeed");
	assert_eq!(text, "Hello from alpha\nSecond line");
}
