# Requirement: REQ-003 Safe File Operations

Alfred MUST provide safe file operations including atomic CRUD where practical, patching with conflict reporting, and bulk move/rename/delete/copy with dry-run support.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-003](../Capability/CAP-003-Safe_File_Operations.md)
- imposes [CNS-001](../Constraint/CNS-001-Workspace_Boundary_Enforcement.md)
- imposes [CNS-003](../Constraint/CNS-003-DryRun_for_Destructive_Operations.md)
- imposes [CNS-004](../Constraint/CNS-004-Atomic_Mutations_Per_Target.md)
- imposes [CNS-010](../Constraint/CNS-010-Permission_Model_and_Guardrails.md)
- imposes [CNS-016](../Constraint/CNS-016-Symlink_and_JunctionSafe_Boundary_Checks.md)
- imposes [CNS-017](../Constraint/CNS-017-CrossPlatform_Atomic_Write_Semantics.md)
- imposes [CNS-021](../Constraint/CNS-021-UTF8_Protocol_and_Path_Encoding_Handling.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T01:16:57Z | copilot | change |
