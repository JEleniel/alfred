# State Machine: STM-005 File Operations Engine State Machine

Deterministic mutation lifecycle for COM-007 (File Operations Engine), including dry-run planning, execution, and rollback semantics.



## Attributes

- **component**: "COM-007"
- **determinism_notes**: ["All mutations are planned before execution; dry-run produces the plan and never writes.","Where atomic operations are not possible, compensating actions are explicit and deterministic."]
- **initial_state**: "STA-021"


## References

_No references defined._

## Links

- has [STA-021](../State/STA-021-FileOps_Idle.md)
- has [STA-022](../State/STA-022-FileOps_Validating_Plan.md)
- has [STA-023](../State/STA-023-FileOps_Executing.md)
- has [STA-024](../State/STA-024-FileOps_Committing.md)
- has [STA-025](../State/STA-025-FileOps_Rolling_Back.md)
- has [STA-026](../State/STA-026-FileOps_Failed.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T02:15:00Z | copilot | create |
