//! Workspace-scoped indexing ignore policy.
//!
//! This module implements `.gitignore`-style semantics for `.alfredignore` files:
//!
//! - Workspace root and per-directory `.alfredignore` files are applied additively.
//! - A user-level `.alfredignore` is applied before workspace rules.
//! - "Always excluded" patterns are applied first and cannot be overridden.
//!
//! The matching engine is intentionally centralized so that indexing and index-backed
//! tools can share deterministic behavior.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use anyhow::{Context, Result};
use ignore::Match as IgnoreMatch;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::warn;

const ALFREDIGNORE_FILENAME: &str = ".alfredignore";

/// Ignore patterns that MUST always be excluded from indexing.
///
/// These patterns cannot be overridden by whitelist rules in any `.alfredignore`.
const ALWAYS_EXCLUDED_PATTERNS: &[&str] = &[
	".git/", ".alfred/", ".agents/", // Executables and linkable artifacts.
	"*.a", "*.dll", "*.dylib", "*.exe", "*.lib", "*.o", "*.obj", "*.pdb", "*.rlib", "*.rmeta",
	"*.so", "*.wasm", "*.dSYM/",
	// Common binary formats (not useful for text indexing).
	"*.7z", "*.bin", "*.bz2", "*.dmg", "*.gz", "*.iso", "*.rar", "*.tar", "*.tgz", "*.xz", "*.zip",
	"*.zst", "*.pdf", // Images.
	"*.bmp", "*.gif", "*.icns", "*.ico", "*.jpeg", "*.jpg", "*.png", "*.svg", "*.svgz", "*.tif",
	"*.tiff", "*.webp",
];

/// Default, overridable ignore patterns applied when no `.alfredignore` exists.
const DEFAULT_ALFREDIGNORE_PATTERNS: &[&str] = &[
	// Rust
	".analyze/",
	"debug/",
	"mutants.out*/",
	"target/",
	"*.d",
	"**/*.rs.bk",
	"*.crate",
	"*.profraw",
	"*.profdata",
	"cargo-install-update-lock",
	"Cargo.lock",
	"Cargo.toml.orig",
	"crates-io-index/",
	// Node / JS
	"node_modules/",
	"npm-debug.log*",
	"yarn-debug.log*",
	"yarn-error.log*",
	"pnpm-debug.log*",
	".pnp.*",
	".yarn/",
	".pnpm-store/",
	"dist/",
	"build/",
	".next/",
	".nuxt/",
	".svelte-kit/",
	".turbo/",
	// Python
	"__pycache__/",
	"*.py[cod]",
	"*.pyo",
	".venv/",
	"venv/",
	".tox/",
	".mypy_cache/",
	".pytest_cache/",
	".ruff_cache/",
	".coverage",
	"coverage.xml",
	".nox/",
	"*.egg-info/",
	".eggs/",
	"pip-wheel-metadata/",
	// .NET
	"bin/",
	"obj/",
	".vs/",
	"TestResults/",
	"packages/",
	// Java / Kotlin
	".gradle/",
	".idea/",
	"*.iml",
	// Go
	"vendor/",
	"*.test",
	"coverage.out",
	// Elixir
	"_build/",
	"deps/",
	// Terraform
	".terraform/",
	"*.tfstate",
	"*.tfstate.*",
	// ----- Aurora model generated files -----
	"docs/design/README-MIS-*.md",
	"docs/design/MIS-*.md",
	"docs/design/MIS-*/**/*",
	"docs/design/aurora/MIS-*/Compact.json",
	// ----- End of Aurora model generated files -----
	// Secrets, credentials, and settings
	".env*",
	"!.env.example",
	"*.cert",
	"*.crt",
	"*.csr",
	"*.der",
	"*.jks",
	"*.key",
	"*.keystore",
	"*.p12",
	"*.pem",
	"*.pfx",
	"*.pvk",
	// Configuration files
	"config.json",
	"!config.default.json",
	"!config.example.json",
	"!config.schema.json",
	// Cache and temporary files
	".cache/",
	".temp/",
	"temp/",
	"tmp/",
	"*.tmp",
	"*.orig",
	"*.rej",
	// direnv
	".direnv/",
	// Logs and diagnostics
	"logs/",
	"*.log",
	"*-debug.log*",
	"*-error.log*",
	"report.[0-9]*.[0-9]*.[0-9]*.[0-9]*.json",
	// Test results & coverage output
	".nyc_output/",
	".coverage/",
	"coverage/",
	"*.lcov",
	"test-results/",
	// Runtime data
	"pids/",
	"*.pid",
	"*.pid.lock",
	"*.seed",
	// Diff/Patch files
	"*.diff",
	"*.patch",
	// Platform-specific cruft
	"*~",
	".directory",
	".fuse_hidden*",
	".nfs*",
	".Trash-*",
	"$RECYCLE.BIN/",
	"[Dd]esktop.ini",
	"ehthumbs.db",
	"ehthumbs_vista.db",
	"*.lnk",
	"*.stackdump",
	"Thumbs.db",
	"Thumbs.db:encryptable",
	"._*",
	".apdisk",
	".AppleDB",
	".AppleDesktop",
	".AppleDouble",
	".com.apple.timemachine.donotpresent",
	".DocumentRevisions-V100",
	".DS_Store",
	".fseventsd",
	".LSOverride",
	".Spotlight-V100",
	".TemporaryItems",
	".Trashes",
	".VolumeIcon.icns",
	"Icon",
	"Network Trash Folder",
	"Temporary Items",
];

/// Gitignore-style ignore matcher used for workspace indexing.
#[derive(Debug)]
pub struct IndexIgnoreMatcher {
	workspace_root: PathBuf,
	always: Arc<Gitignore>,
	defaults_and_user: Arc<Gitignore>,
	workspace_override: Option<Arc<Gitignore>>,
	ignore_file_cache: RwLock<HashMap<PathBuf, IgnoreFileCacheEntry>>,
}

impl IndexIgnoreMatcher {
	pub fn new(
		workspace_root: PathBuf,
		user_ignore_path: PathBuf,
		workspace_ignore_path: Option<PathBuf>,
	) -> Result<Self> {
		let always = build_matcher_from_patterns(&workspace_root, ALWAYS_EXCLUDED_PATTERNS)
			.context("failed to compile always-excluded index patterns")?;
		let defaults_and_user = build_defaults_and_user_matcher(&workspace_root, user_ignore_path)
			.context("failed to compile default index ignore patterns")?;
		let workspace_override = build_workspace_override_matcher(
			workspace_root.as_path(),
			workspace_ignore_path.as_deref(),
		);

		Ok(Self {
			workspace_root,
			always: Arc::new(always),
			defaults_and_user: Arc::new(defaults_and_user),
			workspace_override,
			ignore_file_cache: RwLock::new(HashMap::new()),
		})
	}

	pub fn should_ignore_relative(&self, relative_path: &str, is_dir: bool) -> bool {
		let Some(absolute_path) = self.absolute_path(relative_path) else {
			return false;
		};
		if self
			.always
			.matched_path_or_any_parents(absolute_path.as_path(), is_dir)
			.is_ignore()
		{
			return true;
		}

		let segments = split_segments(relative_path);
		if segments.is_empty() {
			return false;
		}

		let mut chain = Vec::<Arc<Gitignore>>::new();
		self.seed_chain(&mut chain);
		if self.any_parent_dir_ignored(segments.as_slice(), is_dir, &mut chain) {
			return true;
		}

		if is_dir {
			return false;
		}

		is_ignored_by_chain(chain.as_slice(), absolute_path.as_path(), false)
	}

	fn absolute_path(&self, relative_path: &str) -> Option<PathBuf> {
		if relative_path.trim().is_empty() {
			return None;
		}
		Some(self.workspace_root.join(relative_path))
	}

	fn seed_chain(&self, chain: &mut Vec<Arc<Gitignore>>) {
		chain.push(self.defaults_and_user.clone());
		if let Some(override_matcher) = &self.workspace_override {
			chain.push(override_matcher.clone());
		}
		if let Some(matcher) = self.ignore_matcher_for_dir(self.workspace_root.as_path()) {
			chain.push(matcher);
		}
	}

	fn any_parent_dir_ignored(
		&self,
		segments: &[&str],
		is_dir: bool,
		chain: &mut Vec<Arc<Gitignore>>,
	) -> bool {
		let directory_segments = if is_dir {
			segments.len()
		} else {
			segments.len().saturating_sub(1)
		};
		let mut current_dir = self.workspace_root.clone();
		for segment in segments.iter().take(directory_segments) {
			current_dir.push(segment);
			if is_ignored_by_chain(chain.as_slice(), current_dir.as_path(), true) {
				return true;
			}
			self.extend_chain_for_dir(current_dir.as_path(), chain);
		}
		false
	}

	fn extend_chain_for_dir(&self, dir: &Path, chain: &mut Vec<Arc<Gitignore>>) {
		if let Some(matcher) = self.ignore_matcher_for_dir(dir) {
			chain.push(matcher);
		}
	}

	fn ignore_matcher_for_dir(&self, dir: &Path) -> Option<Arc<Gitignore>> {
		let ignore_path = dir.join(ALFREDIGNORE_FILENAME);
		let metadata = match fs::metadata(ignore_path.as_path()) {
			Ok(metadata) => metadata,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				self.set_ignore_file_cache(ignore_path, None, None);
				return None;
			}
			Err(error) => {
				warn!(
					"failed to read .alfredignore metadata path={} error={error}",
					ignore_path.display()
				);
				return None;
			}
		};
		if !metadata.is_file() {
			self.set_ignore_file_cache(ignore_path, metadata.modified().ok(), None);
			return None;
		}

		let modified_at = metadata.modified().ok();
		if let Some(cached) = self.get_ignore_file_cache(ignore_path.as_path(), modified_at) {
			return cached;
		}

		let matcher = build_matcher_from_file(dir, ignore_path.as_path());
		self.set_ignore_file_cache(ignore_path, modified_at, matcher.clone());
		matcher
	}

	fn get_ignore_file_cache(
		&self,
		ignore_path: &Path,
		modified_at: Option<SystemTime>,
	) -> Option<Option<Arc<Gitignore>>> {
		let cache = self.ignore_file_cache.read().ok()?;
		let cached = cache.get(ignore_path)?;
		if cached.modified_at == modified_at {
			return Some(cached.matcher.clone());
		}
		None
	}

	fn set_ignore_file_cache(
		&self,
		ignore_path: PathBuf,
		modified_at: Option<SystemTime>,
		matcher: Option<Arc<Gitignore>>,
	) {
		let Ok(mut cache) = self.ignore_file_cache.write() else {
			return;
		};
		cache.insert(
			ignore_path,
			IgnoreFileCacheEntry {
				modified_at,
				matcher,
			},
		);
	}
}

fn build_workspace_override_matcher(
	root: &Path,
	ignore_path: Option<&Path>,
) -> Option<Arc<Gitignore>> {
	let ignore_path = ignore_path?;
	let content = match fs::read_to_string(ignore_path) {
		Ok(content) => content,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
		Err(error) => {
			warn!(
				"failed to read relocated workspace .alfredignore path={} error={error}",
				ignore_path.display()
			);
			return None;
		}
	};

	let mut builder = GitignoreBuilder::new(root);
	let from = Some(ignore_path.to_path_buf());
	for line in content.lines() {
		if let Err(error) = builder.add_line(from.clone(), line) {
			warn!(
				"failed to parse relocated workspace .alfredignore path={} error={error}",
				ignore_path.display()
			);
		}
	}

	match builder.build() {
		Ok(matcher) => Some(Arc::new(matcher)),
		Err(error) => {
			warn!(
				"failed to compile relocated workspace .alfredignore path={} error={error}",
				ignore_path.display()
			);
			None
		}
	}
}

#[derive(Debug, Clone)]
struct IgnoreFileCacheEntry {
	modified_at: Option<SystemTime>,
	matcher: Option<Arc<Gitignore>>,
}

fn split_segments(relative_path: &str) -> Vec<&str> {
	relative_path
		.split('/')
		.filter(|segment| !segment.is_empty())
		.collect::<Vec<_>>()
}

fn build_matcher_from_patterns(root: &Path, patterns: &[&str]) -> Result<Gitignore> {
	let mut builder = GitignoreBuilder::new(root);
	for pattern in patterns {
		builder
			.add_line(None, pattern)
			.with_context(|| format!("failed to compile ignore pattern: {pattern}"))?;
	}
	Ok(builder.build()?)
}

fn build_defaults_and_user_matcher(root: &Path, user_ignore_path: PathBuf) -> Result<Gitignore> {
	let mut builder = GitignoreBuilder::new(root);
	add_defaults(&mut builder)?;
	add_user_patterns(&mut builder, user_ignore_path.as_path());
	Ok(builder.build()?)
}

fn add_defaults(builder: &mut GitignoreBuilder) -> Result<()> {
	for pattern in DEFAULT_ALFREDIGNORE_PATTERNS {
		builder
			.add_line(None, pattern)
			.with_context(|| format!("failed to compile ignore pattern: {pattern}"))?;
	}
	Ok(())
}

fn add_user_patterns(builder: &mut GitignoreBuilder, user_ignore_path: &Path) {
	let content = match fs::read_to_string(user_ignore_path) {
		Ok(content) => Some(content),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
		Err(error) => {
			warn!(
				"failed to read user .alfredignore path={} error={error}",
				user_ignore_path.display()
			);
			None
		}
	};

	let Some(content) = content else {
		return;
	};

	let from = Some(user_ignore_path.to_path_buf());
	for line in content.lines() {
		if let Err(error) = builder.add_line(from.clone(), line) {
			warn!(
				"failed to parse user .alfredignore path={} error={error}",
				user_ignore_path.display()
			);
		}
	}
}

fn build_matcher_from_file(dir: &Path, ignore_path: &Path) -> Option<Arc<Gitignore>> {
	let mut builder = GitignoreBuilder::new(dir);
	if let Some(error) = builder.add(ignore_path) {
		warn!(
			"failed to parse .alfredignore path={} error={error}",
			ignore_path.display()
		);
	}
	match builder.build() {
		Ok(matcher) => Some(Arc::new(matcher)),
		Err(error) => {
			warn!(
				"failed to compile .alfredignore path={} error={error}",
				ignore_path.display()
			);
			None
		}
	}
}

fn is_ignored_by_chain(matchers: &[Arc<Gitignore>], absolute_path: &Path, is_dir: bool) -> bool {
	let mut last = None;
	for matcher in matchers {
		match matcher.matched(absolute_path, is_dir) {
			IgnoreMatch::Ignore(_) => last = Some(true),
			IgnoreMatch::Whitelist(_) => last = Some(false),
			IgnoreMatch::None => {}
		}
	}
	last.unwrap_or(false)
}
