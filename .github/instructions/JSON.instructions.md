---
description: 'Agent directives for JSON formatting and linting.'
applyTo: '**/*.json'
---

# JSON Instructions

If present, the Prettier config (`.prettierrc.json`) is the source of truth for formatting.

## Formatting & Content Rules

- ALWAYS use `prettier` to format JSON files if available. Do not fight the formatter.
- Schemas, specifications, and tool requirements override these generic preferences.
- ALWAYS include a `$schema` when available.
    - Local schemas should use relative paths.
- ALWAYS validate the document against its declared schema, if present.
- ALWAYS comply with RFC 8259.
- ALWAYS encode as UTF-8 without BOM.
- ALWAYS include exactly one trailing newline.
- ALWAYS preserve primitive types.
    - Do not turn numbers/booleans into strings.
    - Use ISO 8601 for dates/times in UTC, unless the declared schema or consuming system requires another format.
- ALWAYS sort JSON by keys when generating it, and maintain the existing order when editing it.
