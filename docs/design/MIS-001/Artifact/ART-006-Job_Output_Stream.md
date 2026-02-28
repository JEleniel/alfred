# Artifact (Data): ART-006 Job Output Stream

A stream of output chunks and structured status for a background job.



## Attributes

- **contracts**: {"errors":"docs/design/ErrorTaxonomy.md","protocol":"docs/design/Protocol.md"}
- **cursor**: {"encoding":"decimal_string","monotonic":true,"type":"seq"}
- **formats**: ["ndjson","json"]
- **ndjson**: {"item_kinds":["log","stdout","stderr","status","progress","result","error"],"log_record_schema":{"extra":"object (string-to-string pairs)","level":"string (TRACE|DEBUG|INFO|WARN|ERROR)","message":"string","source":"string (e.g., alfred::configuration::load)","timestamp":"string (RFC3339 UTC; seconds preferred; max milliseconds)"},"ordering":"Stable by emission order within the job.","rules":"Each line is a complete JSON object. Callers MUST tolerate chunking and resume using cursors/job offsets."}
- **redaction**: {"notes":"All stream items MUST be redacted deterministically before persistence or emission.","required":true}
- **schema**: "schemas/alfred.job-stream-item.schema.json"
- **semantics**: "append-only"


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
| 2026-02-22T13:04:50Z | copilot | change |
| 2026-02-22T13:17:41Z | copilot | change |
| 2026-02-22T17:35:20Z | copilot | change |
