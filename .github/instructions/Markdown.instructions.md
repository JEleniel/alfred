---
description: 'Agent directives for Markdown formatting and linting.'
applyTo: '**/*.md'
---

# Markdown Instructions

If present, the repository's Markdown lint config (`.markdownlint-cli2.jsonc`) is the source of truth for formatting.

## Formatting & Content Rules

- ALWAYS use `markdownlint-cli2` if available. Do not fight the formatter.
- ALWAYS use only ATX headings (`#`, `##`, …) in title case.
- NEVER indent headings.
- ALWAYS use a single H1 as the document title.
- ALWAYS increase heading levels one at a time and do not skip levels.
    - Sibling headings must be unique.
- NEVER use emphasis/strong as the entire heading text.
- NEVER hard-wrap paragraph text.
- NEVER use extra spaces in prose. Tables may use spacing within cells for alignment.
- NEVER manually align Markdown with spaces outside tables.
- ALWAYS end files with exactly one trailing newline.
- NEVER add multiple blank lines together.
- ALWAYS use four spaces for indentation.
- ALWAYS use sequential numbering for ordered lists.
- NEVER continue numbering across breaks (such as headings).
- ALWAYS use fenced code blocks with backticks.
- ALWAYS include a language for all code blocks.
    - Use `text` when no specific language applies.
- When documenting a command, include sample output.
- NEVER use bare URLs.
- ALWAYS use Markdown links.
    - NEVER create reversed links.
    - NEVER create empty links.
- NEVER use inline HTML unless instructed.
- ALWAYS use leading and trailing pipe characters for table rows.
- ALWAYS ensure consistent column counts in every row.
- ALWAYS keep one blank line before and after tables, code blocks, callouts, and headings.
- ALWAYS use direct links `[]()`.
- NEVER use reference style links `[][]`.

## Frontmatter

- ALWAYS use `---` delimiters.
- NEVER include frontmatter unless required by the consuming tool.
