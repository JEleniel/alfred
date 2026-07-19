---
description: 'Agent directives for YAML formatting and linting.'
applyTo: '**/*.yaml'
---

# YAML / YML Instructions

If present, the repository's Prettier config (`.prettierrc.json`) is the source of truth for whitespace formatting.

## Formatting & Content Rules

- ALWAYS use 2 spaces for indentation in YAML when creating files, and follow the existing indentation when editing.
- ALWAYS use `---` (three dashes) to separate multiple documents within a single file.
    - Do not include `---` in a single-document file unless it is required by the consuming tool.
    - The document-end marker `...` is optional and rarely required; avoid it unless needed.
- ALWAYS quote new strings, and do not change the quoting of existing strings unless changing them. Prefer single quotes unless the string contains single quotes, in which case use double quotes.
- ALWAYS use YAML booleans `true`/`false` (lowercase) and `null` for empty values. Avoid using `yes`/`no` unless required by a specific tool that expects them or they are the existing convention in the document.
- ALWAYS avoid creating or adding anchors (`&`) and aliases (`*`).
- NEVER store secrets or credentials in comments.
- ALWAYS preserve the existing key ordering when editing a file. When adding keys to a known manifest (e.g., GitHub Actions), follow commonly accepted semantic ordering for that manifest.
- ALWAYS ensure that YAML / YML documents are valid under strict parsing.
- ALWAYS save files as UTF-8 without BOM or their original format and ensure exactly one trailing newline at EOF. Use line feed (LF) only.

## Guidelines

- **Safe & Readable**: YAML documents should be human-readable, avoid unnecessary use of advanced YAML features (merge keys, complex tags, excessive anchors), and be robust to common parsers.
- Do not use explicit YAML tags (`!!python/object:...`) or custom tags unless the receiving application requires them.
- Use `#` comments to explain non-obvious choices.
