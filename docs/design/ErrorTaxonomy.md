# Alfred Deterministic Error Taxonomy

This document defines Alfred’s _tool-level_ deterministic error taxonomy. It is used inside Alfred tool results (not as a replacement for MCP/JSON-RPC transport errors).
All tool implementations MUST map failures into the kinds defined here. Other design documents (for example `ToolContracts.md` and `Configuration.md`) MUST NOT specify concrete error kinds or `details` fields inline; they MUST reference this taxonomy instead.

Canonical governance for this taxonomy is defined in [`DesignAuthority.md`](./DesignAuthority.md) and [`QualityPolicy.md`](./QualityPolicy.md).

## Error object shape

A tool error MUST have the following fields:

- `kind`: string (taxonomy code)
- `message`: string (human-readable, stable wording preferred)
- `retryable`: boolean
- `details`: optional object (tool-specific structured data)

## Core error kinds

| kind                           | Meaning                                                                               | Typical retryable? |
| ------------------------------ | ------------------------------------------------------------------------------------- | ------------------ |
| `invalid_argument`             | Input failed validation (type/range/format/policy).                                   | No                 |
| `permission_denied`            | Request blocked by policy or permission model.                                        | No                 |
| `workspace_boundary_violation` | Path or operation escapes or ambiguously interacts with the workspace boundary.       | No                 |
| `not_found`                    | Resource does not exist (file, fact, job id, patch id, etc.).                         | No                 |
| `conflict`                     | Conflict detected (patch conflict, write conflict, version mismatch).                 | Usually No         |
| `timeout`                      | Operation exceeded a deterministic time budget.                                       | Maybe              |
| `canceled`                     | Operation was canceled and reached a terminal canceled state.                         | Maybe              |
| `resource_exhausted`           | Output or computation hit configured limits (bytes, matches, memory).                 | Maybe              |
| `tool_unavailable`             | Required external toolchain or internal subsystem is missing, disabled, or not ready. | Maybe              |
| `io_error`                     | Filesystem/stdio IO error.                                                            | Maybe              |
| `internal`                     | Unexpected failure inside Alfred.                                                     | Maybe              |

## Notes

- Free-form logs MUST NOT be the only place where error information exists.
- `details` SHOULD include enough structure to support deterministic conformance tests.
- Non-public information (NPI) MUST be filtered from responses and logs deterministically. Redaction SHOULD be surfaced as a warning in the tool result envelope (see [`docs/design/Protocol.md`](./Protocol.md)) rather than introducing a new error kind.

## Common `details` conventions

When present, `details` MUST be a JSON object.

### `details.reason`

`details.reason` is a stable, machine-readable discriminator for common tool failure classes. It is intended to support deterministic client downgrade/retry logic and conformance tests.

`details.reason` is an **open set**: new values MAY be added in future versions. Callers MUST handle unknown reason values gracefully (for example, by treating them as opaque and using `kind` for control flow).

### Policy and configuration gating

- Tool disabled by policy:
    - `kind`: `invalid_argument`
    - `retryable`: `false`
    - `details`: `{ "reason": "tool_disabled", "tool": "<tool_name>" }`

### Index-backed operations

Index-backed operations (for example workspace search/query tools) use `tool_unavailable` for index state gating.

- Index disabled:
    - `kind`: `tool_unavailable`
    - `retryable`: `true`
    - `details`: `{ "reason": "index_disabled" }`

- Index not ready:
    - `kind`: `tool_unavailable`
    - `retryable`: `true`
    - `details`: `{ "reason": "index_not_ready" }`

### Workspace boundary violations

A dedicated kind is used so callers can distinguish boundary escapes from ordinary not-found errors.

- Boundary escape or ambiguous boundary interaction:
    - `kind`: `workspace_boundary_violation`
    - `retryable`: `false`
    - `details`: `{ "reason": "workspace_boundary_violation" }`

### Binary or non-text input

- Operation requires text input but received binary or non-representable content:
    - `kind`: `invalid_argument`
    - `retryable`: `false`
    - `details`: `{ "reason": "binary_input" }`

### Duplicate content safeguard

- `patch` would duplicate existing file content:
    - `kind`: `conflict`
    - `retryable`: `false`
    - `details`: `{ "reason": "duplicate_content" }`

### Patch revert errors

- The revert state for the given `patch_id` is no longer retained (superseded by a newer patch call):
    - `kind`: `not_found`
    - `retryable`: `false`
    - `details`: `{ "reason": "patch_id_not_retained", "patch_id": "<uuid>" }`

- The target file has changed since the patch was applied; revert refused:
    - `kind`: `conflict`
    - `retryable`: `false`
    - `details`: `{ "reason": "file_changed_since_patch", "path": "<path>" }`

### Streaming operations

- A streaming operation was refused because another stream is already active:
    - `kind`: `resource_exhausted`
    - `retryable`: `true`
    - `details`: `{ "reason": "stream_active" }`

- A stop or cancel request was issued for a stream, but no stream is active:
    - `kind`: `invalid_argument`
    - `retryable`: `false`
    - `details`: `{ "reason": "stream_not_active" }`

### Search mode parameter conflicts

- A search parameter is incompatible with the supplied `mode`:
    - `kind`: `invalid_argument`
    - `retryable`: `false`
    - `details`: `{ "reason": "mode_parameter_conflict", "parameter": "<param_name>", "mode": "<supplied_mode>" }`
