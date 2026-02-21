# Component (Subsystem): COM-007 File Operations Engine

Performs safe workspace mutations: atomic CRUD (where practical), patch apply with conflict reporting, and bulk ops with dry-run.



## Attributes

- **reports_conflicts**: true
- **supports_dry_run**: true


## Links

- implements [FEA-003](../Feature/FEA-003-Safe_File_Mutation_Tools.md)
- includes [DSR-001](../Data_Source/DSR-001-Workspace_File_Tree.md)
- executes [STM-005](../State_Machine/STM-005-File_Operations_Engine_State_Machine.md)
- runs on [NOD-001](../Node/NOD-001-Workspace_Host_Machine.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T02:15:00Z | copilot | change |
