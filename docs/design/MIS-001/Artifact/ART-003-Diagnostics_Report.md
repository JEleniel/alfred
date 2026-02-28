# Artifact (Report): ART-003 Diagnostics Report

Normalized diagnostics output emitted by build/test/lint/format tooling, optionally with deltas between runs.



## Attributes

- **contracts**: {"errors":"docs/design/ErrorTaxonomy.md","protocol":"docs/design/Protocol.md"}
- **format**: ["json","ndjson"]
- **notes**: ["When emitted as NDJSON, each line is a complete JSON object and the stream is append-only.","Diagnostics envelopes are designed to be validated against a deterministic schema and taxonomy."]
- **schema**: "schemas/alfred.diagnostics.schema.json"
- **sources**: ["cargo","pnpm","linters","formatters"]


## References

_No references defined._

## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-22T12:44:59Z | copilot | change |
| 2026-02-22T15:41:58Z | copilot | change |
| 2026-02-22T17:35:20Z | copilot | change |
| 2026-02-23T21:04:19Z | copilot | change |
