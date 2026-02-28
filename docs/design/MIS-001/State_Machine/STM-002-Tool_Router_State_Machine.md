# State Machine: STM-002 Tool Router State Machine

Deterministic request lifecycle for COM-002 (Tool Router): validate, dispatch, stream/wait, and finalize results.



## Attributes

- **component**: "COM-002"
- **determinism_notes**: ["Requests are validated before dispatch; invalid requests fail fast with deterministic taxonomy."]
- **initial_state**: "STA-007"
- **steady_state**: "STA-007"


## References

_No references defined._

## Links

- has [STA-007](../State/STA-007-Router_Idle.md)
- has [STA-008](../State/STA-008-Router_Validating_Request.md)
- has [STA-009](../State/STA-009-Router_Dispatching.md)
- has [STA-010](../State/STA-010-Router_Streaming_or_Waiting.md)
- has [STA-011](../State/STA-011-Router_Complete.md)
- has [STA-012](../State/STA-012-Router_Failed.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T02:15:00Z | copilot | create |
| 2026-02-23T21:04:19Z | copilot | change |
