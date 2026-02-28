# Data Store (Index): DST-004 Memory Store

Local persistent storage for memory facts and their derived search index.



## Attributes

- **durability**: "persistent"
- **effective_view**: {"id_collision_resolution":"workspace_over_user","id_format":"uuid","id_issued_by":"alfred","mode_default":"merged","modes":["merged","prefer_workspace"],"scope_required_for_writes":true,"user_preferences":{"persist_in":"user","workspace_override_allowed":true},"workspace_relocated_default_location":"<userDataDir>/alfred/<workspace_id>/data/memory/"}
- **offline_only**: true
- **stores**: {"user":{"default_location":"<userDataDir>/alfred/memory/","notes":"User-scoped (global) memory store and index."},"workspace":{"default_location":"<workspaceRoot>/.alfred/memory/","notes":"Workspace-scoped memory store and index (when enabled).","optional":true}}


## References

_No references defined._

## Links

- retrieves [ART-008](../Artifact/ART-008-Memory_Fact.md)
- runs on [NOD-001](../Node/NOD-001-Workspace_Host_Machine.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-22T12:44:59Z | copilot | create |
| 2026-02-22T13:11:53Z | copilot | change |
| 2026-02-28T00:00:00Z | copilot | change |
| 2026-02-28T09:55:00Z | copilot | change |
