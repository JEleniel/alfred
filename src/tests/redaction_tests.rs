use crate::redaction::{DEFAULT_REPLACEMENT_TOKEN, RedactionRule, Redactor};

#[test]
fn redacts_structured_secret_fields() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	let input = "token=abc123 password: letmein api_key='abc-def'";
	let result = redactor.redact_text_with_stats(input);
	let output = result.output;

	assert_eq!(result.stats.redacted_spans, 3);

	// preserve_length=true by default, so replacements are fitted to the value length.
	assert!(output.contains("token=<RED->"));
	assert!(output.contains("password: <-RED->"));
	assert!(output.contains("api_key='<-RED->'"));
}

#[test]
fn redacts_email_and_bearer_patterns() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	let input = "Contact admin@example.com with Authorization: Bearer abc.def.ghi";
	let output = redactor.redact_text(input);

	assert!(!output.contains("admin@example.com"));
	assert!(!output.contains("abc.def.ghi"));
	assert!(output.contains("Bearer "));
	assert!(output.contains('<'));
}

#[test]
fn preserves_non_sensitive_text() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	let input = "workspace indexed with 42 files";
	let output = redactor.redact_text(input);

	assert_eq!(input, output);
}

#[test]
fn redacts_json_like_structured_fields_without_breaking_quotes() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	let input = r#"{"token":"abc123","nested":{"password":"letmein"}}"#;
	let output = redactor.redact_text(input);

	assert!(output.contains(r#""token":"<RED->""#));
	assert!(output.contains(r#""password":"<-RED->""#));
}

#[test]
fn uses_minimal_fallback_tokens_for_tiny_spans() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	assert_eq!(redactor.redact_text("token=a"), "token=*");
	assert_eq!(redactor.redact_text("token=ab"), "token=**");
}

#[test]
fn custom_rules_can_disable_structured_key_matching() {
	let redactor = Redactor::try_from_rules(
		true,
		DEFAULT_REPLACEMENT_TOKEN.to_string(),
		true,
		vec![RedactionRule::Regex {
			pattern: "(?i)example\\.com".to_string(),
			case_sensitive: false,
		}],
	)
	.expect("custom redactor should build");

	let input = "token=abc123 contact admin@example.com";
	let output = redactor.redact_text(input);

	// token should remain (structured-key rules were not configured).
	assert!(output.contains("token=abc123"));
	assert!(!output.contains("admin@example.com"));
}
