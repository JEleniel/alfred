# Alfred Protocol Notes

This document defines Alfred-specific protocol expectations on top of the Model Context Protocol (MCP) stdio transport.

Alfred is intended to be used via an MCP host (for example, an IDE) that:

- Launches Alfred as a local stdio process.
- Exchanges newline-delimited JSON frames.
- Calls tools via the MCP tool surface.

## Framing and encoding

- Transport is stdio.
- Each inbound request and outbound response is a single UTF-8 JSON value on its own line.
- Alfred MUST reject malformed frames deterministically.
- All protocol-visible strings MUST be valid UTF-8.
    - When interacting with filesystem paths that cannot be represented as valid Unicode text, Alfred MUST encode them deterministically before emitting JSON.

## Path strings

Unless a tool contract explicitly states otherwise, all `path` values in tool inputs and outputs:

- Are workspace-relative.
- Use POSIX separators (`/`) regardless of OS.
- MUST NOT be absolute.

If an incoming request provides a path using `\` separators, Alfred MUST treat them as `/` separators before normalization.

### Deterministic path transport encoding

All protocol path strings MUST be valid UTF-8 and MUST use percent-encoding for transport when encoding is required.

- `.` MUST remain literal.
- Path separator semantics remain native to path handling logic; protocol encoding only governs transport shape.
- Inputs containing unsafe or ambiguous path text MUST be rejected at the boundary rather than round-tripped.
- Rejected characters include `<`, `>`, `:`, `"`, `\`, `|`, `?`, `*`, `\0`, ASCII `0..=31`, Unicode control characters, and invalid Unicode values.
- `/` remains valid only as a path separator and MUST NOT appear inside encoded path components.
- Spaces are allowed.
- File-name components matching reserved Windows names (`CON`, `PRN`, `AUX`, `NUL`, `COM0..COM9`, `LPT0..LPT9`) MUST be rejected.
- File-name components ending in `.` MUST be rejected.

When path encoding occurs, Alfred SHOULD add a warning via top-level `warnings` (for example `{"kind":"path_encoded"}`) and MUST ensure ordering/cursors use the encoded representation consistently.

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

Optional top-level fields:

- `warnings`: array of warning objects
- `errors`: array of deterministic error objects (intended for command-level non-fatal errors)

Common metadata fields:

- `tool`: tool name
- `schema_version`: SemVer string for the tool result schema (lockstep with tool version)
- `duration_ms`: integer duration as observed by Alfred
- `transport_equivalent`: optional transport-equivalent metadata

### Failure

A failed tool result MUST be shaped as:

- `status`: `"error"`
- `errors`: non-empty array of deterministic error objects (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md))
- `meta`: common metadata

Optional top-level fields:

- `warnings`: array of warning objects

### Pending (accepted)

Only `fs` bulk execution MAY return `status: "pending"`.

When accepted for background execution, the response MUST include:

- `status`: `"pending"`
- `data.operation_id`: string
- `data.state`: `"queued" | "running"`
- `data.poll_with`: `"fs"`
- `meta.transport_equivalent.http_status`: `202`

Background status MUST be retrieved by calling `fs` with `operation: "bulk"` and `args.mode: "status"`.

## NDJSON usage

Alfred uses NDJSON for structured log persistence and exchange, but not for a standalone job-stream tool family.

- Log records are NDJSON append-only JSON objects.
- NDJSON items MUST be stable-ordered where ordering is contractually defined.
- Each NDJSON line MUST be a complete JSON object.

## MCP compliance notes

- Compliance with the MCP specification is paramount.
- Alfred MUST NOT emit any out-of-band or non-MCP framing on stdio.
- Background execution is represented through standard tool envelopes (`status: "pending"`) and subsequent tool calls.
- Streaming behavior follows the contract in [Streaming semantics](#streaming-semantics) below.

## Streaming semantics

Alfred supports streaming operations (currently `logs.follow`). Streaming complies fully with the MCP specification: Alfred emits only valid MCP frames; no raw NDJSON or non-MCP framing is emitted on stdio.

### Stream lifecycle

1. **Start**: the client invokes the tool normally. Alfred begins emitting a series of standard tool result envelopes.
2. **Intermediate emissions**: each intermediate emission MUST use `status: "ok"` and carry partial data as defined by the tool contract.
3. **Terminal emission**: the final emission signals end-of-stream by including `"stopped": true` in the `data` payload. Each streaming tool contract MUST document which `data` field carries this indicator.
4. **Error termination**: if the stream encounters a fatal error, Alfred MUST emit a terminal envelope with `status: "error"` and the standard error array. No further emissions follow.

### Stop request

The client may request stop by calling the same tool with the appropriate stop arguments (for example `logs` with `args.stop: true`). Alfred MUST:

- Stop emitting after acknowledging the request.
- Return a terminal envelope with `"stopped": true` as the final emission.

If no stream is active and a stop request is received, Alfred MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md), `stream_not_active`).

### Client requirements

- Clients MUST NOT assume a stream is terminated until they receive a terminal envelope (`data.stopped: true`) or a `status: "error"` envelope, or the transport closes.
- At most one stream may be active at a time. A new stream start request while a stream is active MUST fail deterministically (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md), `stream_active`).
- Streaming intent MUST be discoverable via `capabilities` as an `execution_modes` entry (`"stream"`).

## Redaction (non-public information)

Alfred MUST filter non-public information (NPI) from responses and logs, even if NPI is present in the incoming call.

- Alfred MUST NOT emit raw inbound request payloads into logs.
- Alfred MUST deterministically redact NPI in both structured fields and free-form messages.
- If redaction occurs, Alfred SHOULD return a warning via top-level `warnings` rather than failing the call.

Redaction uses a stable replacement token and SHOULD record only aggregate redaction metadata (counts/booleans), never the NPI itself.

The default replacement token is `<-REDACTED->`.

The deterministic redaction algorithm (detection + replacement-length fitting) is specified in [`docs/design/Redaction.md`](./Redaction.md).

## Structured log record (NDJSON)

Alfred emits NDJSON append-only logs intended to be consumed by tooling (for example via log search/tail operations). Each NDJSON line MUST be a single JSON object with this shape:

- `timestamp` (string): RFC3339 (ISO 8601) UTC timestamp (seconds preferred; max milliseconds)
- `level` (string): `"TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"`
- `message` (string): human-readable log message (stable wording preferred)
- `source` (string): Rust-style source name (for example `alfred::configuration::load`)
- `extra` (object): additional structured fields (string-to-string pairs)

`extra` SHOULD carry any additional context (for example component ids, stable event identifiers, tool names, non-NPI call parameters, and redaction metadata) without changing the top-level schema.

All log records MUST be safe to idempotently round-trip through Alfred tooling (that is deterministic JSON and already redacted).

## Determinism requirements

- Ordering MUST be stable (documented per tool).
- Pagination MUST be explicit; default limits MUST be documented per tool.
- Errors MUST include a deterministic `kind` and MUST NOT rely on free-form text for control flow.
