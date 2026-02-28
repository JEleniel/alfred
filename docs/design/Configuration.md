# Alfred Configuration

This document specifies Alfred's configuration model: file locations, precedence, merge rules, and key defaults.

## Configuration levels and precedence

Alfred reads configuration from two sources:

- User configuration: applies to all workspaces on a machine.
- Workspace configuration: applies only within a single workspace.

Workspace configuration MUST override user configuration.

## Default locations

Configuration file locations are implementation-defined. Default locations SHOULD follow a simple rule:

- All user-scoped files live under the OS user config directory or user data directory.
- All workspace-scoped files live under the workspace root.

To avoid repeating full paths throughout this document, default locations are consolidated here.

- User configuration directory: `alfred/`
    - `config.json`
    - `.alfredignore`
    - Optional per-workspace configuration (when workspace-scoped configuration is relocated to user configuration):
        - `<workspace_id>/config.json`
        - `<workspace_id>/.alfredignore`
- User data directory: `alfred/`
    - User-scoped memory: `memory/`
    - Optional per-workspace data (when workspace-scoped data is relocated to user data):
        - `<workspace_id>/data/index/`
        - `<workspace_id>/data/memory/`
        - `<workspace_id>/data/logs/`
- Workspace root:
    - `.alfredignore`
    - Workspace storage root: `<workspace.storage.root>/` (default `.alfred/`)
        - `config.json`
        - `index/`
        - `memory/`
        - `logs/`
        - Optional workspace-local user profile (when user-scoped storage is disabled and relocated into the workspace):
            - `user/config.json`
            - `user/.alfredignore`
            - `user/memory/`

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
- Calls to disabled tools MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).

### Storage location controls

Alfred supports three storage layouts:

- Default: user-scoped configuration/data are stored in OS user directories, and workspace-scoped configuration/data are stored in the workspace.
- Workspace-relocated: workspace-scoped configuration/data are stored in OS user directories keyed by workspace id (to avoid writing Alfred artifacts into the workspace).
- Workspace-local user profile: user-scoped configuration/data are stored under the workspace storage root (to avoid reading/writing OS user directories).

Keys:

- `storage.user.location`: optional string (default `"os"`), one of:
    - `"os"`: use OS user directories.
    - `"workspace"`: store user-scoped config/data under `<workspace.storage.root>/user/`.
- `storage.workspace.location`: optional string (default `"workspace"`), one of:
    - `"workspace"`: store workspace-scoped config/data under `<workspace.storage.root>/`.
    - `"user"`: store workspace-scoped config/data under OS user directories keyed by workspace id.

Notes:

- The configuration precedence rules do not change; only the persistence locations change.
- Anything stored under OS user directories MUST preserve the separation between configuration (OS config dir) and data (OS data dir).

### Path resolution (deterministic defaults)

This section summarizes how Alfred resolves default paths when optional overrides are not provided.

Terminology:

- `workspaceRoot`: the workspace boundary root.
- `<workspace.storage.root>`: workspace storage root folder under `workspaceRoot` (default `.alfred/`).
- `<workspace_id>`: a stable workspace identifier derived from `workspaceRoot`.
- `<user config>` and `<user data>`: OS-provided user configuration and user data locations.

Default layout (`storage.user.location = "os"`, `storage.workspace.location = "workspace"`):

- Workspace config: `workspaceRoot/<workspace.storage.root>/config.json`.
- Workspace index: `workspaceRoot/<workspace.storage.root>/index/`.
- Workspace memory (when enabled): `workspaceRoot/<workspace.storage.root>/memory/`.
- Runtime logs (when `logging.runtime.location = "workspace"` or as a fallback): `workspaceRoot/<workspace.storage.root>/logs/`.
- User config: `<user config>/alfred/config.json`.
- User memory: `<user data>/alfred/memory/`.

Workspace-relocated layout (`storage.workspace.location = "user"`):

- Workspace config: `<user config>/alfred/<workspace_id>/config.json`.
- Workspace index: `<user data>/alfred/<workspace_id>/data/index/`.
- Workspace memory (when enabled): `<user data>/alfred/<workspace_id>/data/memory/`.

Workspace-local user profile (`storage.user.location = "workspace"`):

- User config: `workspaceRoot/<workspace.storage.root>/user/config.json`.
- User memory: `workspaceRoot/<workspace.storage.root>/user/memory/`.

### Workspace storage root

- `workspace.storage.root`: workspace-relative folder used for Alfred workspace-owned artifacts.
    - Default: `.alfred/`.

The resolved root is used by default for workspace-scoped persistence (index, workspace memory, and workspace logs) when `storage.workspace.location = "workspace"`.

### Indexing

- `index.enabled`: boolean (default `true`).
    - When `false`, index-backed tools MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).
- `index.persistence.enabled`: boolean (default `true`).
- `index.persistence.location`: `"workspace" | "user"` (default `"workspace"`).
- `index.persistence.path`: string (optional override).
    - When unset, uses the default location for the selected persistence location (see [Default locations](#default-locations)).
- `index.persistence.interval_seconds`: integer (default `30`).

When `storage.workspace.location = "user"` and `index.persistence.location = "workspace"`, the default workspace index root is under the per-workspace user data path:

- `<user data>/alfred/<workspace_id>/data/index/`

Index ignore rules are driven by `.alfredignore` files (gitignore-style syntax) rather than JSON configuration.

Ignore layers (in order):

- Always excluded patterns: MUST always be ignored and MUST NOT be overridden.
- Built-in default `.alfredignore` patterns: applied by default, but MAY be overridden by whitelist rules.
- User `.alfredignore`: applied before workspace rules.
- Workspace `.alfredignore`: applied additively from workspace root down to subfolders.

The default lists are defined in [`docs/design/DefaultIncludeExcludeList.md`](./DefaultIncludeExcludeList.md).

Single-location rule:

- For a given workspace, Alfred MUST persist the index in exactly one location selected by `index.persistence.location`.
- If `index.persistence.location = "workspace"`, Alfred MUST NOT also maintain a duplicate workspace index in user data for that workspace.

### Memory storage

- `memory.storage.user.enabled`: boolean (default `true`).
- `memory.storage.workspace.enabled`: boolean (default `false`).
- `memory.storage.workspace.path`: path (optional override).
- `memory.storage.user.path`: path (optional override).
- `memory.storage.merge_mode`: `"union_workspace_wins" | "prefer_workspace_with_user_preferences_fallback"`.
    - Default: `"union_workspace_wins"`.

Notes:

- Enabling workspace memory storage provides per-workspace persistence under the configurable workspace storage root.
- Effective read behavior for user+workspace stores is defined in [`docs/design/ToolContracts.md`](./ToolContracts.md).

Merge-mode semantics (effective reads):

- `union_workspace_wins`:
    - Alfred MUST treat the effective memory corpus as the union of enabled stores.
    - If the same `id` exists in more than one enabled scope, Alfred MUST prefer the workspace-scoped fact deterministically.
- `prefer_workspace_with_user_preferences_fallback`:
    - Alfred MUST treat workspace-scoped facts as authoritative when both scopes contain a fact for the same `id`.
    - When listing/searching, Alfred SHOULD order workspace facts ahead of user facts when other ordering keys tie.

When `storage.user.location = "workspace"`, the default user-scoped memory root is:

- `<workspace.storage.root>/user/memory/`

### Logging

- `logging.runtime.location`: optional string (default `"auto"`), one of:
    - `"auto"`: pick the first writable location in this order:
        1. OS default user logs location (for example `~/Library/Logs/alfred/` on macOS).
        2. `<user data>/alfred/logs/`.
        3. OS provided system log location (for example `/var/log/alfred/`) only if writable without elevation.
        4. `<workspace.storage.root>/logs/`.
    - `"workspace"`: force workspace logs.
    - `"user_logs"`: force OS user logs.
    - `"user_data"`: force user data logs.
    - `"system_logs"`: force system logs (only if writable without elevation).
- `logging.runtime.path`: optional path override; if set, overrides `logging.runtime.location`.
- `logging.runtime.retention_days`: integer (default `7`).

At server startup, Alfred MUST create a new runtime log file using a stable, time-sortable filename (UTC timestamp to second precision; RFC3339-like without `:`), for example `alfred-20260228T134512Z.ndjson`.

Rotated logs MUST be archived as ZIP files and pruned according to `logging.runtime.retention_days`.

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
