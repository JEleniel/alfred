# Alfred Protocol Notes

This document defines Alfred-specific protocol expectations on top of the Model Context Protocol (MCP) stdio transport.

Alfred is intended to be used via an MCP host (for example, an IDE) that:

- Launches Alfred as a local stdio process.
- Exchanges newline-delimited JSON frames.
- Calls tools via the MCP tool surface.

## Framing and encoding

- Transport is **stdio**.
- Each inbound request and outbound response is a single **UTF-8 JSON** value on its own line.
- Alfred MUST reject malformed frames deterministically.
- All protocol-visible strings MUST be valid UTF-8.
- All protocol-visible strings MUST be valid UTF-8.
    - When interacting with filesystem paths that cannot be represented as valid Unicode text, Alfred MUST encode them deterministically before emitting JSON.

## Path strings

Unless a tool contract explicitly states otherwise, all `path` values in tool inputs and outputs:

- Are **workspace-relative**.
- Use **POSIX separators** (`/`) regardless of OS.
- MUST NOT be absolute.

If an incoming request provides a path using `\` separators, Alfred MUST treat them as `/` separators before normalization.

### Deterministic encoding for non-text paths

Filesystems can contain paths that are not valid Unicode strings (for example, arbitrary bytes on Unix, or ill-formed UTF-16 on Windows). Alfred MUST still emit valid UTF-8 JSON.

When a path cannot be represented as Unicode text, Alfred MUST emit an encoded ASCII-only representation using these rules:

- Unix (byte paths): represent each non-UTF-8 byte as `\xNN` (uppercase hex), and escape backslash as `\\`.
- Windows (UTF-16): represent any unpaired surrogate 16-bit unit as `\u{XXXX}` (uppercase hex). Other code points MUST be emitted normally.

When encoded path rendering occurs, Alfred SHOULD add a warning via `meta.warnings` (for example `{"kind":"path_encoded"}`) and MUST ensure ordering/cursors use the encoded representation consistently.

## Alfred tool result envelope

Within MCP tool responses, Alfred tool results MUST use a consistent envelope so that clients can:

- Distinguish success vs failure without brittle string matching.
- Apply retry and downgrade logic using deterministic error kinds.
- Consume large results deterministically.

### Success

A successful tool result MUST be shaped as:

- `status`: `"ok"`
- `data`: tool-specific payload
- `meta`: common metadata

Common metadata fields:

- `tool`: tool name
- `schema_version`: SemVer string for the tool result schema (lockstep with tool version)
- `duration_ms`: integer duration as observed by Alfred
- `warnings`: optional array of warning objects

### Failure

A failed tool result MUST be shaped as:

- `status`: `"error"`
- `error`: deterministic error object (see `docs/design/ErrorTaxonomy.md`)
- `meta`: common metadata

### Pending (accepted)

Some tools may start a bounded background job (for example, long-running tasks or large streaming results). In that case the initial call MUST return:

- `status`: `"pending"`
- `job_id`: string
- `data`: tool-specific initial payload
- `meta`: common metadata

`status: "pending"` is semantically equivalent to **HTTP 202 Accepted**. Since Alfred runs over stdio, this is expressed as metadata:

- `meta.transport_equivalent.http_status`: `202`

## NDJSON and streaming

Some tools may produce large result sets (for example workspace search or memory search). Alfred supports two response modes:

- **Synchronous (JSON)**: the tool returns a bounded JSON structure in the normal tool result envelope.
- **Streaming (NDJSON via jobs)**: the tool returns `status: "pending"` with an envelope `job_id`, and the caller reads append-only NDJSON output via job tools (for example `job_read`).

The job tool surface is specified in [`docs/design/ToolContracts.md`](./ToolContracts.md).

Notes:

- All tool calls still use normal JSON framing on stdio. “Streaming” refers to the tool's result content being delivered as NDJSON items via job tooling.
- NDJSON items MUST be stable-ordered, and each line MUST be a complete JSON object.

## MCP compliance notes

- Compliance with the MCP specification is paramount.
- Alfred MUST NOT emit any out-of-band or non-MCP framing on stdio.
- “Streaming” in Alfred is achieved by sending additional MCP/JSON frames (each line is still a JSON value) and/or by using background job tooling (`status: "pending"` + `job_id`) and streaming NDJSON items within that tool surface.

## Redaction (secrets filtering)

Alfred MUST filter non-public information (secrets) from responses and logs, even if secrets are present in the incoming call.

- Alfred MUST NOT emit raw inbound request payloads into logs.
- Alfred MUST deterministically redact secret-looking values in both structured fields and free-form messages.
- If redaction occurs, Alfred SHOULD return a warning via `meta.warnings` rather than failing the call.

Redaction uses a stable replacement token and SHOULD record only aggregate redaction metadata (counts/booleans), never the secret itself.

The default replacement token is `<-REDACTED->`.

The deterministic redaction algorithm (detection + replacement-length fitting) is specified in [`docs/design/Redaction.md`](./Redaction.md).

Exception: tools whose primary purpose is to manage non-public values (for example, environment variable tools) MAY return unredacted values in the tool result `data`. These tools MUST still avoid emitting those values into tool/runtime logs.

## Structured log record (NDJSON)

Alfred emits NDJSON append only logs intended to be consumed by tools (for example via log tailing/filtering), including Alfred itself. Each NDJSON line MUST be a single JSON object with this shape:

- `timestamp` (string): RFC3339 (ISO 8601) UTC timestamp (seconds preferred; max milliseconds)
- `level` (string): `"TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"`
- `message` (string): human-readable log message (stable wording preferred)
- `source` (string): Rust-style source name (for example `alfred::configuration::load`)
- `extra` (object): additional structured fields (string-to-string pairs)

`extra` SHOULD carry any additional context (for example component ids, stable event identifiers, tool names, non-secret call parameters, and redaction metadata) without changing the top-level schema.

All log records MUST be safe to idempotently round-trip through Alfred tooling (i.e., deterministic JSON and already redacted).

## Determinism requirements

- Ordering MUST be stable (documented per tool).
- Pagination MUST be explicit; default limits MUST be documented per tool.
- Errors MUST include a deterministic `kind` and MUST NOT rely on free-form text for control flow.
