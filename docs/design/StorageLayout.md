# Alfred Storage Layout

This document is the canonical reference for every file and folder Alfred reads or writes.
For configuration semantics and precedence rules, see [`Configuration.md`](./Configuration.md).
For default values, see [`Defaults.md`](./Defaults.md).

## Overview

Alfred uses three root storage locations. `alfred/` is appended to whichever base directory is selected for each location.

| Location                  | Purpose                                        |
| ------------------------- | ---------------------------------------------- |
| User configuration folder | User-scoped configuration and ignore rules     |
| User data folder          | User-scoped persistence (index, memory, logs)  |
| Workspace folder          | Workspace-scoped persistence and configuration |

The resolved base for each location depends on the host OS and any overrides in configuration. See [Location resolution](#location-resolution) below.

---

## User configuration folder

**Resolution** — the first directory that exists is selected, in priority order:

1. OS user config directory (e.g. `~/.config/` on Linux, `~/Library/Preferences/` on macOS, `%APPDATA%\` on Windows).
2. OS user data directory.
3. OS config directory (platform-specific fallback).
4. Workspace folder.

**Contents under `alfred/`:**

| Path                           | Description                                                                       |
| ------------------------------ | --------------------------------------------------------------------------------- |
| `config.json`                  | User-scoped configuration. Validated against `schemas/alfred.config.schema.json`. |
| `.alfredignore`                | User-scoped ignore rules (gitignore syntax). Applied before workspace rules.      |
| `<workspace_id>/config.json`   | Per-workspace configuration override, when `storage.workspace.location = "user"`. |
| `<workspace_id>/.alfredignore` | Per-workspace ignore rules, when `storage.workspace.location = "user"`.           |

`<workspace_id>` is the SHA-256 hex hash of the absolute workspace root path. See [`Configuration.md`](./Configuration.md) for derivation details.

---

## User data folder

**Resolution** — the first writable directory is selected, in priority order:

1. OS user data directory (e.g. `~/.local/share/` on Linux, `~/Library/Application Support/` on macOS, `%LOCALAPPDATA%\` on Windows).
2. OS data directory (platform-specific fallback).
3. Workspace folder.

**Contents under `alfred/`:**

| Path                          | Description                                                               |
| ----------------------------- | ------------------------------------------------------------------------- |
| `index/`                      | User-scoped index persistence files.                                      |
| `memory/`                     | User-scoped memory facts.                                                 |
| `logs/`                       | Runtime log files (default location). See [Log files](#log-files).        |
| `<workspace_id>/config.json`  | Workspace configuration, when `storage.workspace.location = "user"`.      |
| `<workspace_id>/data/index/`  | Workspace index persistence, when `storage.workspace.location = "user"`.  |
| `<workspace_id>/data/memory/` | Workspace memory persistence, when `storage.workspace.location = "user"`. |
| `<workspace_id>/data/logs/`   | Workspace runtime logs, when `storage.workspace.location = "user"`.       |

---

## Workspace folder

The workspace folder is rooted at `<workspace.storage.root>` relative to the workspace root (default `.alfred/`).

**Contents:**

| Path                 | Description                                                                                 |
| -------------------- | ------------------------------------------------------------------------------------------- |
| `config.json`        | Workspace configuration. Validated against `schemas/alfred.config.schema.json`.             |
| `index/`             | Workspace index persistence files.                                                          |
| `memory/`            | Workspace-scoped memory facts (when `memory.storage.workspace.enabled = true`).             |
| `logs/`              | Workspace-local runtime logs (when `logging.location` is not overridden).                   |
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

The default built-in patterns are defined in [`DefaultIncludeExcludeList.md`](./DefaultIncludeExcludeList.md).

---

## Log files

Runtime logs are written to the resolved `logging.location` directory (default `<user data>/alfred/logs/`).

| Pattern                      | Description                                                                                                            |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `alfred-<timestamp>Z.json` | Active log file. Timestamp is UTC, RFC3339-like without `:`, second precision (e.g. `alfred-20260228T134512Z.json`). |
| `alfred-<timestamp>Z.zip`    | Archived (rotated) log. Older than the active file; pruned after `logging.retention_days` days (default `7`).          |

A new log file is created on each server startup. Previous files are archived as ZIP and pruned per retention policy.

---

## Layout variants

Alfred supports three storage layouts, controlled by `storage.user.location` and `storage.workspace.location`.

### Default layout

`storage.user.location = "os"` (default), `storage.workspace.location = "workspace"` (default).

| Artifact               | Resolved path                         |
| ---------------------- | ------------------------------------- |
| User config            | `<user config>/alfred/config.json`    |
| User ignore rules      | `<user config>/alfred/.alfredignore`  |
| User index             | `<user data>/alfred/index/`           |
| User memory            | `<user data>/alfred/memory/`          |
| Runtime logs           | `<user data>/alfred/logs/`            |
| Workspace config       | `<workspaceRoot>/.alfred/config.json` |
| Workspace index        | `<workspaceRoot>/.alfred/index/`      |
| Workspace memory       | `<workspaceRoot>/.alfred/memory/`     |
| Workspace ignore rules | `<workspaceRoot>/.alfredignore`       |

### Workspace-relocated layout

`storage.workspace.location = "user"` — workspace artifacts are stored in OS user directories, keyed by `<workspace_id>`.

| Artifact               | Resolved path                                       |
| ---------------------- | --------------------------------------------------- |
| Workspace config       | `<user config>/alfred/<workspace_id>/config.json`   |
| Workspace ignore rules | `<user config>/alfred/<workspace_id>/.alfredignore` |
| Workspace index        | `<user data>/alfred/<workspace_id>/data/index/`     |
| Workspace memory       | `<user data>/alfred/<workspace_id>/data/memory/`    |
| Workspace logs         | `<user data>/alfred/<workspace_id>/data/logs/`      |

Use this layout to avoid writing Alfred artifacts into the workspace (for example in read-only or shared repositories).

### Workspace-local user profile

`storage.user.location = "workspace"` — user-scoped artifacts are stored under the workspace storage root.

| Artifact          | Resolved path                                |
| ----------------- | -------------------------------------------- |
| User config       | `<workspaceRoot>/.alfred/user/config.json`   |
| User ignore rules | `<workspaceRoot>/.alfred/user/.alfredignore` |
| User index        | `<workspaceRoot>/.alfred/user/index/`        |
| User memory       | `<workspaceRoot>/.alfred/user/memory/`       |

Use this layout when OS user directories should not be read or written (for example in sandboxed or ephemeral environments).
