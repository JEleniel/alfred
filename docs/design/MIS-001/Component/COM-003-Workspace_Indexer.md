# Component (Subsystem): COM-003 Workspace Indexer

Maintains the workspace file index and supports listing/search/range/diff primitives.



## Attributes

- **supports**: ["ls","grep","search","read_range","diff"]


## References

_No references defined._

## Links

- implements [FEA-001](../Feature/FEA-001-Index_and_Query_Tools.md)
- includes [DSR-001](../Data_Source/DSR-001-Workspace_File_Tree.md)
- stores in [DST-001](../Data_Store/DST-001-Index_Store.md)
- produces [ART-005](../Artifact/ART-005-Index_Snapshot.md)
- executes [STM-003](../State_Machine/STM-003-Workspace_Indexer_State_Machine.md)
- runs on [NOD-001](../Node/NOD-001-Workspace_Host_Machine.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T02:15:00Z | copilot | change |
| 2026-02-22T15:41:58Z | copilot | change |
| 2026-02-22T17:35:20Z | copilot | change |
