# Constraint: CNS-010 Permission Model and Guardrails

Alfred MUST provide an IDE-compliant permission model and configurable policy guardrails with reasonable defaults. Tool exposure and plan writes MUST be governed by deterministic policies.



## Attributes

- **config_precedence**: {"workspace_overrides_user":true}
- **external_tools**: {"error_kind_on_missing":"tool_unavailable","probe_methods":["--version","--help"],"require_probe":true}
- **plan_writes**: {"atomic_write_required":true,"concurrent_writers":"unsupported_last_write_wins","lock_files_forbidden":true}
- **tool_exposure**: {"calling_disabled_tool_is_error":true,"disabled_tools_omitted_from_capabilities":true,"error_kind":"invalid_argument"}


## References

- [docs/design/ToolContracts.md](../../aurora/MIS-001/Constraint/docs/design/ToolContracts.md)
- [docs/design/Configuration.md](../../aurora/MIS-001/Constraint/docs/design/Configuration.md)


## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-22T17:35:20Z | copilot | change |
| 2026-02-23T21:04:19Z | copilot | change |
