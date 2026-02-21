# Component (Subsystem): COM-001 Stdio Transport

Implements stdio request/response transport and framing for MCP tool calls.



## Attributes

- **streaming**: true
- **transport**: "stdio"


## Links

- exposes [INT-001](../Interface/INT-001-MCP_Stdio_Interface.md)
- implements [FEA-007](../Feature/FEA-007-Deterministic_Response_Contracts.md)
- produces [ART-002](../Artifact/ART-002-MCP_Response.md)
- executes [STM-001](../State_Machine/STM-001-Stdio_Transport_State_Machine.md)
- runs on [NOD-001](../Node/NOD-001-Workspace_Host_Machine.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T02:15:00Z | copilot | change |
