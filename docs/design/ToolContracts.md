# Alfred Tool Contracts

This document describes the Alfred tool surface at an implementation-ready level.

Unless otherwise specified:

- Tools are executed relative to the **workspace root**.
- Read-only tools are side-effect free.
- Mutating tools support a `dry_run` mode where applicable.
- Paged list/search operations default `limit` to **100**.
- Line and column numbers are 1-indexed.

## Redaction (non-public information)

Alfred MUST filter non-public information (secrets) from:

- Tool responses
- Tool/runtime logs

Even if the secret appears in the incoming call.

Redaction is deterministic and uses a stable replacement token (for example, `"<redacted>"`). Tools SHOULD surface redaction as a warning in the result envelope metadata (rather than failing the call).

Exception: tools whose primary purpose is to manage non-public values (for example, environment variable tools) MAY return unredacted values in the tool result `data`. These tools MUST still avoid emitting those values into tool/runtime logs.

## Result envelope (all tools)

All tools MUST return results using the envelope described in [`docs/design/Protocol.md`](./Protocol.md).

In this document, each tool section's **Output** describes the tool result envelope's `data` payload unless explicitly stated otherwise.

Unless otherwise specified, each tool call is synchronous. Tools that may run long MUST either:

- Return a bounded synchronous result, or
- Return `status: "pending"` with an envelope `job_id` (see [`docs/design/Protocol.md`](./Protocol.md)) for subsequent retrieval.

Pending tool results MUST include `meta.transport_equivalent.http_status: 202`.

### Streaming results (NDJSON)

Some tools may stream large outputs. In that case, the initial tool call returns `status: "pending"` with an envelope `job_id`, and the caller retrieves NDJSON output via job tools (for example `job_read`). See [`docs/design/Protocol.md`](./Protocol.md).

## Context tools

These tools provide location awareness and are always side-effect free.

### `pwd`

- Purpose: Return Alfred's current working folder.
- Execution: synchronous.
- Input: none.
- Output:
    - `cwd`: normalized absolute string path
    - `rcwd`: normalized workspace relative string path

### `workspace_root`

- Purpose: Return the configured workspace root folder.
- Execution: synchronous.
- Input: none.
- Output:
    - `root`: normalized workspace root absolute path

## Workspace index and query tools

These tools are backed by Alfred's workspace index and MUST enforce the workspace boundary ([CNS-001](./MIS-001/Constraint/CNS-001-Workspace_Boundary_Enforcement.md)).

Unless otherwise specified:

- All returned paths are workspace-relative and use `/` separators.
- Output ordering is stable and defaults to lexicographic by `path`, then by position.

### `ls`

- Purpose: Deterministically list workspace paths.
- Execution: synchronous.
- Input:
    - `path`: optional string (default `"."`)
    - `recursive`: optional boolean (default `false`)
    - `include_hidden`: optional boolean (default `false`)
    - `include_dirs`: optional boolean (default `true`)
    - `include_files`: optional boolean (default `true`)
    - `cursor`: optional string
    - `limit`: optional integer
- Output:
    - `files`: array of strings
    - `directories`: array of strings
    - `next_cursor`: optional string

### `read_range`

- Purpose: Read a bounded range of lines from a single text file.
- Execution: synchronous.
- Input:
    - `path`: string
    - `start_line`: integer (1-indexed)
    - `end_line`: integer (inclusive; must be `>= start_line`)
- Output:
    - `path`: string
    - `start_line`: integer
    - `end_line`: integer
    - `text`: string

Note: This command MUST return an error if an attempt is made to read a binary file.

### `file_stat`

- Purpose: Return deterministic file metadata.
- Execution: synchronous.
- Input:
    - `path`: string
- Output:
    - `path`: string
    - `kind`: string (`"file" | "dir" | "symlink" | "other"`)
    - `size_bytes`: optional integer
    - `modified_at`: optional string (RFC3339 UTC)

Notes:

- This tool MUST NOT follow symlinks when determining `kind`.

### `file_read_bytes`

- Purpose: Read a bounded byte range from a single file (including binary files).
- Execution: synchronous.
- Input:
    - `path`: string
    - `offset`: integer (0-indexed byte offset)
    - `length`: integer (bytes to read; MUST be `> 0`)
- Output:
    - `path`: string
    - `offset`: integer
    - `bytes_b64`: string (base64)
    - `bytes_read`: integer
    - `eof`: boolean
    - `next_offset`: integer

Notes:

- Output MUST be bounded; maximum `length` MUST be published in `capabilities.limits`.
- `next_offset` MUST equal `offset + bytes_read`.

### `grep`

- Purpose: Search workspace text using a literal query.
- Execution: synchronous.
- Input:
    - `query`: string
    - `case_sensitive`: optional boolean (default `false`)
    - `include_pattern`: optional string (glob applied to workspace-relative paths)
    - `cursor`: optional string
    - `limit`: optional integer
- Output:
    - `matches`: array of objects:
        - `path`: string
        - `line`: integer (1-indexed)
        - `text`: string
    - `next_cursor`: optional string

### `search`

- Purpose: Search workspace text using a literal query or regular expression.
- Execution: synchronous.
- Input:
    - `search`: object, one of
        - `pattern`: string (regex)
        - `query`: string literal
    - `case_sensitive`: optional boolean (default `false`)
    - `full_text`: optional boolean (default `false`)
    - `include_pattern`: optional string (glob applied to workspace-relative paths)
    - `exclude_pattern`: optional string (glob applied to workspace-relative paths)
    - `cursor`: optional string
    - `limit`: optional integer
- Output:
    - `matches`: array of objects:
        - `path`: string
        - `line`: integer (1-indexed)
        - `text`: string
    - `next_cursor`: optional string

### `symbols`

- Purpose: Return a deterministic list of symbols from the workspace index.
- Execution: synchronous.
- Input:
    - `query`: optional string
    - `path`: optional string (restrict to one file)
    - `kinds`: optional array of strings (implementation-defined symbol kinds)
    - `cursor`: optional string
    - `limit`: optional integer
- Output:
    - `symbols`: array of objects:
        - `name`: string
        - `kind`: string
        - `path`: string
        - `range`: object:
            - `start_line`: integer (1-indexed)
            - `start_col`: integer (1-indexed)
            - `end_line`: integer (1-indexed)
            - `end_col`: integer (1-indexed)
    - `next_cursor`: optional string

Note: This feature depends on being able to communicate with the hosting environment's language servers for parsing.

### `diff`

- Purpose: Produce a deterministic textual diff.
- Execution: synchronous.
- Input:
    - `a`: one of:
        - object:
            - `path`: string
            - `from`: integer (1-indexed)
            - `to`: integer (1-indexed; inclusive; must be `>= from`)
        - string
    - `b`: one of:
        - object:
            - `path`: string
            - `from`: integer (1-indexed)
            - `to`: integer (1-indexed; inclusive; must be `>= from`)
        - string
- Output:
    - `diff`: string (unified diff format)

Notes:

- Git operations are out of scope ([CNS-011](./MIS-001/Constraint/CNS-011-Git_and_GitHub_Operations_Out_of_Scope.md)). This tool compares text/file contents; it does not perform git plumbing.

## File mutation tools

These tools mutate workspace contents and MUST enforce:

- Workspace boundary ([CNS-001](./MIS-001/Constraint/CNS-001-Workspace_Boundary_Enforcement.md))
- Dry-run support for destructive operations ([CNS-003](./MIS-001/Constraint/CNS-003-DryRun_for_Destructive_Operations.md))
- Atomic mutation guarantees per target where practical ([CNS-004](./MIS-001/Constraint/CNS-004-Atomic_Mutations_Per_Target.md))

Unless otherwise specified, all mutating tools accept:

- `dry_run`: optional boolean (default `true`)

### `file_create`

- Purpose: Create a file deterministically.
- Execution: synchronous.
- Input:
    - `path`: string
    - `content`: string
    - `dry_run`: optional boolean
- Output:
    - `path`: string
    - `bytes_written`: integer

### `file_append`

- Purpose: Append to a file deterministically.
- Execution: synchronous.
- Input:
    - `path`: string
    - `content`: string
    - `dry_run`: optional boolean
- Output:
    - `path`: string
    - `bytes_written`: integer

### `file_patch`

- Purpose: Modify to an existing file deterministically.
- Execution: synchronous.
- Input:
    - `path`: string
    - `patch`: string, mpatch format
    - `dry_run`: optional boolean
- Output:
    - `path`: string
    - `bytes_written`: integer

### `multi_file_patch`

- Purpose: Apply a multi-file patch.
- Execution: synchronous.
- Input:
    - `patches`: array of objects:
        - `path`: string
        - `patch`: string, mpatch format
    - `dry_run`: optional boolean
- Output:
    - `files`: array of objects:
        - `path`: string
        - `patched`: boolean
        - `conflicts`: optional array of objects (mpatch defined)

### `file_delete`

- Purpose: Delete a file.
- Execution: synchronous.
- Input:
    - `path`: string
    - `dry_run`: optional boolean
- Output:
    - No additional fields.

### `dir_create`

- Purpose: Create a directory or directory tree (mkdir semantics) within the workspace.
- Execution: synchronous.
- Input:
    - `path`: string
    - `parents`: boolean, equivalent to `mkdir -p` (default `false`)
    - `dry_run`: optional boolean
- Output:
    - No additional fields.

### `dir_delete`

- Purpose: Delete a directory (rmdir semantics) within the workspace.
- Execution: synchronous.
- Input:
    - `path`: string
    - `dry_run`: optional boolean
- Output:
    - No additional fields.

Note: This is not intended to remove non-empty directories.

### `file_create_bytes`

- Purpose: Create a file from base64-encoded bytes.
- Execution: synchronous.
- Input:
    - `path`: string
    - `bytes_b64`: string (base64)
    - `dry_run`: optional boolean
- Output:
    - `path`: string
    - `bytes_written`: integer

Notes:

- This tool is intended for binary content and large files where inline UTF-8 content is not suitable.
- `bytes_b64` MUST decode to a bounded byte length per call; the limit MUST be published in `capabilities.limits`.

### `file_append_bytes`

- Purpose: Append base64-encoded bytes to a file.
- Execution: synchronous.
- Input:
    - `path`: string
    - `bytes_b64`: string (base64)
    - `dry_run`: optional boolean
- Output:
    - `path`: string
    - `bytes_written`: integer

Notes:

- For very large writes, callers SHOULD chunk content across multiple calls.
- The server MUST remain robust for large total file sizes; only per-call payload size is bounded.

### `path_move`

- Purpose: Perform deterministic bulk move/rename operations (files and/or directories).
- Execution: synchronous.
- Input:
    - `moves`: array of objects:
        - `from`: string
        - `to`: string
    - `overwrite`: optional boolean (default `false`)
    - `create_parents`: optional boolean (default `false`)
    - `dry_run`: optional boolean
- Output:
    - `moves`: array of objects (same order as input):
        - `from`: string
        - `to`: string
        - `moved`: boolean
        - `warning`: optional string

Notes:

- `from` and `to` MUST be workspace-relative paths.
- The tool MUST reject any move that would escape the workspace boundary.
- If `overwrite` is `false` and `to` already exists, the corresponding item MUST set `moved: false` with a deterministic `warning` value (for example `"target_exists"`).

### `path_copy`

- Purpose: Perform deterministic bulk copy operations (files and/or directories).
- Execution: synchronous or async (MAY return `pending` for large copies).
- Input:
    - `copies`: array of objects:
        - `from`: string
        - `to`: string
    - `overwrite`: optional boolean (default `false`)
    - `create_parents`: optional boolean (default `false`)
    - `dry_run`: optional boolean
- Output:
    - `copies`: array of objects (same order as input):
        - `from`: string
        - `to`: string
        - `copied`: boolean
        - `warning`: optional string

Notes:

- For symlinks, Alfred MUST treat the symlink itself as the filesystem object to copy and MUST NOT follow it.
- If a platform cannot copy a symlink without elevated privileges, the corresponding item MUST set `copied: false` with a deterministic `warning` value.

### `path_delete`

- Purpose: Perform deterministic bulk delete operations.
- Execution: synchronous.
- Input:
    - `paths`: array of strings
    - `recursive`: optional boolean (default `false`)
    - `dry_run`: optional boolean
- Output:
    - `deleted`: array of objects (same order as input):
        - `path`: string
        - `deleted`: boolean
        - `warning`: optional string

Notes:

- When `recursive` is `false`, the corresponding item MUST set `deleted: false` with a deterministic `warning` value if the path is a non-empty directory.
- The tool MUST delete symlinks as leaf nodes and MUST NOT follow them.

## Task execution tools

These tools run local processes under explicit guardrails. By default, tools MUST NOT invoke a shell ([CNS-018](./MIS-001/Constraint/CNS-018-No_Shell_by_Default_for_Task_Execution.md)).

Tasks using some tools, such as `npm` assume certain script names in the configuration.

Long-running work SHOULD return `status: "pending"` with an envelope `job_id` (see [`docs/design/Protocol.md`](./Protocol.md)).

### `list_tasks`

- Purpose: List the available tasks in the current workspace (e.g. `build`, `test`) based on language(s) in use.
- Execution: synchronous.
- Input:
    - `language`: optional string filter
    - `cursor`: optional string
    - `limit`: optional integer
- Output (sync):
    - `language`: string
    - `task`: string
    - `description`: string
    - `parameters`: array of objects:
        - `name`: string
        - `type`: string
        - `description`: string

### `task_run`

- Purpose: Run a named task (e.g. `build`, `test`) selected from a safe allowlist.
- Execution: synchronous or async.
- Input:
    - `task`: string
    - `parameters`: optional object, string/string name/value pairs
    - `timeout_ms`: optional integer
    - `async`: optional boolean (default `false`)
- Output (sync):
    - `exit_code`: integer
    - `results`: array of string, see [ART-003 Diagnostics Report](./MIS-001/Artifact/ART-003-Diagnostics_Report.md)
- Output (async / pending):
    - No additional fields (the envelope includes `job_id`).

### Tasks

This list is not exhaustive.

- Rust:
    - `format`: `cargo fmt`
    - `lint`: `cargo clippy`
    - `test`: `cargo test`
    - `run`: `cargo run`
    - `build`: `cargo build`
    - `release`: `cargo build --release`
- Node (npm):
    - `format`: `npm run format`
    - `lint`: `npm run lint`
    - `test`: `npm run test`
    - `run`: `npm run dev`
    - `build`: `npm run build`
    - `release`: `npm run release`
- Node (pnpm):
    - `format`: `pnpm format`
    - `lint`: `pnpm lint`
    - `test`: `pnpm test`
    - `run`: `pnpm dev`
    - `build`: `pnpm build`
    - `release`: `pnpm release`

## Background job tools

These tools provide deterministic control and retrieval for background work started by other tools.

### `job_status`

- Purpose: Retrieve job status.
- Execution: synchronous.
- Input:
    - `job_id`: string
- Output:
    - `job_id`: string
    - `state`: string (`"queued" | "running" | "succeeded" | "failed" | "canceled"`)
    - `started_at`: optional string (RFC3339 UTC)
    - `ended_at`: optional string (RFC3339 UTC)
    - `exit_code`: optional integer

### `job_statuses`

- Purpose: Return a minimal status list for all running jobs and recently completed jobs whose output has not been fully delivered.
- Execution: synchronous.
- Input:
    - None.
- Output:
    - `jobs`: array of objects:
        - `job_id`: string
        - `state`: string (`"queued" | "running" | "succeeded" | "failed" | "canceled"`)

Notes:

- "Recently completed" means the job is in a terminal state but still has unread output available via `job_read`.

### `job_cancel`

- Purpose: Request cancellation of a running job.
- Execution: synchronous.
- Input:
    - `job_id`: string
- Output:
    - `job_id`: string
    - `canceled`: boolean

### `job_list`

- Purpose: List recent jobs.
- Execution: synchronous.
- Input:
    - `cursor`: optional string
    - `limit`: optional integer
- Output:
    - `jobs`: array of objects:
        - `job_id`: string
        - `state`: string (`"queued" | "running" | "succeeded" | "failed" | "canceled"`)
        - `started_at`: optional string (RFC3339 UTC)
        - `ended_at`: optional string (RFC3339 UTC)
        - `exit_code`: optional integer
    - `next_cursor`: optional string

### `job_read`

- Purpose: Read a job's output stream ([ART-006 Job Output Stream](./MIS-001/Artifact/ART-006-Job_Output_Stream.md)).
- Execution: synchronous.
- Input:
    - `job_id`: string
    - `cursor`: optional string
    - `limit`: optional integer (lines/items)
    - `encoding`: optional `"json" | "ndjson"` (default `"ndjson"`)
- Output:
    - `items`: optional array (when `encoding: "json"`)
    - `ndjson`: optional string (when `encoding: "ndjson"`)
    - `next_cursor`: optional string

## Session introspection tools

These tools provide deterministic introspection over the current Alfred process session (process lifetime).

### `session_recent`

- Purpose: Return recent tool calls and their results (redacted), suitable for troubleshooting and agent self-awareness.
- Execution: synchronous.
- Input:
    - `cursor`: optional string
    - `limit`: optional integer
    - `include_args`: optional boolean (default `false`)
    - `include_results`: optional boolean (default `false`)
- Output:
    - `calls`: array of objects:
        - `id`: string
        - `timestamp`: string (RFC3339 UTC)
        - `tool`: string
        - `status`: string (`"ok" | "error" | "pending"`)
        - `job_id`: optional string
        - `error_kind`: optional string
        - `args`: optional object (redacted)
        - `result`: optional object (tool result envelope; redacted and bounded)
    - `next_cursor`: optional string

Notes:

- Returned `args` and `result` MUST be deterministically redacted.
- `include_results: true` MUST still enforce a bounded payload; for larger results, the tool MUST return `pending` with a `job_id`.

## Log tools

Log tools tail and filter Alfred/runtime logs using the structured log record defined in [`docs/design/Protocol.md`](./Protocol.md).

### `log_tail`

- Purpose: Start a bounded log tail as a background job.
- Execution: async (returns `pending`).
- Input:
    - `path`: optional string (defaults to the path to Alfred's log)
    - `level_min`: optional string (`"TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"`)
    - `source_prefix`: optional string
    - `lines_before`: optional integer (default 10)
    - `since`: optional string (RFC3339 UTC)
- Output (pending):
    - No additional fields (the envelope includes `job_id`; use `job_read`).

### `log_search`

- Purpose: Search logs deterministically.
- Execution: synchronous.
- Input:
    - `path`: optional string (defaults to the path to Alfred's log)
    - `query`: string
    - `level`: optional string
    - `source_prefix`: optional string
    - `cursor`: optional string
    - `limit`: optional integer
- Output:
    - `matches`: array of log records
    - `next_cursor`: optional string

## Plan tools

Plan tooling reads and updates the project plan ([ART-004 Project Plan](./MIS-001/Artifact/ART-004-Project_Plan.md)). The plan format is standardized, and must be human readable.

### Plan item schema

- `id` (integer): stable identifier (sequential, starting at 1)
- `title` (string): concise label
- `priority` (integer): 0 (Critical/Blocker) to 3 (Low)
- `cards` (array of string): related Aurora card ids
- `description` (string): summary of the work to be done
- `deliverables` (array of string): specific items to be delivered
- `acceptance_criteria` (optional string): additional criteria, beyond the deliverables, that must be met
- `notes` (optional string): additional useful information
- `status` (string): `"planned" | "in-progress" | "completed" | "cancelled"`

**Example**:

```markdown
1. [ ] Implement MCP stdio Interface
    - Priority: 2
    - Cards: "INT-001", "ART-001", "ART-002"
    - Description: Per the Aurora model, implement the stdio contract used by the MCP host to call Alfred tools and receive structured results.
    - Deliverables:
        - A stdio MCP interface accepting ART-001 and returning ART-002.
        - Passing positive, negative, and security tests.
    - Status: planned
```

### `plan_get`

- Purpose: Read the current plan.
- Execution: synchronous.
- Input: none.
- Output:
    - `items`: array of plan items

### `plan_update`

- Purpose: Update the status of a specific plan item
- Execution: synchronous.
- Input:
    - `id`: id of the item to update
    - `status`: the new status
- Output:
    - No additional fields.

### `plan_edit`

- Purpose: Edit a plan item.
- Execution: synchronous.
- Input:
    - A single plan item.
- Output:
    - No additional fields.

### `plan_add`

- Purpose: Append a plan item.
- Execution: synchronous.
- Input:
    - A single plan item. `id` will be ignored.
- Output:
    - `id`: id of the new item.

### `plan_delete`

- Purpose: Remove a plan item.
- Execution: synchronous.
- Input:
    - `id`: id of the item to remove
- Output:
    - No additional fields.

## Capability discovery

### `capabilities`

- Purpose: Advertise supported tools, versions, execution modes, and limits.
- Execution: synchronous.
- Input: none.
- Output:
    - `tools`: array of objects (stable-sorted by `name`):
        - `name`: string
        - `version`: string (SemVer)
        - `schema_version`: string (SemVer; lockstep with `version`)
        - `execution_modes`: array (`"sync" | "async"`)
        - `limits`: optional object

Limits (when present) SHOULD include:

- `max_inline_utf8_bytes`: maximum UTF-8 payload size Alfred will accept/emit inline per call.
- `max_file_chunk_bytes`: maximum decoded bytes supported by `file_read_bytes`, `file_create_bytes`, and `file_append_bytes` per call.
- `max_ndjson_item_bytes`: maximum size of a single NDJSON item line emitted via job tooling.

## Action chaining

### `chain`

- Purpose: Execute a deterministic multi-step chain in a single call.
- Execution: synchronous or async.
- Input:
    - `steps`: array of objects:
        - `tool`: string
        - `args`: object
    - `stop_on_failure`: optional boolean (default `true`)
    - `mode`: optional `"sync" | "async"` (default `"sync"`)
- Output (sync):
    - `steps`: array of objects:
        - `index`: integer (1-based)
        - `tool`: string
        - `result`: tool result envelope
    - `stopped_early`: boolean
- Output (async / pending):
    - No additional fields (the envelope includes `job_id`).

## Environment variable tools

Environment variable CRUD is scoped to Alfred-controlled contexts ([CNS-020](./MIS-001/Constraint/CNS-020-Environment_Variable_CRUD_Scope.md)). These tools MUST NOT claim to mutate the parent IDE or shell environment across OSes.

For simplicity, Alfred maintains a single internal environment variable list that is automatically made available to tools called through Alfred.

Unless otherwise specified, environment variable values MUST be treated as non-public information and MUST NOT be written to tool/runtime logs.

When constructing Alfred-controlled contexts from a source environment (for example, the host IDE), Alfred MUST filter the source environment to exclude non-public information (NPI) entries that should not be inherited.

### `env_list`

- Purpose: List environment variables in a managed context.
- Execution: synchronous.
- Input:
    - None.
- Output:
    - `environment`: object, set of string/string key/value pairs

### `env_get`

- Purpose: Retrieve an environment variable from a managed context.
- Execution: synchronous.
- Input:
    - `key`: string
- Output:
    - `value`: optional string

### `env_set`

- Purpose: Set an environment variable in a managed context.
- Execution: synchronous.
- Input:
    - `key`: string
    - `value`: string
    - `dry_run`: optional boolean
- Output:
    - No additional fields.

### `env_unset`

- Purpose: Unset an environment variable in a managed context.
- Execution: synchronous.
- Input:
    - `key`: string
    - `dry_run`: optional boolean
- Output:
    - No additional fields.

## Memory tools (local, indexed, searchable)

The memory toolset provides an offline-only, persistent store for structured “facts”, plus full-text search.

### Storage (user + workspace)

Alfred supports two physical stores for memory facts:

- **User Store (default)**: persisted in the OS user data directory under `alfred/` (for example `~/.local/alfred/alfred.sqlite3`).
- **Workspace Store (optional)**: persisted under the workspace root in `.agents/` (default `.agents/alfred/alfred.sqlite3`).

When a Workspace Store is enabled, Alfred exposes an **effective** view over both stores:

- **Merged (default)**: effective set is the union of both stores; when `id` collides, Workspace overrides User.
- **Prefer workspace**: workspace is primary; the User store is still consulted for `category: preferences`, then overlaid by workspace.

### Fact schema

A fact uses the [ART-008 Memory Fact](./MIS-001/Artifact/ART-008-Memory_Fact.md) shape (see the Aurora model), with these required fields:

- `id` (string): stable identifier
- `subject` (string)
- `fact` (string)
- `citations` (string)
- `reason` (string)
- `category` (string)

Optional fields:

- `tags` (array of strings): caller-controlled labels used for deterministic filtering (stable-sorted ascending; recommended lower-case `kebab-case`)

Stored facts returned by tools also include:

- `created_at` (string): RFC3339 UTC timestamp (seconds preferred; max milliseconds)
- `updated_at` (string): RFC3339 UTC timestamp (seconds preferred; max milliseconds)

#### Category values

`category` SHOULD be one of:

- `general`
- `user_preferences`
- `documentation_practices`
- `coding_practices` (Include the language as a tag)
- `preferred_libraries` (Include the language as a tag)
- `file_specific`
- `bootstrap_and_build`

Alfred MUST treat `category` as an open set (unknown values are accepted) so callers can introduce additional categories without server upgrades.

### `memory_save`

- Purpose: Upsert a fact.
- Execution: synchronous.
- Input:
    - A fact object
- Output:
    - `id`: id of the upserted fact

### `memory_get`

- Purpose: Retrieve a single fact by id.
- Execution: synchronous.
- Input:
    - `id`: string
- Output:
    - `fact`: fact object

### `memory_delete`

- Purpose: Delete a single fact by id.
- Execution: synchronous.
- Input:
    - `id`: string
    - `dry_run`: optional boolean (default `false`)
- Output:
    - `deleted`: boolean

### `memory_list`

- Purpose: List facts deterministically.
- Execution: synchronous.
- Input:
    - `cursor`: optional string
    - `limit`: optional integer
    - `order`: optional `"created_at" | "updated_at" | "subject"` (default `"updated_at"`)
    - `subject`: optional string (exact match)
    - `category`: optional string (exact match)
    - `tags_any`: optional array of strings (match if any tag is present)
    - `tags_and`: optional boolean (default `false`)
- Output:
    - `facts`: array of fact objects (bounded)
    - `next_cursor`: optional string

### `memory_search`

- Purpose: Full-text search over memory facts.
- Execution: synchronous by default; MAY support async for very large stores.
- Input:
    - `query`: string
    - `limit`: optional integer
    - `cursor`: optional string
    - `subject`: optional string (exact match)
    - `category`: optional string (exact match)
    - `tags_any`: optional array of strings
    - `tags_and`: optional boolean (default `false`)
- Output:
    - `matches`: array of objects:
        - `fact`: fact object
        - `score`: number
        - `highlights`: optional object

## Notes on limits and determinism

- All list/search tools MUST define a default `limit` (default 100).
- Ordering MUST be stable and documented.
- Cursor tokens MUST be opaque and deterministic.

## Default paging limits

Unless otherwise specified:

- `limit` default: `100`
