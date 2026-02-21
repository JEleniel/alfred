# Requirement: REQ-005 Task Execution

Alfred MUST run named tasks (e.g. build/test) and common ecosystem commands (cargo, pnpm, lint, format).



## Attributes

- **tool_surface**: ["task_run","cargo","pnpm","lint","format"]


## Links

- requires [CAP-005](../Capability/CAP-005-Task_and_Tool_Execution.md)
- imposes [CNS-001](../Constraint/CNS-001-Workspace_Boundary_Enforcement.md)
- imposes [CNS-018](../Constraint/CNS-018-No_Shell_by_Default_for_Task_Execution.md)
- imposes [CNS-019](../Constraint/CNS-019-CrossPlatform_Cancellation_Semantics.md)
- imposes [CNS-020](../Constraint/CNS-020-Environment_Variable_CRUD_Scope.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T01:16:57Z | copilot | change |
