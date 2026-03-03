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
use crate::redaction::Redactor;

use std::sync::{Arc, Mutex, OnceLock};

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

/// Logical storage scopes for persisted memories.
#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryScope {
	User,
	Workspace,
}

/// Memory fact annotated with its storage scope.
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScopedMemoryFact {
	pub scope: MemoryScope,
	#[serde(flatten)]
	pub fact: MemoryFact,
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
	writer: Option<IndexWriter>,
}

impl MemoryDocumentStore {
	fn open_for_read(root: &Path) -> Result<Self, AlfredError> {
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

		Ok(Self {
			fields,
			reader,
			writer: None,
		})
	}

	fn open_for_write(root: &Path) -> Result<Self, AlfredError> {
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
			writer: Some(writer),
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
		let writer = self.writer.as_mut().ok_or_else(|| {
			AlfredError::Internal("memory store writer is not available".to_string())
		})?;
		writer.delete_term(Term::from_field_text(self.fields.id, fact.id.as_str()));

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

		writer.add_document(document).map_err(|error| {
			AlfredError::Internal(format!("failed to add memory document: {error}"))
		})?;
		self.commit()
	}

	fn delete(&mut self, id: &str) -> Result<(), AlfredError> {
		let writer = self.writer.as_mut().ok_or_else(|| {
			AlfredError::Internal("memory store writer is not available".to_string())
		})?;
		writer.delete_term(Term::from_field_text(self.fields.id, id));
		self.commit()
	}

	fn commit(&mut self) -> Result<(), AlfredError> {
		let writer = self.writer.as_mut().ok_or_else(|| {
			AlfredError::Internal("memory store writer is not available".to_string())
		})?;
		writer.commit().map_err(|error| {
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
	redactor: Arc<Redactor>,
	user_store_path: Option<PathBuf>,
	workspace_store_path: Option<PathBuf>,
}

impl MemoryStore {
	/// Creates memory store paths for user and workspace scopes.
	pub fn new(
		redactor: Arc<Redactor>,
		user_store_path: Option<PathBuf>,
		workspace_store_path: Option<PathBuf>,
	) -> Self {
		Self {
			redactor,
			user_store_path,
			workspace_store_path,
		}
	}

	/// Returns the configured user store path.
	pub fn user_store_path(&self) -> Option<&Path> {
		self.user_store_path.as_deref()
	}

	/// Returns the configured workspace store path.
	pub fn workspace_store_path(&self) -> Option<&Path> {
		self.workspace_store_path.as_deref()
	}

	fn store_path_for_scope(&self, scope: MemoryScope) -> Result<&Path, AlfredError> {
		match scope {
			MemoryScope::User => self.user_store_path.as_deref().ok_or_else(|| {
				AlfredError::InvalidArgument(
					"user memory storage is disabled by configuration".to_string(),
				)
			}),
			MemoryScope::Workspace => self.workspace_store_path.as_deref().ok_or_else(|| {
				AlfredError::InvalidArgument(
					"workspace memory storage is disabled by configuration".to_string(),
				)
			}),
		}
	}

	fn facts_for_scope(&self, scope: MemoryScope) -> Result<Vec<MemoryFact>, AlfredError> {
		let path = match scope {
			MemoryScope::User => self.user_store_path.as_deref(),
			MemoryScope::Workspace => self.workspace_store_path.as_deref(),
		};

		match path {
			Some(path) => open_store(path)?.read_all(),
			None => Ok(Vec::new()),
		}
	}

	/// Upserts a memory fact in the explicitly requested storage scope.
	pub fn upsert_in_scope(
		&self,
		scope: MemoryScope,
		input: MemoryFactInput,
	) -> Result<String, AlfredError> {
		validate_input(&input)?;
		let mut input = input;
		input.subject = self.redactor.redact_text(input.subject.as_str());
		input.fact = self.redactor.redact_text(input.fact.as_str());
		input.citations = self.redactor.redact_text(input.citations.as_str());
		input.reason = self.redactor.redact_text(input.reason.as_str());
		input.category = self.redactor.redact_text(input.category.as_str());
		input.tags = input
			.tags
			.into_iter()
			.map(|tag| self.redactor.redact_text(tag.as_str()))
			.collect();

		let tags = normalize_tags(input.tags);
		let existing = self.get_in_scope(scope, input.id.as_str())?;
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

		let target_path = self.store_path_for_scope(scope)?;
		with_store_for_write(target_path, |store| store.upsert(&fact))?;
		Ok(fact.id)
	}

	/// Returns a fact by id from a specific scope.
	pub fn get_in_scope(
		&self,
		scope: MemoryScope,
		id: &str,
	) -> Result<Option<MemoryFact>, AlfredError> {
		validate_id(id)?;
		let facts = self.facts_for_scope(scope)?;
		Ok(facts.into_iter().find(|fact| fact.id == id))
	}

	/// Returns whether a fact id exists in a specific scope.
	pub fn exists_in_scope(&self, scope: MemoryScope, id: &str) -> Result<bool, AlfredError> {
		let _ = self.store_path_for_scope(scope)?;
		Ok(self.get_in_scope(scope, id)?.is_some())
	}

	/// Returns effective merged facts and preserves the winning scope per id.
	pub fn list_effective_scoped(&self) -> Result<Vec<ScopedMemoryFact>, AlfredError> {
		let mut merged = HashMap::<String, ScopedMemoryFact>::new();
		for fact in self.facts_for_scope(MemoryScope::User)? {
			merged.insert(
				fact.id.clone(),
				ScopedMemoryFact {
					scope: MemoryScope::User,
					fact,
				},
			);
		}
		for fact in self.facts_for_scope(MemoryScope::Workspace)? {
			merged.insert(
				fact.id.clone(),
				ScopedMemoryFact {
					scope: MemoryScope::Workspace,
					fact,
				},
			);
		}

		let mut facts = merged.into_values().collect::<Vec<_>>();
		facts.sort_by(|left, right| left.fact.id.cmp(&right.fact.id));
		Ok(facts)
	}

	/// Retrieves an effective memory fact by id and returns the winning scope.
	pub fn get_effective_scoped(&self, id: &str) -> Result<Option<ScopedMemoryFact>, AlfredError> {
		validate_id(id)?;
		if let Some(found) = self.get_in_scope(MemoryScope::Workspace, id)? {
			return Ok(Some(ScopedMemoryFact {
				scope: MemoryScope::Workspace,
				fact: found,
			}));
		}
		if let Some(found) = self.get_in_scope(MemoryScope::User, id)? {
			return Ok(Some(ScopedMemoryFact {
				scope: MemoryScope::User,
				fact: found,
			}));
		}

		Ok(None)
	}

	/// Upserts a memory fact and returns the stable id.
	pub fn put(&self, input: MemoryFactInput) -> Result<String, AlfredError> {
		validate_input(&input)?;
		let mut input = input;
		input.subject = self.redactor.redact_text(input.subject.as_str());
		input.fact = self.redactor.redact_text(input.fact.as_str());
		input.citations = self.redactor.redact_text(input.citations.as_str());
		input.reason = self.redactor.redact_text(input.reason.as_str());
		input.category = self.redactor.redact_text(input.category.as_str());
		input.tags = input
			.tags
			.into_iter()
			.map(|tag| self.redactor.redact_text(tag.as_str()))
			.collect();

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

		let target_path = if let Some(workspace_path) = self.workspace_store_path.as_deref()
			&& self.has_id(workspace_path, fact.id.as_str())?
		{
			workspace_path
		} else if let Some(user_path) = self.user_store_path.as_deref() {
			user_path
		} else if let Some(workspace_path) = self.workspace_store_path.as_deref() {
			workspace_path
		} else {
			return Err(AlfredError::InvalidArgument(
				"memory storage is disabled by configuration".to_string(),
			));
		};

		with_store_for_write(target_path, |store| store.upsert(&fact))?;
		Ok(fact.id)
	}

	/// Retrieves a single memory fact by id.
	pub fn get(&self, id: &str) -> Result<MemoryFact, AlfredError> {
		self.get_effective_scoped(id)?
			.map(|fact| fact.fact)
			.ok_or_else(|| AlfredError::NotFound(format!("memory fact not found: {id}")))
	}

	/// Deletes a memory fact by id.
	pub fn delete(&self, id: &str, dry_run: bool) -> Result<bool, AlfredError> {
		validate_id(id)?;
		let user_exists = match self.user_store_path.as_deref() {
			Some(path) => self.has_id(path, id)?,
			None => false,
		};
		let workspace_exists = match self.workspace_store_path.as_deref() {
			Some(path) => self.has_id(path, id)?,
			None => false,
		};
		let deleted = user_exists || workspace_exists;
		if !deleted || dry_run {
			return Ok(deleted);
		}

		if user_exists && let Some(user_path) = self.user_store_path.as_deref() {
			with_store_for_write(user_path, |store| store.delete(id))?;
		}
		if workspace_exists && let Some(workspace_path) = self.workspace_store_path.as_deref() {
			with_store_for_write(workspace_path, |store| store.delete(id))?;
		}

		Ok(true)
	}

	/// Returns the effective merged set of memory facts.
	pub fn list_effective(&self) -> Result<Vec<MemoryFact>, AlfredError> {
		Ok(self
			.list_effective_scoped()?
			.into_iter()
			.map(|fact| fact.fact)
			.collect())
	}

	fn has_id(&self, store_path: &Path, id: &str) -> Result<bool, AlfredError> {
		let facts = open_store(store_path)?.read_all()?;
		Ok(facts.iter().any(|fact| fact.id == id))
	}

	fn find_effective_by_id(&self, id: &str) -> Result<Option<MemoryFact>, AlfredError> {
		Ok(self.get_effective_scoped(id)?.map(|fact| fact.fact))
	}
}

fn open_store(configured_path: &Path) -> Result<MemoryDocumentStore, AlfredError> {
	MemoryDocumentStore::open_for_read(store_index_root(configured_path).as_path())
}

fn with_store_for_write<T>(
	configured_path: &Path,
	f: impl FnOnce(&mut MemoryDocumentStore) -> Result<T, AlfredError>,
) -> Result<T, AlfredError> {
	let root = store_index_root(configured_path);
	let lock = writer_lock_for_store(root.as_path())?;
	let _guard = lock
		.lock()
		.map_err(|_| AlfredError::Internal("memory store writer lock is poisoned".to_string()))?;

	let mut store = MemoryDocumentStore::open_for_write(root.as_path())?;
	f(&mut store)
}

fn writer_lock_for_store(root: &Path) -> Result<Arc<Mutex<()>>, AlfredError> {
	static LOCKS: OnceLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();
	let locks = LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
	let mut guard = locks
		.lock()
		.map_err(|_| AlfredError::Internal("memory store lock registry is poisoned".to_string()))?;
	Ok(guard
		.entry(root.to_path_buf())
		.or_insert_with(|| Arc::new(Mutex::new(())))
		.clone())
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
