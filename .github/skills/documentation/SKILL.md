---
name: documentation
description: Instructions for an agent whose sole responsibility is to read existing artifacts and produce accurate, clear, and maintainable documentation.
---

# Documentation Skill

## When to use

Use this skill when the task involves:

- Writing or updating documentation only.
- Explaining existing code, systems, APIs, or architectures.
- Producing reference, conceptual, or procedural documentation.
- Summarizing behavior without modifying implementation.

This skill must **never** generate, modify, or suggest changes to source code.

## General guidelines

- If a `docs/design/aurora/` folder exists, read and follow [Aurora Compact Model](../../aurora/Aurora.compact.instructions.md) to understand the design.
    - The Aurora Compact Model exists to save time and tokens by keeping key information in a single compact (thus the name) file.
    - Use the full Aurora instructions only when applying the Architecture skill.

## Outputs

- Repository-level Markdown documentation files (e.g., `README.md`, `CONTRIBUTING.md`).
- User documentation in Markdown under `docs/`, starting with `docs/README.md`. This explicitly excludes `docs/design/`, which is design documentation.
- Structured reference material (tables, lists, sections).

## Documentation principles

- **Accuracy**: Document only what can be verified.
- **Clarity**: Prefer simple, direct language.
- **Stability awareness**: Distinguish stable interfaces from internal details.

## Deliverables

- Clear, concise, easily navigable documentation artifacts that reflect the current state of the system.
- Explanation of inputs, outputs, and constraints.
- Noted assumptions and limitations.
- Cross-references to relevant artifacts.

## Cross-skill tasks

- If the request requires source code changes (even small ones), do not proceed under this skill; switch to the Coding skill (or split the work into phases).
- If the request is plan-only (update `docs/design/ProjectPlan.md`), use the Planning skill.
- If the request is a formal review of documentation, use the Reviewing skill and record findings in the specified review files.

## Validation

- Documentation changes are accurate and verifiable from the repository content.
- Examples and procedures are complete (no missing prerequisites or steps).
- Links resolve and point to the intended artifacts.
- Markdown linting rules are satisfied (for example via `.markdownlint-cli2.jsonc`).

## Glossary

See the shared [Skills glossary](../GLOSSARY.md).
