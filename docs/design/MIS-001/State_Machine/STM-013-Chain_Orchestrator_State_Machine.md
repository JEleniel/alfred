# State Machine: STM-013 Chain Orchestrator State Machine

Deterministic lifecycle for COM-015 (Chain Orchestrator): build chain, execute steps, stop on failure, and report.



## Attributes

- **component**: "COM-015"
- **determinism_notes**: ["Step ordering is explicit and stable; stop-on-failure is enforced.","Each step emits deterministic status transitions."]
- **initial_state**: "STA-061"


## Links

- has [STA-061](../State/STA-061-Chain_Idle.md)
- has [STA-062](../State/STA-062-Chain_Building.md)
- has [STA-063](../State/STA-063-Chain_Executing.md)
- has [STA-064](../State/STA-064-Chain_Stopping.md)
- has [STA-065](../State/STA-065-Chain_Complete.md)
- has [STA-066](../State/STA-066-Chain_Failed.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T02:15:00Z | copilot | create |
