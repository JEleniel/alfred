//! Local memory tool group.

use std::cmp::Ordering;
use std::collections::HashSet;

use base64::Engine;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::services::memory_store::{MemoryFact, MemoryFactInput};

const DEFAULT_LIMIT: usize = 100;
const CURSOR_PREFIX: &str = "v1:";

/// Local memory CRUD/search tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryTools;

impl MemoryTools {
	pub const NAMES: &'static [&'static str] = &[
		"memory_put",
		"memory_get",
		"memory_delete",
		"memory_list",
		"memory_search",
	];
}

#[derive(Debug, Deserialize)]
struct MemoryIdArgs {
	id: String,
}

#[derive(Debug, Deserialize)]
struct MemoryDeleteArgs {
	id: String,
	dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MemoryListArgs {
	cursor: Option<String>,
	limit: Option<usize>,
	order: Option<String>,
	subject: Option<String>,
	category: Option<String>,
	tags_any: Option<Vec<String>>,
	tags_and: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MemorySearchArgs {
	query: String,
	limit: Option<usize>,
	cursor: Option<String>,
	subject: Option<String>,
	category: Option<String>,
	tags_any: Option<Vec<String>>,
	tags_and: Option<bool>,
}

/// Handles memory tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"memory_put" => handle_memory_put(args, services).map(Some),
		"memory_get" => handle_memory_get(args, services).map(Some),
		"memory_delete" => handle_memory_delete(args, services).map(Some),
		"memory_list" => handle_memory_list(args, services).map(Some),
		"memory_search" => handle_memory_search(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_memory_put(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let input = parse_args::<MemoryFactInput>(args)?;
	let id = services.memory_store.put(input)?;
	Ok(json!({"id": id}))
}

fn handle_memory_get(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryIdArgs>(args)?;
	let fact = services.memory_store.get(args.id.as_str())?;
	Ok(json!({"fact": fact}))
}

fn handle_memory_delete(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryDeleteArgs>(args)?;
	let deleted = services
		.memory_store
		.delete(args.id.as_str(), args.dry_run.unwrap_or(false))?;
	Ok(json!({"deleted": deleted}))
}

fn handle_memory_list(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryListArgs>(args)?;
	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;
	let tags = normalize_tags_filter(args.tags_any.unwrap_or_default());
	let mut facts = services.memory_store.list_effective()?;

	facts = filter_facts(
		facts,
		args.subject.as_deref(),
		args.category.as_deref(),
		tags.as_slice(),
		args.tags_and.unwrap_or(false),
	);
	sort_facts(facts.as_mut_slice(), args.order.as_deref())?;

	let (facts, next_cursor) = paginate_items(facts, offset, limit);
	Ok(json!({
		"facts": facts,
		"next_cursor": next_cursor,
	}))
}

fn handle_memory_search(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemorySearchArgs>(args)?;
	if args.query.trim().is_empty() {
		return Err(AlfredError::InvalidArgument(
			"query must not be empty".to_string(),
		));
	}

	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;
	let tags = normalize_tags_filter(args.tags_any.unwrap_or_default());
	let facts = filter_facts(
		services.memory_store.list_effective()?,
		args.subject.as_deref(),
		args.category.as_deref(),
		tags.as_slice(),
		args.tags_and.unwrap_or(false),
	);

	let tokens = normalize_query_tokens(args.query.as_str());
	let mut matches = score_matches(facts, tokens.as_slice());
	matches.sort_by(compare_search_matches);

	let (matches, next_cursor) = paginate_items(matches, offset, limit);
	Ok(json!({
		"matches": matches,
		"next_cursor": next_cursor,
	}))
}

fn filter_facts(
	facts: Vec<MemoryFact>,
	subject: Option<&str>,
	category: Option<&str>,
	tags_any: &[String],
	tags_and: bool,
) -> Vec<MemoryFact> {
	facts
		.into_iter()
		.filter(|fact| subject_filter_matches(fact, subject))
		.filter(|fact| category_filter_matches(fact, category))
		.filter(|fact| tags_filter_matches(fact, tags_any, tags_and))
		.collect()
}

fn subject_filter_matches(fact: &MemoryFact, subject: Option<&str>) -> bool {
	match subject {
		Some(expected) => fact.subject == expected,
		None => true,
	}
}

fn category_filter_matches(fact: &MemoryFact, category: Option<&str>) -> bool {
	match category {
		Some(expected) => fact.category == expected,
		None => true,
	}
}

fn tags_filter_matches(fact: &MemoryFact, tags_any: &[String], tags_and: bool) -> bool {
	if tags_any.is_empty() {
		return true;
	}

	let available = fact
		.tags
		.iter()
		.map(|tag| tag.as_str())
		.collect::<HashSet<_>>();
	if tags_and {
		return tags_any.iter().all(|tag| available.contains(tag.as_str()));
	}

	tags_any.iter().any(|tag| available.contains(tag.as_str()))
}

fn sort_facts(facts: &mut [MemoryFact], order: Option<&str>) -> Result<(), AlfredError> {
	let order = order.unwrap_or("updated_at");
	match order {
		"created_at" => facts.sort_by(compare_created_at_desc),
		"updated_at" => facts.sort_by(compare_updated_at_desc),
		"subject" => facts.sort_by(compare_subject_asc),
		other => {
			return Err(AlfredError::InvalidArgument(format!(
				"order must be one of created_at, updated_at, subject: {other}"
			)));
		}
	}

	Ok(())
}

fn compare_created_at_desc(left: &MemoryFact, right: &MemoryFact) -> Ordering {
	right
		.created_at
		.cmp(&left.created_at)
		.then_with(|| left.id.cmp(&right.id))
}

fn compare_updated_at_desc(left: &MemoryFact, right: &MemoryFact) -> Ordering {
	right
		.updated_at
		.cmp(&left.updated_at)
		.then_with(|| left.id.cmp(&right.id))
}

fn compare_subject_asc(left: &MemoryFact, right: &MemoryFact) -> Ordering {
	left.subject
		.to_lowercase()
		.cmp(&right.subject.to_lowercase())
		.then_with(|| left.subject.cmp(&right.subject))
		.then_with(|| left.id.cmp(&right.id))
}

fn score_matches(facts: Vec<MemoryFact>, tokens: &[String]) -> Vec<Value> {
	facts
		.into_iter()
		.filter_map(|fact| {
			let score = memory_score(&fact, tokens);
			(score > 0).then(|| json!({"fact": fact, "score": score}))
		})
		.collect()
}

fn compare_search_matches(left: &Value, right: &Value) -> Ordering {
	let left_score = left.get("score").and_then(Value::as_u64).unwrap_or(0);
	let right_score = right.get("score").and_then(Value::as_u64).unwrap_or(0);
	let left_updated = left["fact"]["updated_at"].as_str().unwrap_or_default();
	let right_updated = right["fact"]["updated_at"].as_str().unwrap_or_default();
	let left_id = left["fact"]["id"].as_str().unwrap_or_default();
	let right_id = right["fact"]["id"].as_str().unwrap_or_default();

	right_score
		.cmp(&left_score)
		.then_with(|| right_updated.cmp(left_updated))
		.then_with(|| left_id.cmp(right_id))
}

fn memory_score(fact: &MemoryFact, tokens: &[String]) -> u64 {
	let subject = fact.subject.to_lowercase();
	let body = fact.fact.to_lowercase();
	let citations = fact.citations.to_lowercase();
	let reason = fact.reason.to_lowercase();
	let category = fact.category.to_lowercase();
	let tags = fact
		.tags
		.iter()
		.map(|tag| tag.to_lowercase())
		.collect::<Vec<_>>();

	tokens
		.iter()
		.map(|token| {
			token_score(
				token, &subject, &body, &citations, &reason, &category, &tags,
			)
		})
		.sum()
}

fn token_score(
	token: &str,
	subject: &str,
	body: &str,
	citations: &str,
	reason: &str,
	category: &str,
	tags: &[String],
) -> u64 {
	let subject_score = weighted_contains(subject, token, 4);
	let fact_score = weighted_contains(body, token, 3);
	let citations_score = weighted_contains(citations, token, 2);
	let reason_score = weighted_contains(reason, token, 2);
	let category_score = weighted_contains(category, token, 1);
	let tags_score = tags
		.iter()
		.map(|tag| weighted_contains(tag, token, 2))
		.sum::<u64>();

	subject_score + fact_score + citations_score + reason_score + category_score + tags_score
}

fn weighted_contains(haystack: &str, needle: &str, weight: u64) -> u64 {
	if needle.is_empty() {
		return 0;
	}

	(haystack.matches(needle).count() as u64) * weight
}

fn normalize_query_tokens(query: &str) -> Vec<String> {
	query
		.split_whitespace()
		.map(|token| token.to_lowercase())
		.filter(|token| !token.is_empty())
		.collect()
}

fn normalize_tags_filter(tags: Vec<String>) -> Vec<String> {
	let mut normalized = tags
		.into_iter()
		.map(|tag| tag.trim().to_string())
		.filter(|tag| !tag.is_empty())
		.collect::<Vec<_>>();
	normalized.sort_unstable();
	normalized.dedup();
	normalized
}

fn paginate_items<T>(items: Vec<T>, offset: usize, limit: usize) -> (Vec<T>, Option<String>) {
	if offset >= items.len() {
		return (Vec::new(), None);
	}

	let end = std::cmp::min(items.len(), offset + limit);
	let next_cursor = if end < items.len() {
		Some(encode_cursor(end))
	} else {
		None
	};

	(
		items.into_iter().skip(offset).take(limit).collect(),
		next_cursor,
	)
}

fn parse_pagination(
	cursor: Option<&str>,
	limit: Option<usize>,
) -> Result<(usize, usize), AlfredError> {
	let offset = match cursor {
		Some(token) => decode_cursor(token)?,
		None => 0,
	};

	let limit = limit.unwrap_or(DEFAULT_LIMIT);
	if limit == 0 {
		return Err(AlfredError::InvalidArgument(
			"limit must be greater than 0".to_string(),
		));
	}

	Ok((offset, limit))
}

fn encode_cursor(offset: usize) -> String {
	let payload = format!("{CURSOR_PREFIX}{offset}");
	base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload)
}

fn decode_cursor(cursor: &str) -> Result<usize, AlfredError> {
	let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
		.decode(cursor)
		.map_err(|_| AlfredError::InvalidArgument(format!("invalid cursor token: {cursor}")))?;
	let payload = String::from_utf8(bytes)
		.map_err(|_| AlfredError::InvalidArgument(format!("invalid cursor token: {cursor}")))?;
	let value = payload
		.strip_prefix(CURSOR_PREFIX)
		.ok_or_else(|| AlfredError::InvalidArgument(format!("invalid cursor token: {cursor}")))?;

	value
		.parse::<usize>()
		.map_err(|_| AlfredError::InvalidArgument(format!("invalid cursor token: {cursor}")))
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, AlfredError> {
	serde_json::from_value(value)
		.map_err(|error| AlfredError::InvalidArgument(format!("invalid tool arguments: {error}")))
}
