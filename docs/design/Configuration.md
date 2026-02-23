# Alfred Configuration

This document specifies Alfred’s configuration model: file locations, precedence, merge rules, and key defaults.

## Configuration levels and precedence

Alfred reads configuration from two sources:

- User configuration: applies to all workspaces on a machine.
- Workspace configuration: applies only within a single workspace.

Workspace configuration MUST override user configuration.

## Default locations

Configuration file locations are implementation-defined. Default locations SHOULD be:

- User configuration: OS config directory `alfred/config.json`.
- Workspace configuration: `.agents/alfred/config.json` under the workspace root.

## Merge rules

Configuration is merged deterministically.

- Objects: deep-merge (workspace overrides user).
- Arrays:
    - `tools.disabled`: union of user + workspace values, de-duplicated, stable-sorted.
    - Other arrays: workspace overrides user unless otherwise specified.

## Schema

The workspace and user configuration format is JSON and MUST conform to `docs/design/schemas/alfred.config.schema.json`.

## Keys

### Tool enablement

- `tools.disabled`: array of tool names to disable.

Semantics:

- Disabled tools MUST be omitted from `capabilities`.
- Calls to disabled tools MUST fail with `error.kind: "invalid_argument"`.

### Indexing

- `index.enabled`: boolean (default `true`). When `false`, index-backed tools MUST return `tool_unavailable` with `details.reason: "index_disabled"`.
- `index.persistence.enabled`: boolean (default `true`).
- `index.persistence.path`: workspace-relative path (default `.agents/alfred/index/`).
- `index.persistence.interval_seconds`: integer (default `60`).
- `index.ignore.patterns`: array of gitignore-style patterns.
- `index.ignore.always_patterns`: array of gitignore-style patterns that MUST always be applied and MUST NOT be overridden.

Default `always_patterns` includes (minimum):

- `.git/`
- `.agents/`

### Redaction

- `redaction.enabled`: boolean (default `true`).
- `redaction.replacement_token`: string (default `<-REDACTED->`).
- `redaction.preserve_length`: boolean (default `true`).
- `redaction.rules`: rule list; supports structured-key rules and regex rules.

Rule evaluation order MUST be stable and MUST match [`docs/design/Redaction.md`](./Redaction.md).

### Plan

- `plan.path`: workspace-relative path.

Default selection is:

1. `docs/design/ProjectPlan.md` if it exists.
2. `ProjectPlan.md` otherwise.

Plans are always workspace-scoped.

### Tasks

- `task.working_directory`: workspace-relative path (default `.`).
- `task.env.allowlist`: array of environment variable names (default OS-specific allowlist).

External tool probes:

- `task.external_tools`: object mapping tool name to probe configuration.
    - Example keys: `npm`, `pnpm`.
    - Each entry includes `probe_args` (default `["--version"]`) and `timeout_ms` (default `2000`).
