//! Plan tool group.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::services::plan_store::{PlanItem, PlanStatus};

/// Project plan CRUD tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlanTools;

impl PlanTools {
	pub const NAMES: &'static [&'static str] = &[
		"plan_get",
		"plan_update",
		"plan_edit",
		"plan_add",
		"plan_delete",
	];
}

#[derive(Debug, Deserialize)]
struct PlanUpdateArgs {
	id: u64,
	status: PlanStatus,
}

#[derive(Debug, Deserialize)]
struct PlanDeleteArgs {
	id: u64,
}

#[derive(Debug, Deserialize)]
struct PlanEditArgs {
	id: u64,
	title: String,
	priority: u8,
	cards: Vec<String>,
	description: String,
	deliverables: Vec<String>,
	acceptance_criteria: Option<String>,
	notes: Option<String>,
	status: PlanStatus,
}

#[derive(Debug, Deserialize)]
struct PlanAddArgs {
	#[serde(default)]
	id: Option<u64>,
	title: String,
	priority: u8,
	cards: Vec<String>,
	description: String,
	deliverables: Vec<String>,
	acceptance_criteria: Option<String>,
	notes: Option<String>,
	status: PlanStatus,
}

/// Handles plan tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"plan_get" => handle_plan_get(services).map(Some),
		"plan_update" => handle_plan_update(args, services).map(Some),
		"plan_edit" => handle_plan_edit(args, services).map(Some),
		"plan_add" => handle_plan_add(args, services).map(Some),
		"plan_delete" => handle_plan_delete(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_plan_get(services: &ServiceContainer) -> Result<Value, AlfredError> {
	let items = services.plan_store.read_items()?;
	Ok(json!({ "items": items }))
}

fn handle_plan_update(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<PlanUpdateArgs>(args)?;
	services.plan_store.update_status(args.id, args.status)?;
	Ok(json!({}))
}

fn handle_plan_edit(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<PlanEditArgs>(args)?;
	services.plan_store.edit_item(PlanItem {
		id: args.id,
		title: args.title,
		priority: args.priority,
		cards: args.cards,
		description: args.description,
		deliverables: args.deliverables,
		acceptance_criteria: args.acceptance_criteria,
		notes: args.notes,
		status: args.status,
	})?;
	Ok(json!({}))
}

fn handle_plan_add(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<PlanAddArgs>(args)?;
	let _ = args.id;
	let id = services.plan_store.add_item(PlanItem {
		id: 0,
		title: args.title,
		priority: args.priority,
		cards: args.cards,
		description: args.description,
		deliverables: args.deliverables,
		acceptance_criteria: args.acceptance_criteria,
		notes: args.notes,
		status: args.status,
	})?;

	Ok(json!({ "id": id }))
}

fn handle_plan_delete(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<PlanDeleteArgs>(args)?;
	services.plan_store.delete_item(args.id)?;
	Ok(json!({}))
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}
