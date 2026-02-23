//! Deterministic redaction helpers for log-safe output.

use anyhow::{Context, Result};
use regex::Regex;

/// Default replacement token for redacted content.
pub const DEFAULT_REPLACEMENT_TOKEN: &str = "<-REDACTED->";

/// Redacts non-public information from log messages.
#[derive(Debug, Clone)]
pub struct Redactor {
	replacement_token: String,
	structured_key_rule: Regex,
	email_rule: Regex,
	bearer_rule: Regex,
}

impl Redactor {
	/// Builds the default redaction policy used by logging.
	pub fn default() -> Result<Self> {
		let structured_key_rule = Regex::new(
			r#"(?i)(\b(?:password|passwd|pwd|secret|token|api[_-]?key|authorization|cookie|set-cookie)\b\s*[:=]\s*)(?:Bearer\s+[^\s,;]+|"[^"]*"|'[^']*'|[^\s,;]+)"#,
		)
		.context("failed to compile structured-key redaction rule")?;

		let email_rule = Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b")
			.context("failed to compile email redaction rule")?;

		let bearer_rule = Regex::new(r"(?i)\bBearer\s+[A-Za-z0-9._\-~+/]+=*")
			.context("failed to compile bearer-token redaction rule")?;

		Ok(Self {
			replacement_token: DEFAULT_REPLACEMENT_TOKEN.to_string(),
			structured_key_rule,
			email_rule,
			bearer_rule,
		})
	}

	/// Redacts deterministic NPI patterns from the provided text.
	pub fn redact_text(&self, input: &str) -> String {
		let mut redacted = self
			.structured_key_rule
			.replace_all(input, format!("$1{}", self.replacement_token).as_str())
			.into_owned();

		redacted = self
			.email_rule
			.replace_all(redacted.as_str(), self.replacement_token.as_str())
			.into_owned();

		self.bearer_rule
			.replace_all(
				redacted.as_str(),
				format!("Bearer {}", self.replacement_token).as_str(),
			)
			.into_owned()
	}
}
