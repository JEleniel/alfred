# Alfred Deterministic Error Taxonomy

This document defines Alfred’s _tool-level_ deterministic error taxonomy. It is used inside Alfred tool results (not as a replacement for MCP/JSON-RPC transport errors).

## Error object shape

A tool error MUST have the following fields:

- `kind`: string (taxonomy code)
- `message`: string (human-readable, stable wording preferred)
- `retryable`: boolean
- `details`: optional object (tool-specific structured data)

## Core error kinds

| kind                           | Meaning                                                                     | Typical retryable? |
| ------------------------------ | --------------------------------------------------------------------------- | ------------------ |
| `invalid_argument`             | Input failed validation (type/range/format).                                | No                 |
| `permission_denied`            | Request blocked by policy/permission model.                                 | No                 |
| `workspace_boundary_violation` | Path or operation escapes or ambiguously interacts with workspace boundary. | No                 |
| `not_found`                    | Resource does not exist (file, fact, job id, etc.).                         | No                 |
| `conflict`                     | Conflict detected (patch conflict, write conflict, version mismatch).       | Usually No         |
| `timeout`                      | Operation exceeded a deterministic time budget.                             | Maybe              |
| `canceled`                     | Operation was canceled and reached a terminal canceled state.               | Maybe              |
| `resource_exhausted`           | Output or computation hit configured limits (bytes, matches, memory).       | Maybe              |
| `tool_unavailable`             | External executable/toolchain missing or incompatible.                      | Maybe              |
| `io_error`                     | Filesystem/stdio IO error.                                                  | Maybe              |
| `internal`                     | Unexpected failure inside Alfred.                                           | Maybe              |

## Notes

- Tool implementations MUST map failures into one of the above kinds.
- Free-form logs MUST NOT be the only place where error information exists.
- `details` SHOULD include enough structure to support deterministic conformance tests.

Non-public information (secrets) MUST be filtered from responses and logs deterministically. Redaction SHOULD be surfaced as a warning in tool metadata rather than introducing a new error kind.
