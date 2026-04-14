# Storage Layout

This document provides the narrative detail for Alfred's application-managed on-disk layout and file locations.

## Parent Aurora card

[APP-001](./aurora/MIS-001/Application/APP-001-Alfred_Stdio_Server.json) — this document elaborates how the Alfred application organizes its local stores, logs, and workspace-owned files on disk.

This document is the canonical reference for every file and folder Alfred reads or writes.
For configuration semantics and precedence rules, see [`Configuration.md`](./Configuration.md).
For default values, see [`Default.md`](./Default.md).

## Overview

Alfred uses purpose-specific user folders plus an optional workspace storage root. `alfred/` is appended to each user-level base directory, and `<workspace.storage.root>/` is used inside the workspace when workspace-local storage is enabled.

| Location                  | Purpose                                                     |
| ------------------------- | ----------------------------------------------------------- |
| User configuration folder | User-scoped configuration and ignore rules                  |
| User data folder          | User-scoped durable data and optional relocated workspace data |
| User cache folder         | User-scoped temporary data and optional relocated workspace cache |
| User log folder           | Runtime logs and optional relocated workspace logs          |
| Workspace folder          | Workspace-scoped artifacts stored inside the workspace      |

The resolved base for each location depends on the host OS and any overrides in configuration.

---

## User configuration folder

**Resolution** — the host user configuration directory.

**Contents under `alfred/`:**

| Path                           | Description                                                                       |
| ------------------------------ | --------------------------------------------------------------------------------- |
| `config.json`                  | User-scoped configuration. Validated against `schemas/alfred.config.schema.json`. |
| `.alfredignore`                | User-scoped ignore rules (gitignore syntax). Applied before workspace rules.      |
| `<workspace_id>/config.json`   | Per-workspace configuration override, when `storage.workspace.location = "user"`. |
| `<workspace_id>/.alfredignore` | Per-workspace ignore rules, when `storage.workspace.location = "user"`.           |

`<workspace_id>` follows the identity derivation rules in [`Configuration.md`](./Configuration.md).

---

## User data folder

**Resolution** — the host user data directory.

**Contents under `alfred/`:**

| Path                          | Description                                                               |
| ----------------------------- | ------------------------------------------------------------------------- |
| `index/`                      | User-scoped index persistence files.                                      |
| `memory/`                     | User-scoped memory facts.                                                 |
| `workspace-index.json`        | Debug index of workspace roots and derived workspace-identity hashes.     |
| `<workspace_id>/index/`       | Workspace index persistence, when `storage.workspace.location = "user"`.  |
| `<workspace_id>/memory/`      | Workspace memory persistence, when `storage.workspace.location = "user"`. |

---

## User cache folder

**Resolution** — the user cache directory for the host OS.

**Contents under `alfred/`:**

| Path             | Description                                                          |
| ---------------- | -------------------------------------------------------------------- |
| `<workspace_id>/` | Workspace-scoped temporary data, when `storage.workspace.location = "user"`. |

---

## User log folder

**Resolution** — the user log directory for the host OS.

**Contents under `alfred/`:**

| Path             | Description                                                     |
| ---------------- | --------------------------------------------------------------- |
| `alfred-*.json`  | Runtime log files for Alfred. See [Log files](#log-files).      |
| `<workspace_id>/` | Workspace runtime logs, when `storage.workspace.location = "user"`. |

---

## Workspace folder

The workspace folder is rooted at `<workspace.storage.root>` relative to the workspace root (default `.alfred/`).

**Contents:**

| Path                 | Description                                                                                 |
| -------------------- | ------------------------------------------------------------------------------------------- |
| `data/`              | Workspace-scoped durable data such as index persistence and workspace memory.               |
| `data/workspace.json` | Workspace-local identity token and related workspace identity metadata.                     |
| `config/`            | Workspace configuration when workspace-local configuration is enabled.                       |
| `cache/`             | Workspace-scoped temporary data when workspace-local cache is enabled.                       |
| `logs/`              | Workspace-local runtime logs when workspace-local logs are enabled.                          |
| `user/`              | Optional user profile stored in the workspace (when `storage.user.location = "workspace"`). |
| `user/config.json`   | User configuration stored in-workspace.                                                     |
| `user/.alfredignore` | User ignore rules stored in-workspace.                                                      |
| `user/index/`        | User index persistence stored in-workspace.                                                 |
| `user/memory/`       | User memory stored in-workspace.                                                            |

The `.alfred/` folder itself is in the built-in default exclude list and is never indexed.

---

## Workspace root — ignore files

`.alfredignore` files use gitignore syntax and may appear at any folder level within the workspace. Alfred merges ignore layers in a defined priority order:

| Source                             | Applied                                                        |
| ---------------------------------- | -------------------------------------------------------------- |
| Always-excluded patterns           | Hardcoded; cannot be overridden.                               |
| Built-in default patterns          | Applied by default; MAY be overridden by explicit allow rules. |
| User `.alfredignore`               | User config folder; applied before workspace rules.            |
| Workspace root `.alfredignore`     | Applied after user rules.                                      |
| Subdirectory `.alfredignore` files | Applied additively from root down to each subdirectory.        |

The default built-in patterns are defined in [`Default/IncludeExcludeList.md`](./Default/IncludeExcludeList.md).

---

## Log files

Runtime logs are written to the resolved `logging.path` directory when configured, or to the default user log folder at `<user logs>/alfred/`.

| Pattern                      | Description                                                                                                            |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `alfred-<timestamp>Z.json` | Active log file. Timestamp is UTC, RFC3339-like without `:`, second precision (e.g. `alfred-20260228T134512Z.json`). |
| `alfred-<timestamp>Z.zip`    | Archived (rotated) log. Older than the active file; pruned after `logging.retention_days` days (default `7`).          |

A new log file is created on each server startup. Previous files are archived as ZIP and pruned per retention policy.

---

## Layout variants

Alfred supports three storage layouts, controlled by `storage.user.location` and `storage.workspace.location`.

### Default layout

`storage.user.location = "user"` (default), `storage.workspace.location = "workspace"` (default).

| Artifact               | Resolved path                         |
| ---------------------- | ------------------------------------- |
| User config            | `<user config>/alfred/config.json`         |
| User ignore rules      | `<user config>/alfred/.alfredignore`       |
| User index             | `<user data>/alfred/index/`                |
| User memory            | `<user data>/alfred/memory/`               |
| User temporary data    | `<user cache>/alfred/`                     |
| Runtime logs           | `<user logs>/alfred/`                      |
| Workspace identity     | `<workspaceRoot>/.alfred/data/workspace.json` |
| Workspace data         | `<workspaceRoot>/.alfred/data/`            |
| Workspace config       | `<workspaceRoot>/.alfred/config/` (optional) |
| Workspace logs         | `<workspaceRoot>/.alfred/logs/` (optional) |
| Workspace ignore rules | `<workspaceRoot>/.alfredignore`       |

### Workspace-relocated layout

`storage.workspace.location = "user"` — workspace artifacts are stored in the matching user folders, keyed by `<workspace_id>`.

| Artifact               | Resolved path                                       |
| ---------------------- | --------------------------------------------------- |
| Workspace config       | `<user config>/alfred/<workspace_id>/config.json`   |
| Workspace ignore rules | `<user config>/alfred/<workspace_id>/.alfredignore` |
| Workspace identity     | `<workspaceRoot>/.alfred/data/workspace.json`       |
| Workspace index        | `<user data>/alfred/<workspace_id>/index/`          |
| Workspace memory       | `<user data>/alfred/<workspace_id>/memory/`         |
| Workspace cache        | `<user cache>/alfred/<workspace_id>/`               |
| Workspace logs         | `<user logs>/alfred/<workspace_id>/`                |

Use this layout to avoid writing Alfred artifacts into the workspace (for example in read-only or shared repositories).

### Workspace-local user profile

`storage.user.location = "workspace"` — user-scoped artifacts are stored under the workspace storage root.

| Artifact          | Resolved path                                |
| ----------------- | -------------------------------------------- |
| User config       | `<workspaceRoot>/.alfred/user/config.json`   |
| User ignore rules | `<workspaceRoot>/.alfred/user/.alfredignore` |
| User index        | `<workspaceRoot>/.alfred/user/index/`        |
| User memory       | `<workspaceRoot>/.alfred/user/memory/`       |
| User cache        | `<workspaceRoot>/.alfred/user/cache/`        |
| User logs         | `<workspaceRoot>/.alfred/user/logs/`         |

Use this layout when user-scoped artifacts should be stored in the workspace instead of the user folders (for example in sandboxed or ephemeral environments).
