# Control: CTL-009 Encoding-Safe Path Handling and Reporting

Treat filesystem paths as potentially non-UTF8; ensure protocol outputs remain valid UTF-8 JSON by using deterministic escaping/encoding for unrepresentable paths.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- enforces [CNS-021](../Constraint/CNS-021-UTF8_Protocol_and_Path_Encoding_Handling.md)
- protects [AST-001](../Asset/AST-001-Workspace_Contents.md)
- protects [AST-003](../Asset/AST-003-Nonworkspace_Resources.md)
- mitigates [RIS-009](../Risk/RIS-009-Path_Encoding_NonUTF8_Filenames_Break_Protocol.md)
- mitigates [RIS-006](../Risk/RIS-006-Nondeterministic_Output_Across_OSFilesystems.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
