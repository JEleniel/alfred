//! Local memory tool group.

use std::cmp::Ordering;
use std::collections::HashSet;

use base64::Engine;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::errors::AlfredError;
use crate::services::ServiceContainer;
use crate::services::memory_store::{
	MemoryFact, MemoryFactInput, MemoryScope, ScopedMemoryFact,
};

const DEFAULT_LIMIT: usize = 100;
const CURSOR_PREFIX: &str = "v1:";

/// Local memory CRUD/search tools.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryTools;

impl MemoryTools {
	pub const NAMES: &'static [&'static str] = &["memory"];
}

#[derive(Debug, Clone, Copy)]
enum MemoryOperation {
	Create,
	Retrieve,
	Update,
	Delete,
	Search,
}

impl MemoryOperation {
	fn parse(operation: &str) -> Result<Self, AlfredError> {
		match operation {
			"create" => Ok(Self::Create),
			"retrieve" | "get" => Ok(Self::Retrieve),
			"update" => Ok(Self::Update),
			"delete" => Ok(Self::Delete),
			"search" | "list" => Ok(Self::Search),
			other => Err(AlfredError::InvalidArgument(format!(
				"operation must be one of create, retrieve, update, delete, search: {other}"
			))),
		}
	}

	fn as_str(self) -> &'static str {
		match self {
			Self::Create => "create",
			Self::Retrieve => "retrieve",
			Self::Update => "update",
			Self::Delete => "delete",
			Self::Search => "search",
		}
	}
}

#[derive(Debug, Deserialize)]
struct MemoryArgs {
	operation: String,
	#[serde(default)]
	args: Value,
}

#[derive(Debug, Deserialize)]
struct MemoryRetrieveArgs {
	id: String,
}

#[derive(Debug, Deserialize)]
struct MemoryCreateArgs {
	#[serde(default)]
	id: Option<String>,
	scope: MemoryScope,
	subject: String,
	category: String,
	fact: String,
	#[serde(default)]
	reasoning: Option<String>,
	#[serde(default)]
	reason: Option<String>,
	#[serde(default)]
	citations: Option<String>,
	#[serde(default)]
	tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MemoryUpdateArgs {
	id: String,
	scope: MemoryScope,
	subject: String,
	category: String,
	fact: String,
	#[serde(default)]
	reasoning: Option<String>,
	#[serde(default)]
	reason: Option<String>,
	#[serde(default)]
	citations: Option<String>,
	#[serde(default)]
	tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MemoryDeleteArgs {
	id: String,
	dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MemorySearchArgs {
	query: Option<String>,
	limit: Option<usize>,
	cursor: Option<String>,
	subject: Option<String>,
	category: Option<String>,
	tags: Option<Vec<String>>,
	tags_any: Option<Vec<String>>,
	tags_and: Option<bool>,
}

#[derive(Debug, Clone)]
struct SearchMatch {
	fact: ScopedMemoryFact,
	score: u64,
}

/// Handles memory tool calls.
pub fn dispatch_tool_call(
	name: &str,
	args: Value,
	services: &ServiceContainer,
) -> Result<Option<Value>, AlfredError> {
	match name {
		"memory" => handle_memory(args, services).map(Some),
		_ => Ok(None),
	}
}

fn handle_memory(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryArgs>(args)?;
	let operation = MemoryOperation::parse(args.operation.as_str())?;

	let result = match operation {
		MemoryOperation::Create => handle_create(args.args, services)?,
		MemoryOperation::Retrieve => handle_retrieve(args.args, services)?,
		MemoryOperation::Update => handle_update(args.args, services)?,
		MemoryOperation::Delete => handle_delete(args.args, services)?,
		MemoryOperation::Search => handle_search(args.args, services)?,
	};

	Ok(json!({
		"operation": operation.as_str(),
		"result": result,
	}))
}

fn handle_create(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryCreateArgs>(args)?;
	if args.id.is_some() {
		return Err(AlfredError::InvalidArgument(
			"id must be omitted for create operation".to_string(),
		));
	}

	let id = Uuid::new_v4().to_string();
	let input = to_store_input(
		id.clone(),
		args.subject,
		args.category,
		args.fact,
		args.reasoning,
		args.reason,
		args.citations,
		args.tags,
	);
	services.memory_store.upsert_in_scope(args.scope, input)?;

	let fact = services
		.memory_store
		.get_in_scope(args.scope, id.as_str())?
		.ok_or_else(|| {
			AlfredError::Internal("created memory fact could not be retrieved".to_string())
		})?;

	Ok(json!({ "memory": memory_output(args.scope, &fact) }))
}

fn handle_retrieve(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryRetrieveArgs>(args)?;
	let memory = services
		.memory_store
		.get_effective_scoped(args.id.as_str())?
		.ok_or_else(|| AlfredError::NotFound(format!("memory fact not found: {}", args.id)))?;

	Ok(json!({ "memory": memory_output(memory.scope, &memory.fact) }))
}

fn handle_update(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryUpdateArgs>(args)?;
	if !services
		.memory_store
		.exists_in_scope(args.scope, args.id.as_str())?
	{
		return Err(AlfredError::NotFound(format!(
			"memory fact not found in scope {}: {}",
			scope_name(args.scope),
			args.id
		)));
	}

	let input = to_store_input(
		args.id.clone(),
		args.subject,
		args.category,
		args.fact,
		args.reasoning,
		args.reason,
		args.citations,
		args.tags,
	);
	services.memory_store.upsert_in_scope(args.scope, input)?;

	let fact = services
		.memory_store
		.get_in_scope(args.scope, args.id.as_str())?
		.ok_or_else(|| {
			AlfredError::Internal("updated memory fact could not be retrieved".to_string())
		})?;

	Ok(json!({ "memory": memory_output(args.scope, &fact) }))
}

fn handle_delete(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemoryDeleteArgs>(args)?;
	let deleted = services
		.memory_store
		.delete(args.id.as_str(), args.dry_run.unwrap_or(true))?;

	Ok(json!({ "deleted": deleted }))
}

fn handle_search(args: Value, services: &ServiceContainer) -> Result<Value, AlfredError> {
	let args = parse_args::<MemorySearchArgs>(args)?;
	let query_tokens = parse_search_query(args.query)?;
	let has_query = query_tokens.is_some();
	let (offset, limit) = parse_pagination(args.cursor.as_deref(), args.limit)?;
	let tags = normalize_tags_filter(resolve_tags_filter(args.tags, args.tags_any));
	let facts = filter_facts(
		services.memory_store.list_effective_scoped()?,
		args.subject.as_deref(),
		args.category.as_deref(),
		tags.as_slice(),
		args.tags_and.unwrap_or(false),
	);

	let mut matches = if let Some(tokens) = query_tokens {
		score_matches(facts, tokens.as_slice())
	} else {
		facts
			.into_iter()
			.map(|fact| SearchMatch { fact, score: 0 })
			.collect::<Vec<_>>()
	};

	if has_query {
		matches.sort_by(compare_search_matches);
	} else {
		matches.sort_by(compare_match_facts);
	}

	let (matches, next_cursor) = paginate_items(matches, offset, limit);
	let matches = matches
		.into_iter()
		.map(|matched| {
			json!({
				"fact": memory_output(matched.fact.scope, &matched.fact.fact),
				"score": matched.score,
			})
		})
		.collect::<Vec<_>>();

	Ok(json!({
		"matches": matches,
		"next_cursor": next_cursor,
	}))
}

fn filter_facts(
	facts: Vec<ScopedMemoryFact>,
	subject: Option<&str>,
	category: Option<&str>,
	tags_any: &[String],
	tags_and: bool,
) -> Vec<ScopedMemoryFact> {
	facts
		.into_iter()
		.filter(|fact| subject_filter_matches(fact, subject))
		.filter(|fact| category_filter_matches(fact, category))
		.filter(|fact| tags_filter_matches(fact, tags_any, tags_and))
		.collect()
}

fn subject_filter_matches(fact: &ScopedMemoryFact, subject: Option<&str>) -> bool {
	match subject {
		Some(expected) => fact.fact.subject == expected,
		None => true,
	}
}

fn category_filter_matches(fact: &ScopedMemoryFact, category: Option<&str>) -> bool {
	match category {
		Some(expected) => fact.fact.category == expected,
		None => true,
	}
}

fn tags_filter_matches(fact: &ScopedMemoryFact, tags_any: &[String], tags_and: bool) -> bool {
	if tags_any.is_empty() {
		return true;
	}

	let available = fact
		.fact
		.tags
		.iter()
		.map(|tag| tag.as_str())
		.collect::<HashSet<_>>();
	if tags_and {
		return tags_any.iter().all(|tag| available.contains(tag.as_str()));
	}

	tags_any.iter().any(|tag| available.contains(tag.as_str()))
}

fn score_matches(facts: Vec<ScopedMemoryFact>, tokens: &[String]) -> Vec<SearchMatch> {
	facts
		.into_iter()
		.filter_map(|fact| {
			let score = memory_score(&fact, tokens);
			(score > 0).then_some(SearchMatch { fact, score })
		})
		.collect()
}

fn compare_search_matches(left: &SearchMatch, right: &SearchMatch) -> Ordering {
	right
		.score
		.cmp(&left.score)
		.then_with(|| right.fact.fact.updated_at.cmp(&left.fact.fact.updated_at))
		.then_with(|| left.fact.fact.id.cmp(&right.fact.fact.id))
}

fn compare_match_facts(left: &SearchMatch, right: &SearchMatch) -> Ordering {
	right
		.fact
		.fact
		.updated_at
		.cmp(&left.fact.fact.updated_at)
		.then_with(|| left.fact.fact.id.cmp(&right.fact.fact.id))
}

fn memory_score(fact: &ScopedMemoryFact, tokens: &[String]) -> u64 {
	let subject = fact.fact.subject.to_lowercase();
	let body = fact.fact.fact.to_lowercase();
	let reason = fact.fact.reason.to_lowercase();
	let category = fact.fact.category.to_lowercase();
	let scope = scope_name(fact.scope).to_lowercase();
	let tags = fact
		.fact
		.tags
		.iter()
		.map(|tag| tag.to_lowercase())
		.collect::<Vec<_>>();

	tokens
		.iter()
		.map(|token| token_score(token, &subject, &body, &reason, &category, &scope, &tags))
		.sum()
}

fn token_score(
	token: &str,
	subject: &str,
	body: &str,
	reason: &str,
	category: &str,
	scope: &str,
	tags: &[String],
) -> u64 {
	let subject_score = weighted_contains(subject, token, 4);
	let fact_score = weighted_contains(body, token, 3);
	let reason_score = weighted_contains(reason, token, 2);
	let category_score = weighted_contains(category, token, 1);
	let scope_score = weighted_contains(scope, token, 1);
	let tags_score = tags
		.iter()
		.map(|tag| weighted_contains(tag, token, 2))
		.sum::<u64>();

	subject_score + fact_score + reason_score + category_score + scope_score + tags_score
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

fn resolve_tags_filter(tags: Option<Vec<String>>, tags_any: Option<Vec<String>>) -> Vec<String> {
	match (tags, tags_any) {
		(Some(tags), Some(mut tags_any)) => {
			tags_any.extend(tags);
			tags_any
		}
		(Some(tags), None) => tags,
		(None, Some(tags_any)) => tags_any,
		(None, None) => Vec::new(),
	}
}

fn memory_output(scope: MemoryScope, fact: &MemoryFact) -> Value {
	json!({
		"id": fact.id,
		"scope": scope_name(scope),
		"subject": fact.subject,
		"category": fact.category,
		"fact": fact.fact,
		"reasoning": fact.reason,
		"tags": fact.tags,
		"created_at": fact.created_at,
		"updated_at": fact.updated_at,
	})
}

fn scope_name(scope: MemoryScope) -> &'static str {
	match scope {
		MemoryScope::User => "user",
		MemoryScope::Workspace => "workspace",
	}
}

fn to_store_input(
	id: String,
	subject: String,
	category: String,
	fact: String,
	reasoning: Option<String>,
	reason: Option<String>,
	citations: Option<String>,
	tags: Vec<String>,
) -> MemoryFactInput {
	let reasoning = reasoning
		.or(reason)
		.unwrap_or_else(|| "unspecified".to_string());
	let citations = citations.unwrap_or_else(|| "unspecified".to_string());

	MemoryFactInput {
		id,
		subject,
		fact,
		citations,
		reason: reasoning,
		category,
		tags,
	}
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

fn parse_search_query(query: Option<String>) -> Result<Option<Vec<String>>, AlfredError> {
	match query {
		Some(query) => {
			if query.trim().is_empty() {
				return Err(AlfredError::InvalidArgument(
					"query must not be empty when provided".to_string(),
				));
			}

			Ok(Some(normalize_query_tokens(query.as_str())))
		}
		None => Ok(None),
	}
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
