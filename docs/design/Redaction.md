# Alfred Redaction

This document specifies Alfred’s deterministic redaction behavior for non-public information (NPI).

Redaction is applied:

- At index ingestion (redacted content MUST NOT be stored or searchable).
- To tool outputs (redacted content MUST NOT be emitted).
- To logs and job streams (redacted content MUST NOT be persisted or emitted).

Redaction is deterministic. The same inputs MUST produce the same outputs.

## Terminology

- NPI: non-public information. Includes secrets (tokens/keys/passwords) and may include user-configured PII/PHI-like patterns.
- Span: a half-open range `[start, end)` over a UTF-8 string in terms of Unicode scalar value indices (not bytes).

## Replacement token

The default replacement token is `<-REDACTED->`.

Alfred MUST support length-fitting replacement so callers that care about alignment (for example, diagnostics ranges, context snippets) can keep stable spacing.

### Token fitting algorithm

Given a redaction span with scalar-length `L`:

1. Start with base token `T = "<-REDACTED->"`.
2. If `len(T) == L`, replacement is `T`.
3. If `len(T) < L`, insert additional dashes immediately after the first `"<-"` to reach length `L`.
    - Example: `"<-REDACTED->"` → `"<----REDACTED->"`.
4. If `len(T) > L`, shrink the token deterministically:
    - Remove one character at a time from the tail of the substring `"DETCA"` in this order: `D`, then `E`, then `T`, then `C`, then `A`.
    - If the token is still too long, remove dashes from the token body (left-to-right).
    - If still too long, use the minimal fallback for very small `L`:
        - `L == 1`: `"*"`
        - `L == 2`: `"**"`
        - `L >= 3`: `"<" + ("-" repeated (L - 2)) + ">"`

All steps above are purely mechanical and MUST be implemented exactly so that redaction output is deterministic.

## Detection

Detection is rule-based. Alfred MUST run rule evaluation in a stable order and produce a stable set of non-overlapping spans.

### Rule classes

Alfred supports these rule classes (with user configuration to add/remove rules):

- Structured key/value rules (JSON-like or map-like content)
- Free-text regex rules

The default detector set is intentionally conservative: it targets common secret-bearing fields and obvious identifiers. It is not a guarantee of identifying all sensitive data.

### Default rules (baseline)

#### Structured key rules

If a structured field key matches any of the following case-insensitive patterns, the corresponding value MUST be redacted:

- `password`, `passwd`, `pwd`
- `secret`
- `token`
- `api_key`, `apikey`, `api-key`
- `authorization`
- `cookie`, `set-cookie`

#### Free-text regex rules

The following patterns are applied case-insensitively to free text and matched spans are redacted:

- Email address
- Common bearer token prefixes (for example `Bearer <token>`)

Exact regex definitions and additional PII/PHI-like patterns are configurable (see [`docs/design/Configuration.md`](./Configuration.md)).

### Span merge

1. Collect candidate spans in rule order.
2. Sort spans by `(start, end)`.
3. Merge overlaps by taking the union (earliest-start wins; merged span end is max end).
4. Apply replacement left-to-right.

## Metadata and warnings

When redaction occurs, Alfred SHOULD emit a warning in the tool result envelope (see [`docs/design/Protocol.md`](./Protocol.md)):

- `warnings += {"kind":"redaction","redacted_spans":<count>}`

Warnings MUST NOT include the original secret values.
