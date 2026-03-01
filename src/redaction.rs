//! Deterministic redaction helpers for log-safe output.
//!
//! See `docs/design/Redaction.md` for the contract.

use anyhow::{Context, Result};
use regex::Regex;
use regex::RegexBuilder;
use serde_json::Value;

/// Default replacement token for redacted content.
pub const DEFAULT_REPLACEMENT_TOKEN: &str = "<-REDACTED->";

/// Summary statistics about a redaction pass.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct RedactionStats {
	pub redacted_spans: usize,
}

/// Result of redacting a single string.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RedactionResult {
	pub output: String,
	pub stats: RedactionStats,
}

/// A configurable redaction rule.
///
/// These are compiled and evaluated in-order to produce a stable set of non-overlapping spans.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RedactionRule {
	/// Redact the value portion of key/value-like text.
	///
	/// `keys` entries are treated as regex fragments and combined into a single case-insensitive
	/// alternation. This matches both `key=value` and JSON-like `"key": "value"`.
	StructuredKeys { keys: Vec<String> },
	/// Redact any free-text matches of the configured regex.
	Regex {
		pattern: String,
		case_sensitive: bool,
	},
	/// Built-in email address matcher.
	BuiltinEmail,
	/// Built-in bearer-token matcher (redacts only the token portion).
	BuiltinBearerToken,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct RedactionSpan {
	start: usize,
	end: usize,
}

impl RedactionSpan {
	fn is_valid(&self) -> bool {
		self.start < self.end
	}
}

/// Redacts non-public information from log messages and tool outputs.
#[derive(Debug, Clone)]
pub struct Redactor {
	enabled: bool,
	replacement_token: String,
	preserve_length: bool,
	rules: Vec<CompiledRule>,
}

#[derive(Debug, Clone)]
enum CompiledRule {
	StructuredKeys(Regex),
	Regex(Regex),
	Email(Regex),
	BearerToken(Regex),
}

impl Redactor {
	/// Builds the default redaction policy.
	///
	/// Defaults match `docs/design/Configuration.md`:
	/// - enabled: true
	/// - replacement_token: `<-REDACTED->`
	/// - preserve_length: true
	pub fn try_default() -> Result<Self> {
		Self::try_from_rules(
			true,
			DEFAULT_REPLACEMENT_TOKEN.to_string(),
			true,
			default_redaction_rules(),
		)
	}

	/// Builds a redactor with explicit configuration.
	pub fn try_new(
		enabled: bool,
		replacement_token: String,
		preserve_length: bool,
	) -> Result<Self> {
		Self::try_from_rules(
			enabled,
			replacement_token,
			preserve_length,
			default_redaction_rules(),
		)
	}

	/// Builds a redactor with explicit configuration and rule list.
	pub fn try_from_rules(
		enabled: bool,
		replacement_token: String,
		preserve_length: bool,
		rules: Vec<RedactionRule>,
	) -> Result<Self> {
		let builtin_email = Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b")
			.context("failed to compile email redaction rule")?;
		let builtin_bearer = Regex::new(r"(?i)\bBearer\s+(?P<token>[A-Za-z0-9._\-~+/]+=*)")
			.context("failed to compile bearer-token redaction rule")?;

		let mut compiled = Vec::new();
		for rule in rules {
			match rule {
				RedactionRule::StructuredKeys { keys } => {
					compiled.push(CompiledRule::StructuredKeys(compile_structured_key_rule(
						keys.as_slice(),
					)?));
				}
				RedactionRule::Regex {
					pattern,
					case_sensitive,
				} => {
					let regex = RegexBuilder::new(pattern.as_str())
						.case_insensitive(!case_sensitive)
						.build()
						.with_context(|| format!("failed to compile redaction regex: {pattern}"))?;
					compiled.push(CompiledRule::Regex(regex));
				}
				RedactionRule::BuiltinEmail => {
					compiled.push(CompiledRule::Email(builtin_email.clone()));
				}
				RedactionRule::BuiltinBearerToken => {
					compiled.push(CompiledRule::BearerToken(builtin_bearer.clone()));
				}
			}
		}

		Ok(Self {
			enabled,
			replacement_token,
			preserve_length,
			rules: compiled,
		})
	}

	/// Redacts deterministic NPI patterns from the provided text.
	///
	/// This preserves the pre-existing API surface for callers that don't need stats.
	pub fn redact_text(&self, input: &str) -> String {
		self.redact_text_with_stats(input).output
	}

	/// Redacts deterministic NPI patterns from the provided text and returns stats.
	pub fn redact_text_with_stats(&self, input: &str) -> RedactionResult {
		if !self.enabled || input.is_empty() {
			return RedactionResult {
				output: input.to_string(),
				stats: RedactionStats::default(),
			};
		}

		let mut spans = Vec::new();
		for rule in &self.rules {
			spans.extend(collect_spans_for_rule(rule, input));
		}

		let spans = merge_spans(spans);
		if spans.is_empty() {
			return RedactionResult {
				output: input.to_string(),
				stats: RedactionStats::default(),
			};
		}

		let output = apply_replacements(
			input,
			&spans,
			self.replacement_token.as_str(),
			self.preserve_length,
		);
		RedactionResult {
			output,
			stats: RedactionStats {
				redacted_spans: spans.len(),
			},
		}
	}

	/// Recursively redacts all string values inside a JSON value.
	pub fn redact_json_value(&self, value: &mut Value) -> RedactionStats {
		if !self.enabled {
			return RedactionStats::default();
		}

		match value {
			Value::String(text) => {
				let result = self.redact_text_with_stats(text.as_str());
				*text = result.output;
				result.stats
			}
			Value::Array(items) => {
				let mut stats = RedactionStats::default();
				for item in items {
					let child = self.redact_json_value(item);
					stats.redacted_spans =
						stats.redacted_spans.saturating_add(child.redacted_spans);
				}
				stats
			}
			Value::Object(object) => {
				let mut stats = RedactionStats::default();
				for (_key, value) in object.iter_mut() {
					let child = self.redact_json_value(value);
					stats.redacted_spans =
						stats.redacted_spans.saturating_add(child.redacted_spans);
				}
				stats
			}
			_ => RedactionStats::default(),
		}
	}
}

impl Default for Redactor {
	fn default() -> Self {
		Self::try_default().expect("default redaction policy should compile")
	}
}

fn merge_spans(mut spans: Vec<RedactionSpan>) -> Vec<RedactionSpan> {
	spans.retain(RedactionSpan::is_valid);
	if spans.is_empty() {
		return spans;
	}

	spans.sort_by(|left, right| left.start.cmp(&right.start).then(left.end.cmp(&right.end)));

	let mut merged = Vec::new();
	let mut cursor = spans[0];
	for span in spans.into_iter().skip(1) {
		if span.start < cursor.end {
			cursor.end = cursor.end.max(span.end);
			continue;
		}
		merged.push(cursor);
		cursor = span;
	}
	merged.push(cursor);
	merged
}

/// Returns the baseline redaction rules used when `redaction.rules` is not configured.
pub fn default_redaction_rules() -> Vec<RedactionRule> {
	vec![
		RedactionRule::StructuredKeys {
			keys: vec![
				"password".to_string(),
				"passwd".to_string(),
				"pwd".to_string(),
				"secret".to_string(),
				"token".to_string(),
				"api[_-]?key".to_string(),
				"authorization".to_string(),
				"cookie".to_string(),
				"set-cookie".to_string(),
			],
		},
		RedactionRule::BuiltinEmail,
		RedactionRule::BuiltinBearerToken,
	]
}

fn compile_structured_key_rule(keys: &[String]) -> Result<Regex> {
	let alternation = keys
		.iter()
		.map(|key| key.trim())
		.filter(|key| !key.is_empty())
		.collect::<Vec<_>>()
		.join("|");

	if alternation.is_empty() {
		return Regex::new(r"$^").context("failed to compile empty structured-key rule");
	}

	let pattern = format!(
		r#"(?ix)
		(?:^|[\s\{{\[,(])
		\"?(?:{alternation})\"?
		\s*[:=]\s*
		(?:
			Bearer\s+(?P<bearer_token>[^\s,;\}}\]\)]+) |
			\"(?P<value_dq>(?:\\.|[^\"\\])*)\" |
			'(?P<value_sq>(?:\\.|[^'\\])*)' |
			(?P<value_bare>[^\s,;\}}\]\)]+)
		)"#
	);

	Regex::new(pattern.as_str()).context("failed to compile structured-key redaction rule")
}

fn collect_spans_for_rule(rule: &CompiledRule, input: &str) -> Vec<RedactionSpan> {
	match rule {
		CompiledRule::StructuredKeys(regex) => {
			let mut spans = Vec::new();
			for captures in regex.captures_iter(input) {
				let matched = captures
					.name("bearer_token")
					.or_else(|| captures.name("value_dq"))
					.or_else(|| captures.name("value_sq"))
					.or_else(|| captures.name("value_bare"));
				let Some(matched) = matched else {
					continue;
				};
				let start = byte_to_scalar_index(input, matched.start());
				let end = byte_to_scalar_index(input, matched.end());
				let span = RedactionSpan { start, end };
				if span.is_valid() {
					spans.push(span);
				}
			}
			spans
		}
		CompiledRule::Regex(regex) | CompiledRule::Email(regex) => regex
			.find_iter(input)
			.map(|matched| RedactionSpan {
				start: byte_to_scalar_index(input, matched.start()),
				end: byte_to_scalar_index(input, matched.end()),
			})
			.filter(RedactionSpan::is_valid)
			.collect(),
		CompiledRule::BearerToken(regex) => {
			let mut spans = Vec::new();
			for captures in regex.captures_iter(input) {
				let Some(matched) = captures.name("token") else {
					continue;
				};
				let start = byte_to_scalar_index(input, matched.start());
				let end = byte_to_scalar_index(input, matched.end());
				let span = RedactionSpan { start, end };
				if span.is_valid() {
					spans.push(span);
				}
			}
			spans
		}
	}
}

fn apply_replacements(
	input: &str,
	spans: &[RedactionSpan],
	replacement_token: &str,
	preserve_length: bool,
) -> String {
	let char_to_byte = scalar_to_byte_offsets(input);

	let mut output = String::with_capacity(input.len());
	let mut last_end = 0usize;
	for span in spans {
		let start_byte = char_to_byte.get(span.start).copied().unwrap_or(input.len());
		let end_byte = char_to_byte.get(span.end).copied().unwrap_or(input.len());

		if start_byte > last_end {
			output.push_str(&input[last_end..start_byte]);
		}

		let replacement = if preserve_length {
			fit_replacement_token(replacement_token, span.end.saturating_sub(span.start))
		} else {
			replacement_token.to_string()
		};
		output.push_str(replacement.as_str());
		last_end = end_byte;
	}

	if last_end < input.len() {
		output.push_str(&input[last_end..]);
	}

	output
}

fn scalar_to_byte_offsets(input: &str) -> Vec<usize> {
	let mut offsets = Vec::with_capacity(input.chars().count().saturating_add(1));
	for (byte_index, _ch) in input.char_indices() {
		offsets.push(byte_index);
	}
	offsets.push(input.len());
	offsets
}

fn byte_to_scalar_index(input: &str, byte_index: usize) -> usize {
	input
		.get(..byte_index)
		.map(|prefix| prefix.chars().count())
		.unwrap_or_else(|| input.chars().count())
}

/// Fits a replacement token to the desired scalar-length `L`.
///
/// For the default token (`<-REDACTED->`), this implements the exact algorithm in
/// `docs/design/Redaction.md`.
fn fit_replacement_token(token: &str, desired_len: usize) -> String {
	if desired_len == 0 {
		return String::new();
	}

	let token_len = token.chars().count();
	if token_len == desired_len {
		return token.to_string();
	}
	if token_len < desired_len {
		return pad_token(token, desired_len);
	}

	if token == DEFAULT_REPLACEMENT_TOKEN {
		return shrink_default_token(desired_len);
	}

	// Deterministic fallback for non-default tokens.
	minimal_fallback_token(desired_len)
}

fn pad_token(token: &str, desired_len: usize) -> String {
	let mut output = token.to_string();
	let mut current = output.chars().count();
	if current >= desired_len {
		return output;
	}

	if let Some(rest) = output.strip_prefix("<-") {
		let extra = desired_len.saturating_sub(current);
		let mut padded = String::with_capacity(output.len() + extra);
		padded.push_str("<-");
		for _ in 0..extra {
			padded.push('-');
		}
		padded.push_str(rest);
		return padded;
	}

	while current < desired_len {
		output.push('-');
		current += 1;
	}
	output
}

fn shrink_default_token(desired_len: usize) -> String {
	// Start from the base default token.
	let mut chars = DEFAULT_REPLACEMENT_TOKEN.chars().collect::<Vec<_>>();

	// Helper to compute current length.
	let mut current_len = chars.len();
	if current_len == desired_len {
		return chars.into_iter().collect();
	}

	// Step 4a: remove one character at a time from the tail of substring "DETCA" in
	// this order: D, then E, then T, then C, then A.
	for needle in ['D', 'E', 'T', 'C', 'A'] {
		if current_len <= desired_len {
			break;
		}
		if let Some(pos) = chars.iter().rposition(|ch| *ch == needle) {
			chars.remove(pos);
			current_len = current_len.saturating_sub(1);
		}
	}

	// Step 4b: remove dashes from the token body (left-to-right).
	while current_len > desired_len {
		let Some(pos) = chars.iter().position(|ch| *ch == '-') else {
			break;
		};
		chars.remove(pos);
		current_len = current_len.saturating_sub(1);
	}

	if current_len == desired_len {
		return chars.into_iter().collect();
	}

	// Step 4c: minimal fallback for very small L.
	minimal_fallback_token(desired_len)
}

fn minimal_fallback_token(desired_len: usize) -> String {
	match desired_len {
		1 => "*".to_string(),
		2 => "**".to_string(),
		_ => {
			let mut token = String::with_capacity(desired_len);
			token.push('<');
			for _ in 0..desired_len.saturating_sub(2) {
				token.push('-');
			}
			token.push('>');
			token
		}
	}
}
