# Alfred Redaction

This document specifies Alfred’s deterministic redaction behavior for non-public information (NPI).

Redaction is applied:

- At index ingestion (redacted content MUST NOT be stored or searchable).
- To tool outputs (redacted content MUST NOT be emitted).
- To logs and job streams (redacted content MUST NOT be persisted or emitted).

Operational path logging policy:

- Absolute paths that can disclose host user names or home-directory segments MUST NOT be logged verbatim.
- Logs SHOULD use workspace-relative paths or short path tails that do not reveal non-public host identity details.

Redaction is deterministic. The same inputs MUST produce the same outputs.

## Terminology

- NPI: non-public information. Includes NPI-looking values (tokens/keys/passwords) and may include user-configured PII/PHI-like patterns.
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

If the **innermost field name** in a key path matches any of the following case-insensitive patterns, the corresponding value MUST be redacted. "Innermost" means the last segment of a dotted or nested key (for example, in `config.db.password` the innermost name is `password`, which matches):

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

## Test vectors

The following examples are normative. Implementations MUST produce these exact outputs.

### Token fitting vectors

`<-REDACTED->` has length 13. All lengths are Unicode scalar value counts.

| L   | Output token     | Method                                                            |
| --- | ---------------- | ----------------------------------------------------------------- |
| 1   | `*`              | Minimal fallback.                                                 |
| 2   | `**`             | Minimal fallback.                                                 |
| 3   | `<->`            | Minimal fallback: `<` + 1 dash + `>`.                             |
| 4   | `<-->`           | Minimal fallback: `<` + 2 dashes + `>`.                           |
| 5   | `<RED>`          | Body dashes removed, then `ACTE` removed from tail; `<RED>` fits. |
| 6   | `<RED->`         | Left body dash removed; `<RED->` fits.                            |
| 8   | `<-RED->`        | All 5 chars of `DETCA` removed from tail; `<-RED->` fits.         |
| 9   | `<-REDA->`       | 4 chars (`CTED`) removed from tail.                               |
| 10  | `<-REDAC->`      | 3 chars (`TED`) removed from tail.                                |
| 11  | `<-REDACT->`     | 2 chars (`ED`) removed from tail.                                 |
| 12  | `<-REDACTE->`    | 1 char (`D`) removed from tail.                                   |
| 13  | `<-REDACTED->`   | Exact match; no change.                                           |
| 14  | `<--REDACTED->`  | 1 extra dash inserted after `<-`.                                 |
| 15  | `<---REDACTED->` | 2 extra dashes inserted after `<-`.                               |

### Detection vectors

#### Structured key — top-level

Input:

```json
{ "api_key": "sk-abc123XYZ" }
```

Output (with `preserve_length: false`):

```json
{ "api_key": "<-REDACTED->" }
```

The value `"sk-abc123XYZ"` (length 12) is redacted. With `preserve_length: true` the replacement token is length-fitted to 12: `<-REDACTE->`.

#### Structured key — nested (innermost field rule)

Input:

```json
{ "database": { "host": "localhost", "password": "hunter2" } }
```

The innermost key `password` matches the rule. `host` does not match.

Output (preserve_length: true, `"hunter2"` has length 7):

```json
{ "database": { "host": "localhost", "password": "<-REDA->" } }
```

#### Free-text — bearer token

Input string:

```text
Authorization: Bearer ghp_AbCdEfGhIjKlMnOpQrStUvWxYz012345
```

The `Bearer <token>` pattern matches. The span covering the token value is redacted.

#### Free-text — email address

Input string:

```text
Contact alice@example.com for support.
```

Output (preserve_length: true, `alice@example.com` has length 17 → fitted token `<------REDACTED->`):

```text
Contact <------REDACTED-> for support.
```

### Span merge vector

Input string: `"password": "secret123"` (parsed as structured content)

- Rule 1 (structured key): matches `password`, span covers `secret123`.
- Rule 2 (free-text regex): suppose a regex also matches `secret` within `secret123`, producing a sub-span inside Rule 1's span.

After merge: the overlapping spans are unioned; only one replacement is emitted covering the full `secret123` span.

## Metadata and warnings

When redaction occurs, Alfred SHOULD emit a warning in the tool result envelope (see [`docs/design/Protocol.md`](./Protocol.md)):

- `warnings += {"kind":"redaction","redacted_spans":<count>}`

Warnings MUST NOT include the original secret values.
