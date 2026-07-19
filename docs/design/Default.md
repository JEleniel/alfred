# Default

This document records Alfred's application-level default values. The built-in include and exclude patterns are detailed separately in [`Default/IncludeExcludeList.md`](./Default/IncludeExcludeList.md).

## Parent Aurora card

[APP-001](./aurora/MIS-001/Application/APP-001-Alfred_Stdio_Server.json) — this document elaborates the default operating values of the Alfred application.

## Configuration defaults

### Storage

| Key                          | Default               | Notes                                                    |
| ---------------------------- | --------------------- | -------------------------------------------------------- |
| `storage.user.location`      | `"user"`              | Uses user config, data, cache, and log folders.          |
| `storage.workspace.location` | `"workspace"`         | Workspace-scoped artifacts live under the workspace root by default. |
| `workspace.storage.root`     | `.alfred/`            | Relative to workspace root.                              |

#### Storage location search

See [`StorageLayout.md`](./StorageLayout.md) for the full file and folder reference.

Alfred resolves purpose-specific user folders plus an optional workspace storage root at startup. `alfred/` is always appended to user-level base directories.

**User configuration folder** — uses the host user configuration directory.

Contains: `config.json`

**User data folder** — uses the host user data directory.

Contains: `index/` and `memory/` subfolders (workspace-independent persistence).

**User cache folder** — uses the host user cache directory.

Contains: temporary Alfred data.

**User log folder** — uses the host user log directory.

Contains: Alfred runtime logs.

**Workspace folder** — `.alfred/` relative to the workspace root unless overridden by `workspace.storage.root`.

Contains:

- `data/` — workspace-scoped durable data.
- `config/` — workspace configuration when enabled.
- `cache/` — workspace-scoped temporary data when enabled.
- `logs/` — workspace-local logs when enabled.
- `user/` (optional) — per-user overrides stored in the workspace:
    - `config.json`
    - `index/`
    - `memory/`
    - `cache/`
    - `logs/`

### Index

| Key                                  | Default       | Notes |
| ------------------------------------ | ------------- | ----- |
| `index.enabled`                      | `true`        |       |
| `index.persistence.enabled`          | `true`        |       |
| `index.persistence.location`         | `"workspace"` |       |
| `index.persistence.interval_seconds` | `30`          |       |

### Memory storage

| Key                                | Default                  | Notes |
| ---------------------------------- | ------------------------ | ----- |
| `memory.storage.user.enabled`      | `true`                   |       |
| `memory.storage.workspace.enabled` | `true`                   |       |
| `memory.storage.merge_mode`        | `"union_workspace_wins"` |       |

### Logging

| Key                      | Default                    | Notes |
| ------------------------ | -------------------------- | ----- |
| `logging.path`           | `<user logs>/alfred/`      |       |
| `logging.retention_days` | `7`                        |       |

### Redaction

| Key                           | Default        | Notes |
| ----------------------------- | -------------- | ----- |
| `redaction.enabled`           | `true`         |       |
| `redaction.replacement_token` | `<-REDACTED->` |       |
| `redaction.preserve_length`   | `true`         |       |

### Plan

| Key         | Default                                                          | Notes                                        |
| ----------- | ---------------------------------------------------------------- | -------------------------------------------- |
| `plan.path` | `docs/design/ProjectPlan.md` (if present), else `ProjectPlan.md` | Workspace-relative. Always workspace-scoped. |

### Tool enablement

| Key              | Default | Notes                                 |
| ---------------- | ------- | ------------------------------------- |
| `tools.disabled` | `[]`    | Empty — all tools enabled by default. |

### `fs` operation gating

| Key                      | Default                                           | Notes                                          |
| ------------------------ | ------------------------------------------------- | ---------------------------------------------- |
| `fs.disabled_operations` | `["bulk.delete", "delete_dir", "delete_file"]` | Delete operations are disabled until enabled. |

---

## Tool parameter defaults

### All mutating operations

| Parameter | Default |
| --------- | ------- |
| `dry_run` | `true`  |

### `search`

| Parameter        | Default                                             |
| ---------------- | --------------------------------------------------- |
| `mode`           | `"general"`                                         |
| `case_sensitive` | `false`                                             |
| `cursor`         | (none — start from beginning)                       |
| `limit`          | (none — return all results up to capability limits) |

### `fs`

#### `search` operation

| Parameter        | Default |
| ---------------- | ------- |
| `path`           | `"."`   |
| `recursive`      | `false` |
| `include_hidden` | `false` |
| `include_dirs`   | `true`  |
| `include_files`  | `true`  |
| `cursor`         | (none)  |
| `limit`          | (none)  |

#### `create_dir` operation

| Parameter | Default |
| --------- | ------- |
| `parents` | `false` |

#### `bulk` / `execute` mode — per-operation defaults

| Parameter                  | Default |
| -------------------------- | ------- |
| `run_in_background`        | `false` |
| move/copy `overwrite`      | `false` |
| move/copy `create_parents` | `false` |
| delete `recursive`         | `false` |

### `logs`

#### `follow` operation

| Parameter | Default            |
| --------- | ------------------ |
| `stop`    | `false`            |
| `path`    | Alfred runtime log |

### `memory`

#### `delete` operation

| Parameter | Default |
| --------- | ------- |
| `dry_run` | `true`  |

### `status`

| Parameter | Default |
| --------- | ------- |
| `verbose` | `false` |

---

## Capability limits

These values are defined as constants in `src/tools/capabilities.rs` and are canonical at runtime.

| Limit                          | Value             | Applies to        |
| ------------------------------ | ----------------- | ----------------- |
| `max_inline_utf8_bytes`        | 1,048,576 (1 MiB) | `patch`, `fs`     |
| `max_patch_files_per_call`     | 128               | `patch`           |
| `max_bulk_operations_per_call` | 256               | `fs` bulk execute |
| `max_log_records_per_call`     | 1,000             | `logs`            |

---

## Protocol defaults

### Log record timestamp precision

| Field       | Default                                               |
| ----------- | ----------------------------------------------------- |
| `timestamp` | RFC3339 UTC, seconds precision (milliseconds allowed) |

### Path representation

| Property        | Default                                                 |
| --------------- | ------------------------------------------------------- |
| Path separators | Host-OS-native separators after normalization           |
| Path scope      | Resolve within the workspace boundary unless a contract states otherwise |
| Absolute paths  | Accepted when host-OS-valid and they resolve in bounds  |
