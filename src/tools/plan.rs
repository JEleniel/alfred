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
	pub const NAMES: &'static [&'static str] = &["plan"];
}

#[derive(Debug, Clone, Copy)]
enum PlanOperation {
	Get,
	Add,
	UpdateStatus,
	Delete,
}

impl PlanOperation {
	fn parse(operation: &str) -> Result<Self, AlfredError> {
		match operation {
			"get" | "retrieve" => Ok(Self::Get),
			"add" => Ok(Self::Add),
			"update_status" => Ok(Self::UpdateStatus),
			"delete" | "remove" => Ok(Self::Delete),
			other => Err(AlfredError::InvalidArgument(format!(
				"operation must be one of get, add, update_status, delete: {other}"
			))),
		}
	}

	fn as_str(self) -> &'static str {
		match self {
			Self::Get => "get",
			Self::Add => "add",
			Self::UpdateStatus => "update_status",
			Self::Delete => "delete",
		}
	}
}

#[derive(Debug, Deserialize)]
struct PlanArgs {
	operation: String,
	#[serde(default)]
	args: Value,
}

#[derive(Debug, Deserialize)]
struct PlanUpdateStatusArgs {
	id: u64,
	status: PlanStatus,
}

#[derive(Debug, Deserialize)]
struct PlanDeleteArgs {
	id: u64,
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
		"plan" => handle_plan(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_plan(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<PlanArgs>(args)?;
	let operation = PlanOperation::parse(args.operation.as_str())?;

	let result = match operation {
		PlanOperation::Get => {
			let items = services.plan_store.read_items()?;
			json!({ "items": items })
		}
		PlanOperation::Add => {
			let args = parse_args::<PlanAddArgs>(args.args)?;
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
			json!({ "id": id })
		}
		PlanOperation::UpdateStatus => {
			let args = parse_args::<PlanUpdateStatusArgs>(args.args)?;
			services.plan_store.update_status(args.id, args.status)?;
			json!({})
		}
		PlanOperation::Delete => {
			let args = parse_args::<PlanDeleteArgs>(args.args)?;
			services.plan_store.delete_item(args.id)?;
			json!({})
		}
	};

	Ok(json!({
		"operation": operation.as_str(),
		"result": result,
	}))
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}
