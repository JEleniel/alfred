# Control: CTL-006 Cross-Platform Atomic Write and Replace

Implement atomic writes using temp files + sync + replace semantics appropriate to each OS; detect locked destinations (common on Windows) and fail explicitly rather than partially applying changes.



## Attributes

_No attributes defined._

## Links

- protects [AST-001](../Asset/AST-001-Workspace_Contents.md)
- enforces [CNS-017](../Constraint/CNS-017-CrossPlatform_Atomic_Write_Semantics.md)
- mitigates [RIS-005](../Risk/RIS-005-NonAtomic_File_Replace_Due_to_Locking.md)
- mitigates [RIS-003](../Risk/RIS-003-Workspace_Corruption.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
