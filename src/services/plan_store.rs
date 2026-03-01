//! Plan storage service implementation.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::errors::AlfredError;

/// Canonical plan item status values.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanStatus {
	Planned,
	InProgress,
	Completed,
	Cancelled,
}

/// Canonical project plan item schema.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanItem {
	pub id: u64,
	pub title: String,
	pub priority: u8,
	pub cards: Vec<String>,
	pub description: String,
	pub deliverables: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub acceptance_criteria: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub notes: Option<String>,
	pub status: PlanStatus,
}

#[derive(Debug, Clone)]
struct PlanDocument {
	preamble: String,
	items: Vec<PlanItem>,
}

/// Reads and writes the project plan artifact.
#[derive(Debug, Clone)]
pub struct PlanStore {
	workspace_root: PathBuf,
	plan_path: PathBuf,
}

impl PlanStore {
	/// Creates a plan store bound to a workspace and plan file path.
	pub fn new(workspace_root: PathBuf, plan_path: PathBuf) -> Self {
		Self {
			workspace_root,
			plan_path,
		}
	}

	fn ensure_plan_is_within_workspace(&self) -> Result<(), AlfredError> {
		if self.plan_path.starts_with(&self.workspace_root) {
			return Ok(());
		}

		Err(AlfredError::WorkspaceBoundaryViolation(format!(
			"plan path is outside workspace boundary: {}",
			self.plan_path.display()
		)))
	}

	/// Returns the configured plan path.
	pub fn plan_path(&self) -> &Path {
		&self.plan_path
	}

	/// Reads and returns canonical plan items.
	pub fn read_items(&self) -> Result<Vec<PlanItem>, AlfredError> {
		let document = self.read_document()?;
		Ok(document.items)
	}

	/// Updates the status of a single plan item.
	pub fn update_status(&self, id: u64, status: PlanStatus) -> Result<(), AlfredError> {
		let mut document = self.read_document()?;
		let Some(item) = document
			.items
			.iter_mut()
			.find(|candidate| candidate.id == id)
		else {
			return Err(AlfredError::NotFound(format!(
				"plan item id not found: {id}"
			)));
		};
		item.status = status;
		self.write_document(&document)
	}

	/// Replaces a full plan item while preserving stable IDs.
	pub fn edit_item(&self, item: PlanItem) -> Result<(), AlfredError> {
		validate_item(&item)?;
		let mut document = self.read_document()?;
		let Some(index) = document
			.items
			.iter()
			.position(|candidate| candidate.id == item.id)
		else {
			return Err(AlfredError::NotFound(format!(
				"plan item id not found: {}",
				item.id
			)));
		};
		document.items[index] = item;
		self.write_document(&document)
	}

	/// Appends a new plan item with server-assigned id.
	pub fn add_item(&self, mut item: PlanItem) -> Result<u64, AlfredError> {
		validate_item_for_add(&item)?;
		let mut document = self.read_document()?;
		let next_id = document
			.items
			.iter()
			.map(|existing| existing.id)
			.max()
			.unwrap_or(0)
			+ 1;
		item.id = next_id;
		document.items.push(item.clone());
		self.write_document(&document)?;
		Ok(next_id)
	}

	/// Deletes a plan item by id.
	pub fn delete_item(&self, id: u64) -> Result<(), AlfredError> {
		let mut document = self.read_document()?;
		let starting_len = document.items.len();
		document.items.retain(|item| item.id != id);
		if document.items.len() == starting_len {
			return Err(AlfredError::NotFound(format!(
				"plan item id not found: {id}"
			)));
		}

		self.write_document(&document)
	}

	fn read_document(&self) -> Result<PlanDocument, AlfredError> {
		self.ensure_plan_is_within_workspace()?;
		let read_path =
			match crate::workspace_boundary::try_resolve_existing_path_within_workspace_root(
				self.workspace_root.as_path(),
				self.plan_path.as_path(),
			)? {
				Some(resolved) => resolved,
				None => self.plan_path.clone(),
			};

		let raw = fs::read_to_string(read_path.as_path()).map_err(|error| match error.kind() {
			std::io::ErrorKind::NotFound => {
				AlfredError::NotFound(format!("plan file not found: {}", self.plan_path.display()))
			}
			_ => AlfredError::IoError(format!(
				"failed to read plan file {}: {error}",
				self.plan_path.display()
			)),
		})?;

		parse_plan_document(raw.as_str()).map_err(|message| {
			AlfredError::InvalidArgument(format!(
				"failed to parse plan file {}: {message}",
				self.plan_path.display()
			))
		})
	}

	fn write_document(&self, document: &PlanDocument) -> Result<(), AlfredError> {
		self.ensure_plan_is_within_workspace()?;
		let plan_path = crate::workspace_boundary::resolve_write_target_within_workspace_root(
			self.workspace_root.as_path(),
			self.plan_path.as_path(),
		)?;

		let parent = plan_path.parent().ok_or_else(|| {
			AlfredError::Internal(format!(
				"plan path has no parent directory: {}",
				plan_path.display()
			))
		})?;
		fs::create_dir_all(parent).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to create plan directory {}: {error}",
				parent.display()
			))
		})?;

		let rendered = render_plan_document(document);
		let temp_path = plan_path.with_extension(format!("tmp-{}", std::process::id()));

		let mut temp_file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(&temp_path)
			.map_err(|error| {
				AlfredError::IoError(format!(
					"failed to open temporary plan file {}: {error}",
					temp_path.display()
				))
			})?;

		temp_file.write_all(rendered.as_bytes()).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to write temporary plan file {}: {error}",
				temp_path.display()
			))
		})?;
		temp_file.sync_all().map_err(|error| {
			AlfredError::IoError(format!(
				"failed to sync temporary plan file {}: {error}",
				temp_path.display()
			))
		})?;

		fs::rename(&temp_path, plan_path.as_path()).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to replace plan file {}: {error}",
				self.plan_path.display()
			))
		})
	}
}

fn parse_plan_document(raw: &str) -> Result<PlanDocument, String> {
	let lines = raw.lines().collect::<Vec<_>>();
	let item_regex = Regex::new(r"^(\d+)\.\s+\[( |x|X)\]\s+(.+)$")
		.map_err(|error| format!("failed to compile plan parser regex: {error}"))?;

	let item_starts = lines
		.iter()
		.enumerate()
		.filter_map(|(index, line)| item_regex.is_match(line).then_some(index))
		.collect::<Vec<_>>();

	if item_starts.is_empty() {
		let preamble = raw.trim().to_string();
		return Ok(PlanDocument {
			preamble,
			items: Vec::new(),
		});
	}

	let preamble = lines[..item_starts[0]].join("\n").trim().to_string();
	let mut items = Vec::new();
	for (position, start) in item_starts.iter().enumerate() {
		let end = item_starts
			.get(position + 1)
			.copied()
			.unwrap_or(lines.len());
		let block = lines[*start..end].to_vec();
		let item = parse_plan_item(block, &item_regex)?;
		items.push(item);
	}

	items.sort_by_key(|item| item.id);
	validate_unique_ids(items.as_slice())?;

	Ok(PlanDocument { preamble, items })
}

fn parse_plan_item(lines: Vec<&str>, item_regex: &Regex) -> Result<PlanItem, String> {
	if lines.is_empty() {
		return Err("unexpected empty plan item block".to_string());
	}

	let heading_captures = item_regex
		.captures(lines[0])
		.ok_or_else(|| format!("invalid plan heading: {}", lines[0]))?;
	let id = heading_captures
		.get(1)
		.ok_or_else(|| "missing plan id".to_string())?
		.as_str()
		.parse::<u64>()
		.map_err(|error| format!("invalid plan id: {error}"))?;
	let checked = heading_captures
		.get(2)
		.map(|capture| capture.as_str().eq_ignore_ascii_case("x"))
		.unwrap_or(false);
	let title = heading_captures
		.get(3)
		.ok_or_else(|| "missing plan title".to_string())?
		.as_str()
		.trim()
		.to_string();

	let mut priority: Option<u8> = None;
	let mut cards: Option<Vec<String>> = None;
	let mut description: Option<String> = None;
	let mut deliverables = Vec::new();
	let mut acceptance_criteria: Option<String> = None;
	let mut notes: Option<String> = None;
	let mut status: Option<PlanStatus> = None;
	let mut in_deliverables = false;

	for line in lines.iter().skip(1) {
		if line.trim().is_empty() {
			continue;
		}

		if in_deliverables {
			if let Some(deliverable) = line.strip_prefix("        - ") {
				deliverables.push(deliverable.trim().to_string());
				continue;
			}
			if !line.starts_with("    - ") {
				return Err(format!("invalid deliverable line for item {id}: {line}"));
			}
			in_deliverables = false;
		}

		let Some(body) = line.strip_prefix("    - ") else {
			return Err(format!(
				"invalid plan field indentation for item {id}: {line}"
			));
		};

		if body.eq_ignore_ascii_case("Deliverables:") {
			in_deliverables = true;
			continue;
		}

		let Some((raw_key, raw_value)) = body.split_once(':') else {
			return Err(format!("invalid plan field for item {id}: {line}"));
		};
		let key = raw_key.trim().to_ascii_lowercase();
		let value = raw_value.trim();

		match key.as_str() {
			"priority" => {
				let parsed = value
					.parse::<u8>()
					.map_err(|error| format!("invalid priority for item {id}: {error}"))?;
				priority = Some(parsed);
			}
			"cards" => {
				cards = Some(parse_cards(value));
			}
			"description" => {
				description = Some(value.to_string());
			}
			"acceptance criteria" => {
				acceptance_criteria = non_empty_option(value);
			}
			"notes" => {
				notes = non_empty_option(value);
			}
			"status" => {
				status = Some(parse_status(value)?);
			}
			_ => {
				return Err(format!("unknown plan field for item {id}: {raw_key}"));
			}
		}
	}

	let status = status.unwrap_or(if checked {
		PlanStatus::Completed
	} else {
		PlanStatus::Planned
	});

	let item = PlanItem {
		id,
		title,
		priority: priority.ok_or_else(|| format!("missing priority for item {id}"))?,
		cards: cards.ok_or_else(|| format!("missing cards for item {id}"))?,
		description: description.ok_or_else(|| format!("missing description for item {id}"))?,
		deliverables,
		acceptance_criteria,
		notes,
		status,
	};

	validate_item(&item).map_err(|error| match error {
		AlfredError::InvalidArgument(message) => message,
		other => other.to_string(),
	})?;

	Ok(item)
}

fn parse_cards(raw: &str) -> Vec<String> {
	raw.split(',')
		.map(|token| {
			token
				.trim()
				.trim_matches('"')
				.trim_matches('\'')
				.to_string()
		})
		.filter(|token| !token.is_empty())
		.collect()
}

fn parse_status(raw: &str) -> Result<PlanStatus, String> {
	match raw.trim().to_ascii_lowercase().as_str() {
		"planned" => Ok(PlanStatus::Planned),
		"in-progress" => Ok(PlanStatus::InProgress),
		"completed" => Ok(PlanStatus::Completed),
		"cancelled" => Ok(PlanStatus::Cancelled),
		other => Err(format!("invalid status: {other}")),
	}
}

fn non_empty_option(value: &str) -> Option<String> {
	let trimmed = value.trim();
	(!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn render_plan_document(document: &PlanDocument) -> String {
	let mut rendered = String::new();
	let mut items = document.items.clone();
	items.sort_by_key(|item| item.id);

	if !document.preamble.trim().is_empty() {
		rendered.push_str(document.preamble.trim());
		rendered.push_str("\n\n");
	}

	for (index, item) in items.iter().enumerate() {
		if index > 0 {
			rendered.push('\n');
		}
		rendered.push_str(render_item(item).as_str());
	}

	if !rendered.ends_with('\n') {
		rendered.push('\n');
	}

	rendered
}

fn render_item(item: &PlanItem) -> String {
	let mut output = String::new();
	let checkbox = if matches!(item.status, PlanStatus::Completed) {
		"x"
	} else {
		" "
	};
	output.push_str(format!("{}. [{}] {}\n", item.id, checkbox, item.title.trim()).as_str());
	output.push_str(format!("    - Priority: {}\n", item.priority).as_str());
	output.push_str(
		format!(
			"    - Cards: {}\n",
			item.cards
				.iter()
				.map(|card| format!("\"{card}\""))
				.collect::<Vec<_>>()
				.join(", ")
		)
		.as_str(),
	);
	output.push_str(format!("    - Description: {}\n", item.description.trim()).as_str());
	output.push_str("    - Deliverables:\n");
	for deliverable in &item.deliverables {
		output.push_str(format!("        - {}\n", deliverable.trim()).as_str());
	}
	if let Some(acceptance_criteria) = &item.acceptance_criteria {
		output.push_str(
			format!(
				"    - Acceptance Criteria: {}\n",
				acceptance_criteria.trim()
			)
			.as_str(),
		);
	}
	if let Some(notes) = &item.notes {
		output.push_str(format!("    - Notes: {}\n", notes.trim()).as_str());
	}
	output.push_str(format!("    - Status: {}\n", render_status(&item.status)).as_str());
	output
}

fn render_status(status: &PlanStatus) -> &'static str {
	match status {
		PlanStatus::Planned => "planned",
		PlanStatus::InProgress => "in-progress",
		PlanStatus::Completed => "completed",
		PlanStatus::Cancelled => "cancelled",
	}
}

fn validate_unique_ids(items: &[PlanItem]) -> Result<(), String> {
	let mut ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
	ids.sort_unstable();
	for pair in ids.windows(2) {
		if pair[0] == pair[1] {
			return Err(format!("duplicate plan item id: {}", pair[0]));
		}
	}

	Ok(())
}

fn validate_item(item: &PlanItem) -> Result<(), AlfredError> {
	if item.id == 0 {
		return Err(AlfredError::InvalidArgument(
			"id must be greater than 0".to_string(),
		));
	}
	validate_item_common(item)
}

fn validate_item_for_add(item: &PlanItem) -> Result<(), AlfredError> {
	validate_item_common(item)
}

fn validate_item_common(item: &PlanItem) -> Result<(), AlfredError> {
	if item.title.trim().is_empty() {
		return Err(AlfredError::InvalidArgument(
			"title must not be empty".to_string(),
		));
	}
	if item.priority > 3 {
		return Err(AlfredError::InvalidArgument(
			"priority must be between 0 and 3".to_string(),
		));
	}
	if item.cards.iter().any(|card| card.trim().is_empty()) {
		return Err(AlfredError::InvalidArgument(
			"cards must not contain empty values".to_string(),
		));
	}
	if item.description.trim().is_empty() {
		return Err(AlfredError::InvalidArgument(
			"description must not be empty".to_string(),
		));
	}
	if item.deliverables.is_empty() {
		return Err(AlfredError::InvalidArgument(
			"deliverables must contain at least one item".to_string(),
		));
	}
	if item
		.deliverables
		.iter()
		.any(|deliverable| deliverable.trim().is_empty())
	{
		return Err(AlfredError::InvalidArgument(
			"deliverables must not contain empty values".to_string(),
		));
	}

	Ok(())
}
