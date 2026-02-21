# State Machine: STM-007 Task Runner State Machine

Deterministic execution lifecycle for COM-009 (Task Runner), including preparation, execution, cancellation, and reporting.



## Attributes

- **component**: "COM-009"
- **determinism_notes**: ["Command invocation is normalized (cwd, env merge, quoting rules) according to platform constraints.","Cancellation always produces a final terminal status and flushes structured output."]
- **initial_state**: "STA-033"


## Links

- has [STA-033](../State/STA-033-Task_Idle.md)
- has [STA-034](../State/STA-034-Task_Preparing.md)
- has [STA-035](../State/STA-035-Task_Running.md)
- has [STA-036](../State/STA-036-Task_Reporting.md)
- has [STA-037](../State/STA-037-Task_Canceling.md)
- has [STA-038](../State/STA-038-Task_Failed.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T02:15:00Z | copilot | create |
