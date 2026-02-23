# Alfred Tool Contracts

This document describes Alfred's architecture-level tool surface at an implementation-ready level.

Unless otherwise specified:

- Tools are executed relative to the workspace root.
- Read-only operations are side-effect free.
- Mutating operations support `dry_run` where applicable.
- Paged operations default `limit` to `100`.
- Line and column numbers are 1-indexed.

## Configuration and policy

Alfred behavior is configurable at two levels:

- User configuration: applies to all workspaces on a machine.
- Workspace configuration: applies only within a single workspace.

Workspace configuration MUST override user configuration.

Default config locations:

- User: OS config directory `alfred/config.json`.
- Workspace: `<workspaceRoot>/.alfred/config.json`.

Configuration keys and defaults are defined in [`docs/design/Configuration.md`](./Configuration.md).

### Tool enablement

Any tool MAY be disabled by configuration.

- Disabled tools MUST be omitted from `capabilities`.
- Calls to disabled tools MUST fail with `error.kind: "invalid_argument"` and `retryable: false`.

## Redaction (non-public information)

Alfred MUST filter non-public information (NPI) from:

- Tool responses.
- Tool/runtime logs.
- Search indexes and memory indexes at ingestion time.

NPI includes (at minimum) secret-looking values (tokens/keys/passwords) and MAY include user-configured PII/PHI-like patterns.

Redaction is deterministic and uses a stable replacement token (default `<-REDACTED->`).

The deterministic redaction algorithm (detection + replacement-length fitting) is specified in [`docs/design/Redaction.md`](./Redaction.md).

Tools SHOULD surface redaction as a warning in result metadata rather than failing the call.

## Result envelope (all tools)

All tools MUST return results using the envelope described in [`docs/design/Protocol.md`](./Protocol.md).

In this document, each tool section's **Output** describes the envelope `data` payload unless explicitly stated otherwise.

Unless otherwise specified, each tool call is synchronous.

Only `bulk_fs_operations` MAY execute in background mode.

When `bulk_fs_operations` runs in background mode, the initial call MUST return `status: "pending"` with operation metadata embedded in `data`:

- `operation_id`: string.
- `state`: `"queued" | "running"`.
- `poll_with`: fixed value `"bulk_fs_operations"`.

No standalone job-control tool surface exists.

## Effective top-level tool surface

The architecture intentionally consolidates the public command set to reduce tool-count pressure:

- `workspace_dir`
- `search`
- `fs_operations`
- `bulk_fs_operations`
- `patch`
- `log_operations`
- `plan_operations`
- `memory`
- `capabilities`

## Context tool

### `workspace_dir`

- Purpose: return Alfred's workspace root folder.
- Execution: synchronous.
- Input: none.
- Output:
    - `root`: normalized workspace-root absolute path.

## Search tool

### `search`

- Purpose: deterministic workspace text search (literal and regex modes).
- Execution: synchronous.
- Input:
    - `query`: string.
    - `mode`: optional `"literal" | "regex"` (default `"literal"`).
    - `case_sensitive`: optional boolean (default `false`).
    - `full_text`: optional boolean (default `false`).
    - `include_pattern`: optional string (glob applied to workspace-relative paths).
    - `exclude_pattern`: optional string (glob applied to workspace-relative paths).
    - `cursor`: optional string.
    - `limit`: optional integer.
- Output:
    - `matches`: array of objects:
        - `path`: string.
        - `line`: integer (1-indexed).
        - `text`: string.
    - `next_cursor`: optional string.

Notes:

- This tool is index-backed.
- If the workspace index is not available, the tool MUST fail with:
    - `error.kind: "tool_unavailable"`
    - `retryable: true`
    - `details.reason: "index_not_ready"`

## File and directory operations tool

### `fs_operations`

- Purpose: perform all non-bulk filesystem operations through a single deterministic interface.
- Execution: synchronous.
- Input:
    - `operation`: one of:
        - `"list"`
        - `"read_range"`
        - `"stat"`
        - `"diff"`
        - `"create_file"`
        - `"append_file"`
        - `"delete_file"`
        - `"create_dir"`
        - `"delete_dir"`
    - `args`: object whose shape depends on `operation`.
    - `dry_run`: optional boolean (required for mutating operations; default `true`).
- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific result object.

Supported operation contracts:

- `list`
    - Input args:
        - `path`: optional string (default `"."`).
        - `recursive`: optional boolean (default `false`).
        - `include_hidden`: optional boolean (default `false`).
        - `include_dirs`: optional boolean (default `true`).
        - `include_files`: optional boolean (default `true`).
        - `cursor`: optional string.
        - `limit`: optional integer.
    - Result:
        - `files`: array of strings.
        - `directories`: array of strings.
        - `next_cursor`: optional string.

- `read_range`
    - Input args:
        - `path`: string.
        - `start_line`: integer.
        - `end_line`: integer (inclusive; `>= start_line`).
    - Result:
        - `path`: string.
        - `start_line`: integer.
        - `end_line`: integer.
        - `text`: string.
    - Note: MUST fail deterministically for non-text/binary inputs.

- `stat`
    - Input args:
        - `path`: string.
    - Result:
        - `path`: string.
        - `kind`: `"file" | "dir" | "symlink" | "other"`.
        - `size_bytes`: optional integer.
        - `modified_at`: optional string (RFC3339 UTC).

- `diff`
    - Input args:
        - `a`: one of:
            - object:
                - `path`: string.
                - `from`: integer.
                - `to`: integer.
            - string.
        - `b`: one of:
            - object:
                - `path`: string.
                - `from`: integer.
                - `to`: integer.
            - string.
    - Result:
        - `diff`: string (unified diff format).

- `create_file`
    - Input args:
        - `path`: string.
        - `content`: string.
    - Result:
        - `path`: string.
        - `bytes_written`: integer.

- `append_file`
    - Input args:
        - `path`: string.
        - `content`: string.
    - Result:
        - `path`: string.
        - `bytes_written`: integer.

- `delete_file`
    - Input args:
        - `path`: string.
    - Result:
        - `deleted`: boolean.

- `create_dir`
    - Input args:
        - `path`: string.
        - `parents`: optional boolean (default `false`).
    - Result:
        - `created`: boolean.

- `delete_dir`
    - Input args:
        - `path`: string.
    - Result:
        - `deleted`: boolean.

Rules:

- Byte-oriented file operations are out of scope for Alfred and MUST NOT be exposed.
- All paths MUST be workspace-relative, normalized, and use `/` separators.

## Bulk filesystem operations tool

### `bulk_fs_operations`

- Purpose: run deterministic bulk move/copy/delete operations, including optional background execution and built-in status retrieval.
- Execution: synchronous or background.
- Input:
    - `mode`: one of `"execute" | "status" | "cancel"`.
    - `operation_id`: required for `status` and `cancel`.
    - `run_in_background`: optional boolean (valid only with `mode: "execute"`, default `false`).
    - `operations`: required for `mode: "execute"`; array of:
        - move:
            - `kind`: `"move"`.
            - `from`: string.
            - `to`: string.
            - `overwrite`: optional boolean (default `false`).
            - `create_parents`: optional boolean (default `false`).
        - copy:
            - `kind`: `"copy"`.
            - `from`: string.
            - `to`: string.
            - `overwrite`: optional boolean (default `false`).
            - `create_parents`: optional boolean (default `false`).
        - delete:
            - `kind`: `"delete"`.
            - `path`: string.
            - `recursive`: optional boolean (default `false`).
    - `dry_run`: optional boolean (default `true`).
- Output:
    - `operation_id`: string.
    - `state`: `"queued" | "running" | "succeeded" | "failed" | "canceled" | "partial"`.
    - `summary`: object:
        - `total`: integer.
        - `completed`: integer.
        - `failed`: integer.
    - `items`: optional array of per-item results for completed operations.

Rules:

- This is the only tool family allowed to run in background mode.
- Status polling MUST be performed by calling this same tool with `mode: "status"`.
- There are no standalone job tools.

## Patch tool

### `patch`

- Purpose: apply one or more text patches deterministically.
- Execution: synchronous.
- Input:
    - `patches`: array of objects:
        - `path`: string.
        - `patch`: string (mpatch format).
    - `dry_run`: optional boolean (default `true`).
- Output:
    - `files`: array of objects:
        - `path`: string.
        - `patched`: boolean.
        - `bytes_written`: optional integer.
        - `conflicts`: optional array.
        - `warnings`: optional array of warning objects.

Duplicate-content safeguard:

- Alfred MUST evaluate each patch for duplicate-content risk before write.
- If applying a patch would duplicate existing content (for example, append a full-file copy to the end of the same file), Alfred MUST emit a warning object:
    - `kind`: `"duplicate_content_risk"`.
    - `path`: string.
    - `details`: optional object with deterministic detection metadata.
- Duplicate-content risk SHOULD NOT hard-fail by default; conflicts still follow normal conflict semantics.

## Log operations tool

### `log_operations`

- Purpose: provide deterministic log search and bounded tail access.
- Execution: synchronous.
- Input:
    - `operation`: one of `"search" | "tail"`.
    - `args`: object depending on operation.
- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific result object.

Supported operations:

- `search`
    - Input args:
        - `path`: optional string (defaults to Alfred runtime log).
        - `query`: string.
        - `level`: optional string.
        - `source_prefix`: optional string.
        - `cursor`: optional string.
        - `limit`: optional integer.
    - Result:
        - `matches`: array of structured log records.
        - `next_cursor`: optional string.

- `tail`
    - Input args:
        - `path`: optional string (defaults to Alfred runtime log).
        - `level_min`: optional `"TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"`.
        - `source_prefix`: optional string.
        - `cursor`: optional string.
        - `limit`: optional integer.
    - Result:
        - `records`: array of structured log records.
        - `next_cursor`: optional string.

## Plan operations tool

### `plan_operations`

- Purpose: read and mutate the workspace project plan through one command surface.
- Execution: synchronous.
- Input:
    - `operation`: one of `"get" | "add" | "edit" | "update_status" | "delete"`.
    - `args`: object depending on operation.
- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific payload.

Plan location and locking:

- Default plan selection:
    1. `docs/design/ProjectPlan.md` if present.
    2. `ProjectPlan.md` otherwise.
- Writes MUST be serialized using lock files under `<workspaceRoot>/.alfred/locks/`.

## Memory tool

### `memory`

- Purpose: local/offline memory CRUD and full-text retrieval through one command surface.
- Execution: synchronous (MAY support bounded async in future versions).
- Input:
    - `operation`: one of `"put" | "get" | "delete" | "list" | "search"`.
    - `args`: object depending on operation.
- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific payload.

Storage model:

- User store (default): persisted in user data directory.
- Optional workspace store: persisted under `<workspaceRoot>/.alfred/` by default, configurable via `workspace.storage.root`.
- When workspace memory storage is enabled, Alfred MAY compose effective results from user + workspace stores according to configured merge policy.

## Capability discovery

### `capabilities`

- Purpose: advertise supported tools, versions, execution modes, and limits.
- Execution: synchronous.
- Input: none.
- Output:
    - `tools`: array of objects, stable-sorted by `name`:
        - `name`: string.
        - `version`: string (SemVer).
        - `schema_version`: string (SemVer; lockstep with `version`).
        - `execution_modes`: array (`"sync" | "background"`).
        - `limits`: optional object.

Limits SHOULD include:

- `max_inline_utf8_bytes`: maximum inline payload size per call.
- `max_patch_files_per_call`: maximum number of files accepted by `patch`.
- `max_bulk_operations_per_call`: maximum operations accepted by `bulk_fs_operations`.
- `max_log_records_per_call`: maximum records returned by `log_operations`.

## Notes on limits and determinism

- All list/search tools MUST define a default `limit` (default `100`).
- Ordering MUST be stable and documented.
- Cursor tokens MUST be opaque and deterministic.
