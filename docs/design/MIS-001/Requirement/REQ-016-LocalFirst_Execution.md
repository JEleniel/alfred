# Requirement: REQ-016 Local-First Execution

Alfred MUST run as a local stdio server relative to the workspace host (including VS Code Remote Development modes), enforce workspace boundaries, remain self-contained (no outside services), and support Linux/macOS/Windows.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-007](../Capability/CAP-007-Deterministic_Contracts.md)
- imposes [CNS-001](../Constraint/CNS-001-Workspace_Boundary_Enforcement.md)
- imposes [CNS-006](../Constraint/CNS-006-No_External_Service_Dependencies.md)
- imposes [CNS-007](../Constraint/CNS-007-Local_Stdio_Server.md)
- imposes [CNS-010](../Constraint/CNS-010-Permission_Model_and_Guardrails.md)
- imposes [CNS-011](../Constraint/CNS-011-Git_and_GitHub_Operations_Out_of_Scope.md)
- imposes [CNS-013](../Constraint/CNS-013-CrossPlatform_Desktop_OS_Support.md)
- imposes [CNS-014](../Constraint/CNS-014-Mobile_Platforms_Out_of_Scope.md)
- imposes [CNS-023](../Constraint/CNS-023-VS_Code_Remote_Development_Compatibility.md)
- has [ADR-001](../ADR/ADR-001-Rust_as_Implementation_Language.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T00:09:30Z | copilot | change |
| 2026-02-21T01:10:46Z | copilot | change |
| 2026-02-21T01:23:24Z | copilot | change |
