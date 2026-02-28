# Alfred Tool Contracts

This document describes Alfred's architecture-level tool surface at an implementation-ready level.

Unless otherwise specified:

- Tools are executed relative to the workspace root.
    - For the purpose of tools such as `logs`, the alfred server data locations are considered a read-only part of the workspace.
- Read-only operations are side-effect free.
- Mutating operations support `dry_run` where applicable.
- Operations do not default to paging.
- Line and column numbers in a file are 1-indexed.

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
- Calls to disabled tools MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).

## Redaction (non-public information)

Alfred MUST filter non-public information (NPI) from:

- Tool responses.
- Tool/runtime logs.
- Search indexes and memory indexes at ingestion time.

NPI includes (at minimum) secret-looking values (tokens/keys/passwords) and MAY include user-configured PII/PHI-like patterns.

Redaction is deterministic and uses a stable replacement token (default `<-REDACTED->`).

The deterministic redaction algorithm (detection + replacement-length fitting) is specified in [`docs/design/Redaction.md`](./Redaction.md).

Tools SHOULD surface redaction as a warning in the tool result envelope (see [`docs/design/Protocol.md`](./Protocol.md)) rather than failing the call.

## Result envelope (all tools)

All tools MUST return results using the envelope described in [`docs/design/Protocol.md`](./Protocol.md).

In this document, each tool section's **Output** describes the envelope `data` payload unless explicitly stated otherwise.

No standalone job-control tool surface exists.

## Effective top-level tool surface

The architecture intentionally consolidates the public command set to reduce tool-count pressure:

- `workspace_dir`
- `search`
- `fs`
- `patch`
- `logs`
- `plan`
- `memory`
- `capabilities`
- `status`

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
- If the workspace index is not available, the tool MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).
- Result ordering:
    - `matches` MUST be stable-sorted lexicographically, case-insensitive by `path`, then by `line` ascending.

## File and directory operations tool

### `fs`

- Purpose: perform all non-bulk filesystem operations through a single deterministic interface; run deterministic bulk move/copy/delete operations, including optional background execution and built-in status retrieval.
- Execution: synchronous or background (bulk only).
- Input:
    - `operation`: one of:
        - `"search"`
        - `"read_range"`
        - `"stat"`
        - `"diff"`
        - `"create_file"`
        - `"append_file"`
        - `"delete_file"`
        - `"create_dir"`
        - `"delete_dir"`
        - `"bulk"`
    - `args`: object whose shape depends on `operation`.
    - `dry_run`: optional boolean (required for mutating operations; default `true`).
- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific result object.

- `search`
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
    - Result ordering:
        - `files` and `directories` MUST be stable-sorted lexicographically, case-insensitive by path.

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

- `bulk`
    - Input args:
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
    - Result:
        - `operation_id`: string, uuid.
        - `state`: `"queued" | "running" | "succeeded" | "failed" | "canceled" | "partial"`.
        - `summary`: object:
            - `total`: integer.
            - `completed`: integer.
            - `failed`: integer.
        - `items`: optional array of per-item results for completed operations.

Rules:

- Byte-oriented file operations are out of scope for Alfred and MUST NOT be exposed.
- All paths MUST be workspace-relative, normalized, and use `/` separators.
    - The alfred logs path is permitted for read-only operations.
- Status polling MUST be performed by calling this same tool with `operation: "bulk"` and `args.mode: "status"`.
- There are no standalone job tools.

## Patch tool

### `patch`

- Purpose: apply one or more text patches deterministically.
- Execution: synchronous.
- Input:
    - `operation`: one of:
        - `"patch"`
        - `"multi-patch"`
        - `"revert"`
    - `args`: object whose shape depends on `operation`.
    - `dry_run`: optional boolean (default `true`).

- `operation: "patch"`
    - Input args:
        - `path`: string.
        - `patch`: string (mpatch format).
    - Result:
        - `file`: object:
            - `path`: string.
            - `succeeded`: boolean.
            - `patch_id`: string (uuid) assigned to the applied patch.
            - `bytes_written`: optional integer.
            - `conflicts`: optional array of deterministic error objects.

- `operation: "multi-patch"`
    - Input args:
        - `patches`: array of objects:
            - `path`: string.
            - `patch`: string (mpatch format).
    - Result:
        - `files`: array of objects:
            - `path`: string.
            - `succeeded`: boolean.
            - `patch_id`: string (uuid) assigned to the applied patch.
            - `bytes_written`: optional integer.
            - `conflicts`: optional array of deterministic error objects.

- `operation: "revert"`
    - Input args:
        - `patch_ids`: array of string (uuid) values to revert.
    - Result:
        - `files`: array of objects:
            - `path`: string.
            - `succeeded`: boolean.
            - `patch_id`: string (uuid) of the reverted patch.
            - `bytes_written`: optional integer.
            - `conflicts`: optional array of deterministic error objects.

- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific result object.
    - `dry_run`: echoed dry-run flag.

Revert semantics:

- Every time `patch` applies patches (single or multi), each applied patch is issued a `patch_id` (uuid) and the reverse patch is stored under that `patch_id`.
    - Only the most recent patch batch is stored. Reversion-only calls do not update this.
- A reversion MUST apply the reverse patch using the same semantics as other patches.
- If the target file has changed since the patch was applied, Alfred MUST refuse to revert for that file and return a deterministic conflict error (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).
- After a successful non-dry-run revert for a given `patch_id`, Alfred MUST discard the stored revert state for that `patch_id`.

Duplicate-content safeguard:

- Alfred MUST evaluate each patch for duplicate-content before write.
- If applying a patch would duplicate existing content (for example, append a full-file copy to the end of the same file), Alfred MUST refuse to apply that patch as a hard failure and return a deterministic error with `details.reason: "duplicate_content"` (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).

## Status tool

### `status`

- Purpose: return a lightweight snapshot of Alfred runtime state (index readiness, memory usage, configured paths).
- Execution: synchronous.
- Input:
    - `verbose`: optional boolean (default `false`).
- Output:
    - `version`: string, version of the alfred server.
    - `total_memory`: integer, bytes.
    - `workspace_root`: string.
    - `paths`: object:
        - `plan`: string.
        - `alfred_logs`: string.
    - `index`: object:
        - `ready`: boolean.
        - `indexed_files`: integer.
        - `indexed_directories`: integer.
    - `memory`: object:
        - `ready`: boolean.
        - `indexed_memories`: integer.

Notes:

- `paths.alfred_logs` MAY be an absolute path outside the workspace when runtime logs are configured to live in OS user log locations.

## Log operations tool

### `logs`

- Purpose: provide deterministic log search and bounded tail access.
- Execution: synchronous or stream (follow only).
- Input:
    - `operation`: one of `"search" | "tail" | "follow"`.
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
- `follow`
    - Input args:
        - `stop`: optional boolean (default `false`).
        - `path`: optional string (defaults to Alfred runtime log).
        - `level_min`: optional `"TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"`.
        - `source_prefix`: optional string.
        - `tail`: integer, lines to return from the log initially.
    - Result:
        - `records`: array of structured log records.
        - `stopped`: optional boolean.

Path rules:

- If `args.path` is omitted, Alfred MUST use its configured runtime log path.
- If `args.path` is provided:
    - It MUST be a workspace-relative path.
    - The resolved path MUST remain within the workspace boundary.
    - Absolute paths MUST be rejected.
    - `..` traversal MUST be rejected.

Streaming notes:

- `follow` is a streaming operation when invoked in `stream` execution mode.
- Streamed output MUST use the normal tool result envelope and the normal `logs` output shape.
- Only one `follow` stream may be active at a time.
    - If a `follow` stream is already active, a new `follow` start request MUST fail deterministically with `details.reason: "stream_active"` (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).
    - A `follow` stop request (`stop: true`) MUST stop the active stream.
    - If a `follow` stop request is issued when no stream is active, Alfred MUST fail deterministically with `details.reason: "stream_not_active"` (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).

Behavioral notes:

- A `follow` start request returns the last `tail` lines first, then continues emitting `logs` results as new records are appended.
- A `follow` stop request SHOULD return `stopped: true` when it stops an active stream.

## Plan operations tool

### `plan`

- Purpose: read and track the workspace project plan.
- Execution: synchronous.
- Input:
    - `operation`: one of `"get" | "add" | "update_status" | "delete"`.
    - `args`: object depending on operation.
- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific payload.

Plan location and locking:

- Default plan selection:
    1. `docs/design/ProjectPlan.md` if present.
    2. `ProjectPlan.md` otherwise.
- Concurrent writers (multiple Alfred instances targeting the same workspace plan) are unsupported; last-write-wins behavior is acceptable.
- Editing of the plan should use the normal `fs` operations.

## Memory tool

### `memory`

- Purpose: local/offline memory CRUD and full-text retrieval through one command surface.
- Execution: synchronous (MAY support bounded async in future versions).
- Input:
    - `operation`: one of `"create" | "retrieve" | "update" | "delete" | "search"`.
    - `args`: object depending on operation.
- Output:
    - `operation`: echoed operation name.
    - `result`: operation-specific payload.

Memory scopes:

- `scope`: `"user" | "workspace"`.
    - `"user"`: global/user-scoped memory (preferences, conventions) shared across workspaces.
    - `"workspace"`: workspace-scoped memory specific to the current workspace.

Identity rules:

- A memory fact `id` MUST be a UUID issued by Alfred at creation time.
- For create, callers MUST omit `id` and Alfred MUST return the issued `id`.
- For update, callers MUST provide an existing `id`.

Effective read model:

- `retrieve` and `search` MUST treat memory as one contiguous logical corpus composed from all enabled scopes.
- Results MUST include the `scope` for each returned fact.

Merge policy:

- The effective corpus is derived from user + workspace stores using `memory.storage.merge_mode` (see [`docs/design/Configuration.md`](./Configuration.md)).
- If the same `id` is present in more than one enabled scope, Alfred MUST resolve the conflict deterministically.
    - Default: Return both facts with a different `scope`.

Memory fact shape:

- `id`: string (UUID), issued at create.
- `scope`: `"user" | "workspace"`.
- `subject`: string.
- `category`: string.
- `fact`: string.
- `reasoning`: optional string.
- `tags`: optional array of strings.
- `created_at`: string (RFC3339 UTC), automatically maintained.
- `updated_at`: string (RFC3339 UTC), automatically maintained.

Operation contracts:

- `create`
    - Input args:
        - `scope`: `"user" | "workspace"`.
        - `subject`: string.
        - `category`: string.
        - `fact`: string.
        - `reasoning`: optional string.
        - `tags`: optional array of strings.
    - Result:
        - `memory`: the newly created memory object

- `retrieve`
    - Input args:
        - `id`: string.
    - Result:
        - `memory`: memory fact.

- `update`
    - Input args:
        - `id`: string.
        - `scope`: `"user" | "workspace"`.
        - `subject`: string.
        - `category`: string.
        - `fact`: string.
        - `reasoning`: optional string.
        - `tags`: optional array of strings.
    - Result:
        - `memory`: updated memory object

- `delete`
    - Input args:
        - `id`: string.
        - `dry_run`: optional boolean (default `true`).
    - Result:
        - `deleted`: boolean.

- `search`
    - Input args:
        - `query`: optional string. When omitted, returns all facts matching the filters.
        - `cursor`: optional string.
        - `limit`: optional integer.
        - `subject`: optional string.
        - `category`: optional string.
        - `tags`: optional array of strings.
        - `tags_and`: optional boolean (default `false`).
    - Result:
        - `matches`: array of objects:
            - `fact`: memory fact.
            - `score`: integer.
        - `next_cursor`: optional string.

Storage model:

- User-scoped memory (default): persisted under `<user data>/alfred/memory/`.
- When `storage.user.location = "workspace"`, user-scoped memory is persisted under `<workspace.storage.root>/user/memory/`.
- Workspace-scoped memory (optional): persisted under `<workspace.storage.root>/memory/` when `storage.workspace.location = "workspace"`.
- When `storage.workspace.location = "user"`, workspace-scoped memory is persisted under `<user data>/alfred/<workspace_id>/data/memory/`.
- Alfred MUST NOT maintain more than one persisted copy of any active memory index for a given scope.

## Capability discovery

### `capabilities`

- Purpose: advertise supported tools, versions, execution modes, and limits.
- Execution: synchronous.
- Input: none.
- Output:
    - `tools`: array of objects, stable-sorted lexicographically, case-insensitive by `name`:
        - `name`: string.
        - `version`: string (SemVer).
        - `schema_version`: string (SemVer; lockstep with `version`).
        - `execution_modes`: array (`"sync" | "background" | "stream"`).
        - `limits`: optional object.

Execution modes:

- `sync`: completes within a single tool call and returns a finite payload.
- `background`: may return `status: "pending"` and MUST be polled via a follow-up tool call.
- `stream`: may emit a stream of results over time by emitting repeated, standard tool result envelopes (MCP-compliant).

Limits SHOULD include:

- `max_inline_utf8_bytes`: maximum inline payload size per call.
- `max_patch_files_per_call`: maximum number of files accepted by `patch`.
- `max_bulk_operations_per_call`: maximum operations accepted by `fs` with `operation: "bulk"` and `args.mode: "execute"`.
- `max_log_records_per_call`: maximum records returned by `logs`.

## Notes on limits and determinism

- All list/search tools MUST define a default `limit` (default none).
- Ordering MUST be stable and documented.
- Cursor tokens MUST be opaque and deterministic.

Default ordering policy:

- Unless otherwise specified, collections of strings (paths, ids, names) MUST be stable-sorted lexicographically, case-insensitive.
- When two values compare equal under case-insensitive comparison, implementations MUST break ties deterministically (for example by comparing the original strings).
