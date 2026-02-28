use crate::redaction::{DEFAULT_REPLACEMENT_TOKEN, Redactor};

#[test]
fn redacts_structured_secret_fields() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	let input = "token=abc123 password: letmein api_key='abc-def'";
	let output = redactor.redact_text(input);

	assert!(output.contains(&format!("token={DEFAULT_REPLACEMENT_TOKEN}")));
	assert!(output.contains(&format!("password: {DEFAULT_REPLACEMENT_TOKEN}")));
	assert!(output.contains(&format!("api_key={DEFAULT_REPLACEMENT_TOKEN}")));
}

#[test]
fn redacts_email_and_bearer_patterns() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	let input = "Contact admin@example.com with Authorization: Bearer abc.def.ghi";
	let output = redactor.redact_text(input);

	assert!(!output.contains("admin@example.com"));
	assert!(!output.contains("abc.def.ghi"));
	assert!(output.contains(DEFAULT_REPLACEMENT_TOKEN));
}

#[test]
fn preserves_non_sensitive_text() {
	let redactor = Redactor::try_default().expect("redactor should initialize");
	let input = "workspace indexed with 42 files";
	let output = redactor.redact_text(input);

	assert_eq!(input, output);
}
