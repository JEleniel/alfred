# Artifact (Data): ART-002 MCP Response

A structured tool result returned over stdio as a single JSON frame. Large result sets SHOULD be carried as bounded JSON arrays/objects. Background execution is limited and exposed only via the bulk filesystem operations contract (no standalone job tools).



## Attributes

- **contracts**: {"protocol":"docs/design/Protocol.md","tool_contracts":"docs/design/ToolContracts.md"}
- **design_goal**: "concise and token-conservative"
- **envelope**: {"failure":{"fields":["status","error","meta"],"status":"error"},"pending":{"fields":["status","data","meta"],"notes":"Used only when bulk filesystem operations run in background mode; the response data includes operation metadata and a poll-with hint.","status":"pending","transport_equivalent":{"http_status":202}},"success":{"fields":["status","data","meta"],"status":"ok"}}
- **errors**: {"taxonomy":"docs/design/ErrorTaxonomy.md"}
- **formats**: ["json"]
- **redaction**: {"notes":"Responses MUST be deterministically redacted before emission, except where a tool contract explicitly allows returning unredacted environment values; such values MUST NOT be logged.","required":true}


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
| 2026-02-22T15:41:58Z | copilot | change |
