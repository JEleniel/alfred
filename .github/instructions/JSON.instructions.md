---
description: 'Agent directives for JSON formatting and linting.'
applyTo: '*.json'
---

# JSON Formatting & Linting

If present, the repository's Prettier config (`.prettierrc.json`) is the source of truth for formatting.

## Formatting & Content Rules

- **Include `$schema` when available**: If a schema exists for the file, add `$schema` with the correct URL. If no repo schema is known, prefer SchemaStore URLs (for example `https://json.schemastore.org/prettierrc`).
- **Best-effort schema validation**: When `$schema` is present (or a well-known schema applies), validate if feasible. Do not block changes solely because validation cannot be performed (offline, private schema, etc). Note the outcome if validation fails or is not possible.
- **No comments**: `.json` must be valid JSON.
- **Quotes**: Use double quotes for keys and strings.
- **Types**: Preserve primitive types; do not turn numbers/booleans into strings.
- **Encoding**: UTF-8 without BOM.
- **EOF**: Exactly one trailing newline.
- **Generated/lock files**: Avoid manual edits to generated artifacts and lockfiles. If you must, document why and validate the change.
- **Consistent Ordering**: Sort JSON by keys when generating it, maintain the existing order when editing.
- **Prettier**: Instead of wasting time formatting JSON, use `prettier` when available (and it is in the IDE).
