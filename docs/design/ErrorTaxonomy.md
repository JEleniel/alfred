# Alfred Deterministic Error Taxonomy

This document defines Alfred’s _tool-level_ deterministic error taxonomy. It is used inside Alfred tool results (not as a replacement for MCP/JSON-RPC transport errors).

## Error object shape

A tool error MUST have the following fields:

- `kind`: string (taxonomy code)
- `message`: string (human-readable, stable wording preferred)
- `retryable`: boolean
- `details`: optional object (tool-specific structured data)

## Core error kinds

| kind                           | Meaning                                                                               | Typical retryable? |
| ------------------------------ | ------------------------------------------------------------------------------------- | ------------------ |
| `invalid_argument`             | Input failed validation (type/range/format).                                          | No                 |
| `permission_denied`            | Request blocked by policy/permission model.                                           | No                 |
| `workspace_boundary_violation` | Path or operation escapes or ambiguously interacts with workspace boundary.           | No                 |
| `not_found`                    | Resource does not exist (file, fact, job id, etc.).                                   | No                 |
| `conflict`                     | Conflict detected (patch conflict, write conflict, version mismatch).                 | Usually No         |
| `timeout`                      | Operation exceeded a deterministic time budget.                                       | Maybe              |
| `canceled`                     | Operation was canceled and reached a terminal canceled state.                         | Maybe              |
| `resource_exhausted`           | Output or computation hit configured limits (bytes, matches, memory).                 | Maybe              |
| `tool_unavailable`             | Required external toolchain or internal subsystem is missing, disabled, or not ready. | Maybe              |
| `io_error`                     | Filesystem/stdio IO error.                                                            | Maybe              |
| `internal`                     | Unexpected failure inside Alfred.                                                     | Maybe              |

## Notes

- Tool implementations MUST map failures into one of the above kinds.
- Free-form logs MUST NOT be the only place where error information exists.
- `details` SHOULD include enough structure to support deterministic conformance tests.

Non-public information (secrets) MUST be filtered from responses and logs deterministically. Redaction SHOULD be surfaced as a warning in the tool result envelope (see [`docs/design/Protocol.md`](./Protocol.md)) rather than introducing a new error kind.

## Common `details` conventions

This document is the single source of truth for tool-level error semantics. Other design documents (for example `ToolContracts.md` and `Configuration.md`) should avoid specifying concrete error kinds or `details` fields inline and MUST reference this taxonomy instead.

When present, `details` MUST be a JSON object.

### `details.reason`

`details.reason` is a stable, machine-readable discriminator for common tool failure classes. It is intended to support deterministic client downgrade/retry logic and conformance tests.

Known values:

- `tool_disabled`: the tool exists but is disabled by configuration/policy.
- `index_disabled`: an index-backed operation is unavailable because indexing is disabled.
- `index_not_ready`: an index-backed operation is unavailable because the index is not yet ready.
- `workspace_boundary_violation`: the request attempted to escape the workspace boundary (or used an ambiguous path).
- `binary_input`: the operation requires text input but was provided non-text/binary content.
- `duplicate_content`: the operation was refused because it would duplicate existing content (for example a patch that appends a full-file copy).
- `stream_active`: a streaming operation was refused because another stream is already active.
- `stream_not_active`: a stop/cancel request was issued for a stream, but no stream is active.

### Policy and configuration gating

- Tool disabled by policy:
    - `kind`: `invalid_argument`
    - `retryable`: `false`
    - `details`: `{ "reason": "tool_disabled", "tool": "<tool_name>" }`

### Index-backed operations

Index-backed operations (for example workspace search/query tools) SHOULD use `tool_unavailable` for index state gating.

- Index disabled:
    - `kind`: `tool_unavailable`
    - `retryable`: `true`
    - `details`: `{ "reason": "index_disabled" }`

- Index not ready:
    - `kind`: `tool_unavailable`
    - `retryable`: `true`
    - `details`: `{ "reason": "index_not_ready" }`

### Workspace boundary violations

Workspace-boundary enforcement SHOULD use a dedicated kind so callers can distinguish boundary escapes from ordinary not-found errors.

- Boundary escape / ambiguous boundary interaction:
    - `kind`: `workspace_boundary_violation`
    - `retryable`: `false`
    - `details`: `{ "reason": "workspace_boundary_violation" }`
