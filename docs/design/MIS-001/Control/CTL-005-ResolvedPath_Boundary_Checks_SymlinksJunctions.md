# Control: CTL-005 Resolved-Path Boundary Checks (Symlinks/Junctions)

Use resolved/canonical paths for workspace boundary enforcement, accounting for symlinks/junctions and platform-specific path rules; reject ambiguous or unresolvable path inputs.



## Attributes

_No attributes defined._

## Links

- protects [AST-001](../Asset/AST-001-Workspace_Contents.md)
- protects [AST-003](../Asset/AST-003-Nonworkspace_Resources.md)
- enforces [CNS-016](../Constraint/CNS-016-Symlink_and_JunctionSafe_Boundary_Checks.md)
- obstructs [THC-001](../Threat_Capability/THC-001-Prompt_Injection_and_Tool_Misuse.md)
- mitigates [RIS-004](../Risk/RIS-004-Boundary_Bypass_via_SymlinksJunctions.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
