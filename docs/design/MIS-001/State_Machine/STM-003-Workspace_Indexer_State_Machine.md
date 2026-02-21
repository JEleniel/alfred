# State Machine: STM-003 Workspace Indexer State Machine

Deterministic indexing lifecycle for COM-003 (Workspace Indexer) supporting queries, rebuilds, and safe degradation on network/remote workspaces.



## Attributes

- **component**: "COM-003"
- **determinism_notes**: ["Index builds use deterministic enumeration and normalization.","When change visibility is uncertain, reconciliation replaces watch-only semantics."]
- **initial_state**: "STA-013"
- **steady_state**: "STA-015"


## Links

- has [STA-013](../State/STA-013-Indexer_Initializing.md)
- has [STA-014](../State/STA-014-Indexer_Building_Index.md)
- has [STA-015](../State/STA-015-Indexer_Ready.md)
- has [STA-016](../State/STA-016-Indexer_Rebuilding.md)
- has [STA-017](../State/STA-017-Indexer_Failed.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T02:15:00Z | copilot | create |
