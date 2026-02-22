# Interface: INT-001 MCP Stdio Interface

The stdio contract used by the MCP host to call Alfred tools and receive structured results.



## Attributes

- **contracts**: {"errors":"docs/design/ErrorTaxonomy.md","protocol":"docs/design/Protocol.md","tool_contracts":"docs/design/ToolContracts.md"}
- **determinism**: {"explicit_pagination":true,"stable_ordering":true}
- **encoding**: "utf-8"
- **framing**: "newline-delimited JSON frames (one JSON value per line)"
- **transport**: "stdio"


## Links

- accepts [ART-001](../Artifact/ART-001-MCP_Request.md)
- returns [ART-002](../Artifact/ART-002-MCP_Response.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-22T12:44:59Z | copilot | change |
