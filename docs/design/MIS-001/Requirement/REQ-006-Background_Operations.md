# Requirement: REQ-006 Background Operations

Alfred MUST support asynchronous execution for bulk filesystem operations only, with deterministic status polling and cancellation via the same tool surface (no standalone job/session introspection tools).



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-006](../Capability/CAP-006-Background_Operations.md)
- imposes [CNS-012](../Constraint/CNS-012-Service_Level_Objectives.md)
- imposes [CNS-019](../Constraint/CNS-019-CrossPlatform_Cancellation_Semantics.md)
- imposes [CNS-022](../Constraint/CNS-022-Network_Filesystem_Tolerance.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T01:23:24Z | copilot | change |
