# Alfred Configuration

This document specifies Alfred's configuration model: file locations, precedence, merge rules, and key defaults.

## Configuration levels and precedence

Alfred reads configuration from two sources:

- User configuration: applies to all workspaces on a machine.
- Workspace configuration: applies only within a single workspace.

Workspace configuration MUST override user configuration.

## Default locations

Configuration file locations are implementation-defined. Default locations SHOULD be:

- User configuration: OS config directory `alfred/config.json`.
- Workspace configuration: `<workspaceRoot>/.alfred/config.json`.

## Merge rules

Configuration is merged deterministically.

- Objects: deep-merge (workspace overrides user).
- Arrays:
    - `tools.disabled`: union of user + workspace values, de-duplicated, stable-sorted.
    - Other arrays: workspace overrides user unless otherwise specified.

## Schema

The workspace and user configuration format is JSON and MUST conform to `schemas/alfred.config.schema.json`.

## Keys

### Tool enablement

- `tools.disabled`: array of tool names to disable.

Semantics:

- Disabled tools MUST be omitted from `capabilities`.
- Calls to disabled tools MUST fail with `error.kind: "invalid_argument"`.

### Workspace storage root

- `workspace.storage.root`: workspace-relative folder used for Alfred workspace-owned artifacts.
    - Default: `.alfred/`.

The resolved root is used by default for index and workspace-memory persistence unless a more specific path is configured.

### Indexing

- `index.enabled`: boolean (default `true`).
    - When `false`, index-backed tools MUST return `tool_unavailable` with `details.reason: "index_disabled"`.
- `index.persistence.enabled`: boolean (default `true`).
- `index.persistence.location`: `"workspace" | "user"` (default `"workspace"`).
- `index.persistence.path`: string.
    - If `location = "workspace"`, default `<workspace.storage.root>/index/`.
    - If `location = "user"`, default `workspaces/<workspace_id>/index/` under user data.
- `index.persistence.interval_seconds`: integer (default `60`).
- `index.ignore.patterns`: array of gitignore-style patterns.
- `index.ignore.always_patterns`: array of gitignore-style patterns that MUST always be applied and MUST NOT be overridden.

Default `index.ignore.always_patterns` includes (minimum):

- `.git/`
- `.alfred/`

Single-location rule:

- For a given workspace, Alfred MUST persist the index in exactly one location selected by `index.persistence.location`.
- If `index.persistence.location = "workspace"`, Alfred MUST NOT also maintain a duplicate workspace index in user data for that workspace.

### Memory storage

- `memory.storage.user.enabled`: boolean (default `true`).
- `memory.storage.workspace.enabled`: boolean (default `false`).
- `memory.storage.workspace.path`: workspace-relative path (default `<workspace.storage.root>/memory/alfred.sqlite3`).
- `memory.storage.user.path`: user-data-relative path (default `alfred/alfred.sqlite3`).
- `memory.storage.merge_mode`: `"union_workspace_wins" | "prefer_workspace_with_user_preferences_fallback"`.
    - Default: `"union_workspace_wins"`.

Notes:

- Enabling workspace memory storage provides per-workspace persistence under the configurable workspace storage root.
- Effective read behavior for user+workspace stores is defined in [`docs/design/ToolContracts.md`](./ToolContracts.md).

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
