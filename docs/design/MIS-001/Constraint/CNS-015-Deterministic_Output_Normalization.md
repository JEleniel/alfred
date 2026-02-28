# Constraint: CNS-015 Deterministic Output Normalization

Alfred MUST produce deterministic outputs across supported OSes (Linux/macOS/Windows), including stable sorting, stable path normalization, and stable formatting regardless of filesystem enumeration order or case-sensitivity defaults.



## Attributes

- **errors**: {"must_use_taxonomy":"docs/design/ErrorTaxonomy.md"}
- **ordering**: {"string_order":"case_insensitive_unicode_lexicographic","tie_breaker":"case_sensitive_lexicographic"}
- **pagination**: {"must_be_stable":true}
- **redaction**: {"algorithm":"docs/design/Redaction.md","must_be_deterministic":true,"token":"<-REDACTED->"}


## References

- [docs/design/ToolContracts.md](../../aurora/MIS-001/Constraint/docs/design/ToolContracts.md)
- [docs/design/Protocol.md](../../aurora/MIS-001/Constraint/docs/design/Protocol.md)


## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
| 2026-02-22T17:35:20Z | copilot | change |
