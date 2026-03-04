# Alfred Defaults Reference

This document is the single source of truth for every default value in Alfred.

Other documents MAY cite a default value inline for readability, but MUST reference this document as the authoritative source. When any default changes, update this document first and follow up in cited documents.

## Configuration defaults

### Storage

| Key                          | Default               | Notes                                                    |
| ---------------------------- | --------------------- | -------------------------------------------------------- |
| `storage.user.location`      | `<user data>/alfred/` | First writable user data dir, see location search below. |
| `storage.workspace.location` | `"workspace"`         | Workspace-rooted storage.                                |
| `workspace.storage.root`     | `.alfred/`            | Relative to workspace root.                              |

#### Storage location search

See [`StorageLayout.md`](./StorageLayout.md) for the full file and folder reference.

Alfred resolves three root storage locations at startup. `alfred/` is always appended to whichever base is selected.

**User configuration folder** — searches in priority order, selects the first found:

1. The user's config folder (e.g. `~/.config/` on Linux).
2. The user's data folder.
3. An OS config folder, if available.
4. The workspace folder.

Contains: `config.json`

**User data folder** — searches in priority order, selects the first writable location:

1. The user's data folder (e.g. `~/.local/share/` on Linux, `~/Library/Application Support/` on macOS).
2. An OS data folder, if available.
3. The workspace folder.

Contains: `index/` and `memory/` subfolders (workspace-independent persistence).

**Workspace folder** — `.alfred/` relative to the workspace root unless overridden by `workspace.storage.root`.

Contains:

- `index/` — workspace index persistence.
- `memory/` — workspace memory persistence.
- `config.json` — workspace configuration.
- `user/` (optional) — per-user overrides stored in the workspace:
    - `config.json`
    - `index/`
    - `memory/`

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
| `logging.location`       | `<user data>/alfred/logs/` |       |
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
| Path separators | POSIX `/`                                               |
| Path scope      | Workspace-relative (unless a contract states otherwise) |
| Absolute paths  | Rejected unless specifically permitted by the contract  |
