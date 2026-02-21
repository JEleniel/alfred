# State Machine: STM-001 Stdio Transport State Machine

Deterministic lifecycle for COM-001 (Stdio Transport) while framing requests and writing responses over stdio.



## Attributes

- **component**: "COM-001"
- **determinism_notes**: ["All reads are framed; partial frames do not surface to the router.","All writes complete or fail; failures are reported via normalized diagnostics."]
- **initial_state**: "STA-001"
- **terminal_states**: ["STA-006"]


## Links

- has [STA-001](../State/STA-001-Transport_Initializing.md)
- has [STA-002](../State/STA-002-Transport_Listening.md)
- has [STA-003](../State/STA-003-Transport_Receiving.md)
- has [STA-004](../State/STA-004-Transport_Writing_Response.md)
- has [STA-005](../State/STA-005-Transport_Faulted.md)
- has [STA-006](../State/STA-006-Transport_Shutdown.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T02:15:00Z | copilot | create |
