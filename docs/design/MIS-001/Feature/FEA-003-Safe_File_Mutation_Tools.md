# Feature: FEA-003 Safe File Mutation Tools

Implement atomic CRUD (where practical), patch with conflict reporting, bulk ops with dry-run, and bounded byte-chunk file mutation for binary/any-size files.



## Attributes

- **contracts**: {"tool_contracts":"docs/design/ToolContracts.md"}
- **tools**: ["file_create","file_append","file_patch","multi_file_patch","file_delete","dir_create","dir_delete","file_create_bytes","file_append_bytes","path_move","path_copy","path_delete"]


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
