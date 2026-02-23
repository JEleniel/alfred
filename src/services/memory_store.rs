//! Local memory storage service.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::AllQuery;
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term, doc};

use crate::errors::AlfredError;

const INDEX_WRITER_HEAP_BYTES: usize = 20_000_000;

/// Input shape for memory fact upserts.
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MemoryFactInput {
	pub id: String,
	pub subject: String,
	pub fact: String,
	pub citations: String,
	pub reason: String,
	pub category: String,
	#[serde(default)]
	pub tags: Vec<String>,
}

/// Stored memory fact shape.
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MemoryFact {
	pub id: String,
	pub subject: String,
	pub fact: String,
	pub citations: String,
	pub reason: String,
	pub category: String,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<String>,
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Clone, Copy)]
struct MemoryFields {
	id: Field,
	subject: Field,
	fact: Field,
	citations: Field,
	reason: Field,
	category: Field,
	tags: Field,
	created_at: Field,
	updated_at: Field,
}

struct MemoryDocumentStore {
	fields: MemoryFields,
	reader: IndexReader,
	writer: IndexWriter,
}

impl MemoryDocumentStore {
	fn open(root: &Path) -> Result<Self, AlfredError> {
		fs::create_dir_all(root).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to create memory store directory {}: {error}",
				root.display()
			))
		})?;

		let schema = memory_schema();
		let fields = resolve_schema_fields(&schema)?;
		let directory = MmapDirectory::open(root).map_err(|error| {
			AlfredError::IoError(format!(
				"failed to open memory store directory {}: {error}",
				root.display()
			))
		})?;
		let index = Index::open_or_create(directory, schema).map_err(|error| {
			AlfredError::Internal(format!(
				"failed to open or create memory store index: {error}"
			))
		})?;
		let reader = index
			.reader_builder()
			.reload_policy(ReloadPolicy::Manual)
			.try_into()
			.map_err(|error| {
				AlfredError::Internal(format!("failed to initialize memory store reader: {error}"))
			})?;
		let writer = index.writer(INDEX_WRITER_HEAP_BYTES).map_err(|error| {
			AlfredError::Internal(format!("failed to initialize memory store writer: {error}"))
		})?;

		Ok(Self {
			fields,
			reader,
			writer,
		})
	}

	fn read_all(&self) -> Result<Vec<MemoryFact>, AlfredError> {
		self.reader.reload().map_err(|error| {
			AlfredError::Internal(format!("failed to reload memory store reader: {error}"))
		})?;

		let searcher = self.reader.searcher();
		let doc_count = searcher.search(&AllQuery, &Count).map_err(|error| {
			AlfredError::Internal(format!("failed to count memory documents: {error}"))
		})?;
		if doc_count == 0 {
			return Ok(Vec::new());
		}

		let docs = searcher
			.search(&AllQuery, &TopDocs::with_limit(doc_count))
			.map_err(|error| {
				AlfredError::Internal(format!("failed to list memory documents: {error}"))
			})?;

		let mut facts = Vec::new();
		for (_score, address) in docs {
			let document = searcher.doc::<TantivyDocument>(address).map_err(|error| {
				AlfredError::Internal(format!(
					"failed to decode memory document from store: {error}"
				))
			})?;
			if let Some(fact) = parse_fact(&document, self.fields) {
				facts.push(fact);
			}
		}

		facts.sort_by(|left, right| left.id.cmp(&right.id));
		Ok(facts)
	}

	fn upsert(&mut self, fact: &MemoryFact) -> Result<(), AlfredError> {
		self.writer
			.delete_term(Term::from_field_text(self.fields.id, fact.id.as_str()));

		let mut document = TantivyDocument::default();
		document.add_text(self.fields.id, fact.id.as_str());
		document.add_text(self.fields.subject, fact.subject.as_str());
		document.add_text(self.fields.fact, fact.fact.as_str());
		document.add_text(self.fields.citations, fact.citations.as_str());
		document.add_text(self.fields.reason, fact.reason.as_str());
		document.add_text(self.fields.category, fact.category.as_str());
		document.add_text(self.fields.created_at, fact.created_at.as_str());
		document.add_text(self.fields.updated_at, fact.updated_at.as_str());
		for tag in &fact.tags {
			document.add_text(self.fields.tags, tag.as_str());
		}

		self.writer.add_document(document).map_err(|error| {
			AlfredError::Internal(format!("failed to add memory document: {error}"))
		})?;
		self.commit()
	}

	fn delete(&mut self, id: &str) -> Result<(), AlfredError> {
		self.writer
			.delete_term(Term::from_field_text(self.fields.id, id));
		self.commit()
	}

	fn commit(&mut self) -> Result<(), AlfredError> {
		self.writer.commit().map_err(|error| {
			AlfredError::Internal(format!("failed to commit memory store changes: {error}"))
		})?;
		self.reader.reload().map_err(|error| {
			AlfredError::Internal(format!("failed to reload memory store reader: {error}"))
		})
	}
}

/// Represents configured memory store locations.
#[derive(Debug, Clone)]
pub struct MemoryStore {
	user_store_path: PathBuf,
	workspace_store_path: PathBuf,
}

impl MemoryStore {
	/// Creates memory store paths for user and workspace scopes.
	pub fn new(user_store_path: PathBuf, workspace_store_path: PathBuf) -> Self {
		Self {
			user_store_path,
			workspace_store_path,
		}
	}

	/// Returns the configured user store path.
	pub fn user_store_path(&self) -> &Path {
		&self.user_store_path
	}

	/// Returns the configured workspace store path.
	pub fn workspace_store_path(&self) -> &Path {
		&self.workspace_store_path
	}

	/// Upserts a memory fact and returns the stable id.
	pub fn put(&self, input: MemoryFactInput) -> Result<String, AlfredError> {
		validate_input(&input)?;
		let tags = normalize_tags(input.tags);
		let existing = self.find_effective_by_id(input.id.as_str())?;
		let now = now_timestamp();
		let created_at = existing
			.as_ref()
			.map(|fact| fact.created_at.clone())
			.unwrap_or_else(|| now.clone());
		let fact = MemoryFact {
			id: input.id,
			subject: input.subject.trim().to_string(),
			fact: input.fact.trim().to_string(),
			citations: input.citations.trim().to_string(),
			reason: input.reason.trim().to_string(),
			category: input.category.trim().to_string(),
			tags,
			created_at,
			updated_at: now,
		};

		let mut target = if self.has_id(self.workspace_store_path.as_path(), fact.id.as_str())? {
			open_store(self.workspace_store_path.as_path())?
		} else {
			open_store(self.user_store_path.as_path())?
		};

		target.upsert(&fact)?;
		Ok(fact.id)
	}

	/// Retrieves a single memory fact by id.
	pub fn get(&self, id: &str) -> Result<MemoryFact, AlfredError> {
		validate_id(id)?;
		self.find_effective_by_id(id)?
			.ok_or_else(|| AlfredError::NotFound(format!("memory fact not found: {id}")))
	}

	/// Deletes a memory fact by id.
	pub fn delete(&self, id: &str, dry_run: bool) -> Result<bool, AlfredError> {
		validate_id(id)?;
		let user_exists = self.has_id(self.user_store_path.as_path(), id)?;
		let workspace_exists = self.has_id(self.workspace_store_path.as_path(), id)?;
		let deleted = user_exists || workspace_exists;
		if !deleted || dry_run {
			return Ok(deleted);
		}

		if user_exists {
			open_store(self.user_store_path.as_path())?.delete(id)?;
		}
		if workspace_exists {
			open_store(self.workspace_store_path.as_path())?.delete(id)?;
		}

		Ok(true)
	}

	/// Returns the effective merged set of memory facts.
	pub fn list_effective(&self) -> Result<Vec<MemoryFact>, AlfredError> {
		let mut merged = HashMap::<String, MemoryFact>::new();
		for fact in open_store(self.user_store_path.as_path())?.read_all()? {
			merged.insert(fact.id.clone(), fact);
		}
		for fact in open_store(self.workspace_store_path.as_path())?.read_all()? {
			merged.insert(fact.id.clone(), fact);
		}

		let mut facts = merged.into_values().collect::<Vec<_>>();
		facts.sort_by(|left, right| left.id.cmp(&right.id));
		Ok(facts)
	}

	fn has_id(&self, store_path: &Path, id: &str) -> Result<bool, AlfredError> {
		let facts = open_store(store_path)?.read_all()?;
		Ok(facts.iter().any(|fact| fact.id == id))
	}

	fn find_effective_by_id(&self, id: &str) -> Result<Option<MemoryFact>, AlfredError> {
		if let Some(found) = open_store(self.workspace_store_path.as_path())?
			.read_all()?
			.into_iter()
			.find(|fact| fact.id == id)
		{
			return Ok(Some(found));
		}

		Ok(open_store(self.user_store_path.as_path())?
			.read_all()?
			.into_iter()
			.find(|fact| fact.id == id))
	}
}

fn open_store(configured_path: &Path) -> Result<MemoryDocumentStore, AlfredError> {
	MemoryDocumentStore::open(store_index_root(configured_path).as_path())
}

fn store_index_root(configured_path: &Path) -> PathBuf {
	configured_path.with_extension("tantivy")
}

fn memory_schema() -> Schema {
	let mut builder = Schema::builder();
	builder.add_text_field("id", STRING | STORED);
	builder.add_text_field("subject", TEXT | STORED);
	builder.add_text_field("fact", TEXT | STORED);
	builder.add_text_field("citations", TEXT | STORED);
	builder.add_text_field("reason", TEXT | STORED);
	builder.add_text_field("category", STRING | STORED);
	builder.add_text_field("tags", STRING | TEXT | STORED);
	builder.add_text_field("created_at", STRING | STORED);
	builder.add_text_field("updated_at", STRING | STORED);
	builder.build()
}

fn resolve_schema_fields(schema: &Schema) -> Result<MemoryFields, AlfredError> {
	Ok(MemoryFields {
		id: schema
			.get_field("id")
			.map_err(|error| AlfredError::Internal(format!("memory schema missing id: {error}")))?,
		subject: schema.get_field("subject").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing subject: {error}"))
		})?,
		fact: schema.get_field("fact").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing fact: {error}"))
		})?,
		citations: schema.get_field("citations").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing citations: {error}"))
		})?,
		reason: schema.get_field("reason").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing reason: {error}"))
		})?,
		category: schema.get_field("category").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing category: {error}"))
		})?,
		tags: schema.get_field("tags").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing tags: {error}"))
		})?,
		created_at: schema.get_field("created_at").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing created_at: {error}"))
		})?,
		updated_at: schema.get_field("updated_at").map_err(|error| {
			AlfredError::Internal(format!("memory schema missing updated_at: {error}"))
		})?,
	})
}

fn parse_fact(document: &TantivyDocument, fields: MemoryFields) -> Option<MemoryFact> {
	let id = document.get_first(fields.id)?.as_str()?.to_string();
	let subject = document.get_first(fields.subject)?.as_str()?.to_string();
	let fact = document.get_first(fields.fact)?.as_str()?.to_string();
	let citations = document.get_first(fields.citations)?.as_str()?.to_string();
	let reason = document.get_first(fields.reason)?.as_str()?.to_string();
	let category = document.get_first(fields.category)?.as_str()?.to_string();
	let created_at = document.get_first(fields.created_at)?.as_str()?.to_string();
	let updated_at = document.get_first(fields.updated_at)?.as_str()?.to_string();
	let mut tags = document
		.get_all(fields.tags)
		.filter_map(|value| value.as_str().map(|text| text.to_string()))
		.collect::<Vec<_>>();
	tags.sort_unstable();
	tags.dedup();

	Some(MemoryFact {
		id,
		subject,
		fact,
		citations,
		reason,
		category,
		tags,
		created_at,
		updated_at,
	})
}

fn validate_input(input: &MemoryFactInput) -> Result<(), AlfredError> {
	validate_non_empty(input.id.as_str(), "id")?;
	validate_non_empty(input.subject.as_str(), "subject")?;
	validate_non_empty(input.fact.as_str(), "fact")?;
	validate_non_empty(input.citations.as_str(), "citations")?;
	validate_non_empty(input.reason.as_str(), "reason")?;
	validate_non_empty(input.category.as_str(), "category")
}

fn validate_id(id: &str) -> Result<(), AlfredError> {
	validate_non_empty(id, "id")
}

fn validate_non_empty(value: &str, field: &str) -> Result<(), AlfredError> {
	if value.trim().is_empty() {
		return Err(AlfredError::InvalidArgument(format!(
			"{field} must not be empty"
		)));
	}

	Ok(())
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
	let mut normalized = tags
		.into_iter()
		.map(|tag| tag.trim().to_string())
		.filter(|tag| !tag.is_empty())
		.collect::<Vec<_>>();
	normalized.sort_unstable();
	normalized.dedup();
	normalized
}

fn now_timestamp() -> String {
	Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}
