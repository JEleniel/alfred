# Component (Subsystem): COM-017 Memory Manager

Provides CRUD for memory facts and deterministic full-text search over stored facts, maintaining a local index. Supports subject/category/tag filtering and an effective view over user/workspace stores.



## Attributes

- **indexing**: {"default_limit":100,"filters":["subject","category","tags"],"mode":"local"}
- **redaction**: "deterministic"
- **storage**: {"effective_view":"merged (default) or prefer_workspace","user_store":"DST-004","workspace_store":"DST-004"}
- **tool_surface**: ["memory"]


## References

_No references defined._

## Links

- implements [FEA-014](../Feature/FEA-014-Local_Indexed_Memory_Tooling.md)
- stores in [DST-004](../Data_Store/DST-004-Memory_Store.md)
- produces [ART-008](../Artifact/ART-008-Memory_Fact.md)
- runs on [NOD-001](../Node/NOD-001-Workspace_Host_Machine.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-22T12:44:59Z | copilot | create |
| 2026-02-22T13:04:50Z | copilot | change |
| 2026-02-22T13:11:53Z | copilot | change |
