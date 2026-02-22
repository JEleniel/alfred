# Project Plan

## Mission

Keep the Alfred architecture model and design artifacts implementation-ready, with deterministic and secure tool contracts.

## Current work items

1. [x] Add “Local, Indexed, Searchable Memory” to the MIS-001 Aurora model.
2. [x] Specify missing protocol and tool-contract details to address current design gaps.
3. [x] Regenerate and validate the Aurora renderings and compact export.
4. [x] Update top-level design docs to reference the new contracts and the local memory feature.
5. [x] Align requirement-to-contract traceability for bulk file ops, any-size file IO, and session introspection.
6. [ ] Update the Aurora model cards to reflect the expanded tool surface.
7. [ ] Regenerate and validate the Aurora renderings and compact export.

## Acceptance criteria

- The Aurora model validates.
- The rendered model bundle in `docs/design/MIS-001/` includes the new Memory feature (traceable from Driver → Requirement → Capability → Feature → Component → Data Store).
- The design includes a concrete, implementable contract for:
    - Request/response framing and payload encoding expectations.
    - Deterministic error envelope shape and taxonomy reference.
    - Memory CRUD and search tool shapes.
    - Memory retrieval by `subject` and `category`, plus optional `tags` filtering.
    - Default paging limit of 100 unless otherwise specified.
    - Async initiation via `status: "pending"` with `meta.transport_equivalent.http_status: 202`.
    - Standardized NDJSON log record shape suitable for tool consumption.
    - Deterministic redaction of secrets from responses and logs.
    - Bulk file operations with dry-run support (`path_move`, `path_copy`, `path_delete`).
    - Session introspection for recent tool calls/results (`session_recent`).
    - Any-size file handling via bounded byte-chunk IO (`file_read_bytes`, `file_create_bytes`, `file_append_bytes`) and advertised per-call limits.

## Notes / open decisions

- NDJSON transport: Alfred MUST remain MCP-compliant and therefore MUST only emit MCP/JSON frames over stdio. Large result streaming uses either:
    - Embedded NDJSON text inside a normal MCP tool response payload, or
    - `status: "pending"` + `job_id`, followed by append-only NDJSON items streamed via job tooling.
      Alfred MUST NOT switch the stdio transport into an out-of-band “raw NDJSON stream” mode.
- Memory store locations and precedence:
    - Default persistent **User Store**: user data directory under `alfred/` (for example `~/.local/alfred/alfred.sqlite3`).
    - Optional **Workspace Store**: workspace-local under `.agents/` (default `.agents/alfred/alfred.sqlite3`).
    - Default effective view is **merged** (workspace overlays user when ids collide).
    - Optional mode: **prefer workspace** (workspace is primary; user store is still consulted for `category: user_preferences`, then overlaid by workspace).
    - `category: user_preferences` MUST always be persisted in the User Store and MAY be overridden by a Workspace Store entry with the same `id`.
