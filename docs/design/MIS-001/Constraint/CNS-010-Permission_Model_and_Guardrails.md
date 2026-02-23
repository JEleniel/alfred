# Constraint: CNS-010 Permission Model and Guardrails

Alfred MUST provide an IDE-compliant permission model and configurable policy guardrails with reasonable defaults. Tool exposure, task execution, and plan writes MUST be governed by deterministic policies.



## Attributes

- **config_precedence**: {"workspace_overrides_user":true}
- **external_tools**: {"error_kind_on_missing":"tool_unavailable","probe_methods":["--version","--help"],"require_probe":true}
- **plan_writes**: {"error_kind_on_lock_failure":"conflict","require_exclusive_lock":true}
- **references**: {"configuration":"docs/design/Configuration.md","tool_contracts":"docs/design/ToolContracts.md"}
- **task_execution**: {"env_allowlist_required":true,"shell_disabled_by_default":true}
- **tool_exposure**: {"calling_disabled_tool_is_error":true,"chain_referencing_disabled_tool_is_error":true,"disabled_tools_omitted_from_capabilities":true,"error_kind":"invalid_argument"}


## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-22T17:35:20Z | copilot | change |
