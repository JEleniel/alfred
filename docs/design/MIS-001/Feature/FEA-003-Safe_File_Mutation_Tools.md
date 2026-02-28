# Feature: FEA-003 Safe File Mutation Tools

Implement atomic CRUD (where practical), patch with conflict reporting, and bulk operations with dry-run.



## Attributes

- **contracts**: {"tool_contracts":"docs/design/ToolContracts.md"}
- **tools**: ["fs_operations","bulk_fs_operations","patch"]


## References

_No references defined._

## Links

- realizes [CAP-003](../Capability/CAP-003-Safe_File_Operations.md)
- implies [CNS-001](../Constraint/CNS-001-Workspace_Boundary_Enforcement.md)
- implies [CNS-003](../Constraint/CNS-003-DryRun_for_Destructive_Operations.md)
- implies [CNS-004](../Constraint/CNS-004-Atomic_Mutations_Per_Target.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-22T16:01:08Z | copilot | change |
