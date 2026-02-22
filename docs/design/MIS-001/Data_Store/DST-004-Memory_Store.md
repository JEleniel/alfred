# Data Store (Relational): DST-004 Memory Store

Local persistent storage for memory facts and their derived search index.



## Attributes

- **durability**: "persistent"
- **effective_view**: {"id_collision_resolution":"workspace_over_user","mode_default":"merged","modes":["merged","prefer_workspace"],"user_preferences":{"persist_in":"user","workspace_override_allowed":true}}
- **offline_only**: true
- **stores**: {"user":{"default_location":"<userDataDir>/alfred/alfred.sqlite3","notes":"Primary persistent store."},"workspace":{"default_location":"<workspaceRoot>/.agents/alfred/alfred.sqlite3","optional":true}}


## Links

- retrieves [ART-008](../Artifact/ART-008-Memory_Fact.md)
- runs on [NOD-001](../Node/NOD-001-Workspace_Host_Machine.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-22T12:44:59Z | copilot | create |
| 2026-02-22T13:11:53Z | copilot | change |
