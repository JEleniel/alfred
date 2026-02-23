# Feature: FEA-001 Index and Query Tools

Provide ls/grep/search/range/diff primitives backed by a workspace index, including deterministic file metadata and bounded byte reads.



## Attributes

- **contracts**: {"tool_contracts":"docs/design/ToolContracts.md"}
- **tools**: ["ls","grep","search","read_range","diff","file_stat","file_read_bytes"]


## Links

- realizes [CAP-001](../Capability/CAP-001-Workspace_Index_and_Query.md)
- implies [CNS-001](../Constraint/CNS-001-Workspace_Boundary_Enforcement.md)
- implies [CNS-002](../Constraint/CNS-002-SideEffect_Free_Reads.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-22T16:11:03Z | copilot | change |
| 2026-02-22T17:35:20Z | copilot | change |
