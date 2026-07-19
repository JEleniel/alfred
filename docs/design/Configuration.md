# Alfred Configuration

This document specifies Alfred's configuration model: policy-controlled configuration levels, precedence, merge rules, and configuration keys.

## Parent Aurora card

[CNS-010](./aurora/MIS-001/Constraint/CNS-010-Permission_Model_and_Guardrails.json) — this document elaborates the permission-model and policy-guardrail constraint by defining how configuration is sourced, merged, and applied.

## Configuration levels and precedence

Alfred reads configuration from two sources:

- User configuration: applies to all workspaces on a machine.
- Workspace configuration: applies only within a single workspace.

Workspace configuration MUST override user configuration.

Default values referenced by this document are defined in [`docs/design/Default.md`](./Default.md).

## Default locations

For a complete reference of every file and folder Alfred reads or writes, see [`StorageLayout.md`](./StorageLayout.md).

Configuration file locations are implementation-defined. Default locations SHOULD follow a simple rule:

- User-scoped configuration lives under the user configuration folder.
- User-scoped durable data lives under the user data folder.
- User-scoped temporary data lives under the user cache folder.
- User-scoped runtime logs live under the user log folder.
- Workspace-scoped artifacts live under the workspace root only when their location is `workspace`.

To avoid repeating full paths throughout this document, default locations are consolidated here.

- User configuration folder: `alfred/`
    - `config.json`
    - `.alfredignore`
    - Optional per-workspace workspace-configuration files when `storage.workspace.location = "user"`:
        - `<workspace_id>/config.json`
        - `<workspace_id>/.alfredignore`
- User data folder: `alfred/`
    - User-scoped durable data such as `index/` and `memory/`
    - Optional per-workspace durable data when `storage.workspace.location = "user"`:
        - `<workspace_id>/index/`
        - `<workspace_id>/memory/`
- User cache folder: `alfred/`
    - User-scoped temporary Alfred data
    - Optional per-workspace temporary data when `storage.workspace.location = "user"`
- User log folder: `alfred/`
    - Runtime log files
    - Optional per-workspace log files when `storage.workspace.location = "user"`
- Workspace root:
    - `.alfredignore`
    - Workspace storage root: `<workspace.storage.root>/` (default `.alfred/`)
        - `data/`
        - Optional `config/`, `cache/`, and `logs/` folders when those workspace-scoped artifact classes are enabled

## Merge rules

Configuration is merged deterministically.

- Objects: deep-merge (workspace overrides user).
- Arrays:
    - `tools.disabled`: union of user + workspace values, de-duplicated, stable-sorted.
    - `fs.disabled_operations`: union of user + workspace values, de-duplicated, stable-sorted.
    - Other arrays: workspace overrides user unless otherwise specified.

## Schema

The workspace and user configuration format is JSON and MUST conform to `schemas/alfred.config.schema.json`.

## Keys

### Tool enablement

- `tools.disabled`: array of tool names to disable.

Semantics:

- Disabled tools MUST be omitted from `capabilities`.
- Calls to disabled tools MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).

### `fs` operation gating

- `fs.disabled_operations`: array of `fs` operation selectors to disable.

Supported selectors:

- `delete_file`
- `delete_dir`
- `bulk.delete` — applies to any `fs` bulk item with `kind: "delete"`.

Semantics:

- Disabled `fs` operations MUST fail deterministically (see `ErrorTaxonomy.md`).
- Default delete-operation guardrails are defined in `Default.md`.

### Storage location controls

Alfred supports two location values for storage:

- `user`: store the artifact in the matching user folder for its purpose.
- `workspace`: store the artifact under the workspace storage root.

Keys:

- `storage.user.location`: optional string (default `"user"`), one of:
    - `"user"`: use the user configuration, data, cache, and log folders.
    - `"workspace"`: store user-scoped config/data under `<workspace.storage.root>/user/`.
- `storage.workspace.location`: optional string (default `"workspace"`), one of:
    - `"workspace"`: store workspace-scoped config/data under `<workspace.storage.root>/`.
    - `"user"`: store workspace-scoped artifacts under per-workspace user-folder subdirectories keyed by workspace id.

Notes:

- The configuration precedence rules do not change; only the persistence locations change.
- User-folder storage MUST preserve separation by purpose: configuration, durable data, temporary data, and logs.
- `workspace.storage.root` and `storage.workspace.location = "user"` are mutually exclusive for workspace-scoped artifacts, except for the local workspace identity token.

### Workspace Root Selection

Alfred selects `workspaceRoot` in this order:

1. A workspace root provided by the hosting environment.
2. A user-specified root when the host does not provide one.
3. A source-control marker fallback such as `.git/` when neither of the above is available.

If none of these produce a root, Alfred MUST fail deterministically and require an explicit workspace root.

User-specified roots MUST be verified for read and write access before acceptance. If a user-specified root resolves outside the user's home-directory tree, Alfred MUST emit a warning indicating that the boundary is outside the normal home-scoped workspace area.

### Path resolution (deterministic defaults)

This section summarizes how Alfred resolves default paths when optional overrides are not provided.

Terminology:

- `workspaceRoot`: the workspace boundary root.
- `<workspace_id>`: a stable, unique identifier for this workspace. Alfred derives it from the workspace boundary root, not from `<workspace.storage.root>`.
- `<workspace.storage.root>`: workspace storage root folder under `workspaceRoot` (default `.alfred/`).
- `<user config>`, `<user data>`, `<user cache>`, and `<user logs>`: user-level locations for configuration, durable data, temporary data, and logs.

Workspace identity uses this derivation chain:

1. Primary: a generated UUID token persisted in workspace-local Alfred storage.
2. Secondary: SHA-256 of the user-home-relative path to `workspaceRoot` when that relative path exists.
3. Fallback: SHA-256 of the absolute path to `workspaceRoot`.

Alfred SHOULD record the workspace root and all derived hashes in a debug index under `<user data>/alfred/` so users can inspect workspace-to-hash mappings.

Default layout (`storage.user.location = "user"`, `storage.workspace.location = "workspace"`):

- User config: `<user config>/alfred/config.json`.
- User durable data: `<user data>/alfred/`.
- User temporary data: `<user cache>/alfred/`.
- Runtime logs: `<user logs>/alfred/`.
- Workspace durable data: `workspaceRoot/<workspace.storage.root>/data/`.
- Workspace config, cache, and logs are absent by default and are created only when those workspace-scoped artifact classes are enabled.

Workspace-relocated layout (`storage.workspace.location = "user"`):

- Workspace config: `<user config>/alfred/<workspace_id>/config.json`.
- Workspace durable data: `<user data>/alfred/<workspace_id>/`.
- Workspace identity debug index: `<user data>/alfred/workspace-index.json`.
- Workspace temporary data: `<user cache>/alfred/<workspace_id>/`.
- Workspace logs: `<user logs>/alfred/<workspace_id>/`.

Workspace-local user profile (`storage.user.location = "workspace"`):

- User config: `workspaceRoot/<workspace.storage.root>/user/config.json`.
- User memory: `workspaceRoot/<workspace.storage.root>/user/memory/`.

### Workspace storage root

- `workspace.storage.root`: workspace-relative folder used for Alfred workspace-owned artifacts.
    - Default: `.alfred/`.

The resolved root is used by default for workspace-scoped persistence (index, workspace memory, and workspace logs) when `storage.workspace.location = "workspace"`.

When `storage.workspace.location = "user"`, `<workspace.storage.root>` does not apply to workspace-scoped artifacts.

The workspace-local identity token remains under `<workspace.storage.root>/data/` even when `storage.workspace.location = "user"`.

### Indexing

- `index.enabled`: boolean (default `true`).
    - When `false`, index-backed tools MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).
- `index.persistence.enabled`: boolean (default `true`).
- `index.persistence.location`: `"workspace" | "user"` (default `"workspace"`).
- `index.persistence.path`: string (optional override).
    - When unset, uses the default location for the selected persistence location (see [Default locations](#default-locations)).
- `index.persistence.interval_seconds`: integer (default `30`).

When `storage.workspace.location = "user"` and `index.persistence.location = "workspace"`, the default workspace index root is mirrored into the per-workspace user data path:

- `<user data>/alfred/<workspace_id>/index/`

Index ignore rules are driven by `.alfredignore` files (gitignore-style syntax) rather than JSON configuration.

Ignore layers (in order):

- Always excluded patterns: MUST always be ignored and MUST NOT be overridden.
- Built-in default `.alfredignore` patterns: applied by default, but MAY be overridden by whitelist rules.
- User `.alfredignore`: applied before workspace rules.
- Workspace `.alfredignore`: applied additively from workspace root down to subfolders.

The default lists are defined in [`docs/design/Default/IncludeExcludeList.md`](./Default/IncludeExcludeList.md).

Single-location rule:

- For a given workspace, Alfred MUST persist the index in exactly one location selected by `index.persistence.location`.
- If `index.persistence.location = "workspace"`, Alfred MUST NOT also maintain a duplicate workspace index in user data for that workspace.

### Memory storage

- `memory.storage.user.enabled`: boolean (default `true`).
- `memory.storage.workspace.enabled`: boolean (default `true`).
- `memory.storage.workspace.path`: path (optional override).
- `memory.storage.user.path`: path (optional override).
- `memory.storage.merge_mode`: `"union_workspace_wins" | "prefer_workspace_with_user_preferences_fallback"`.
    - Default: `"union_workspace_wins"`.

Notes:

- Enabling workspace memory storage provides per-workspace persistence under the configurable workspace storage root.
- Effective read behavior for user+workspace stores is defined in [`docs/design/ToolApiDefinition.md`](./ToolApiDefinition.md).

Merge-mode semantics (effective reads):

> **Same-id behavior**: When the same `id` (UUID) exists in more than one enabled scope, Alfred MUST return all instances, each with its `scope` field set, regardless of merge_mode. Because IDs are UUIDs issued by Alfred, this SHOULD NOT occur in normal operation but MAY occur in migrated or shared stores.

- `union_workspace_wins`:
    - Alfred MUST treat the effective memory corpus as the union of enabled stores.
    - When ordering results with equal relevance scores, workspace-scoped facts SHOULD appear before user-scoped facts.
- `prefer_workspace_with_user_preferences_fallback`:
    - When listing/searching, Alfred SHOULD order workspace facts ahead of user facts when other ordering keys tie.

When `storage.user.location = "workspace"`, the default user-scoped memory root is:

- `<workspace.storage.root>/user/memory/`

### Logging

- `logging.path`: optional path override; when unset, runtime logs default to `<user logs>/alfred/`.
- `logging.retention_days`: integer (default `7`).

At server startup, Alfred MUST create a new runtime log file using a stable, time-sortable filename (UTC timestamp to second precision; RFC3339-like without `:`), for example `alfred-20260228T134512Z.json`.

Rotated logs MUST be archived as ZIP files and pruned according to `logging.retention_days`.

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
