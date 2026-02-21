---
name: documentation
description: Instructions for an agent whose sole responsibility is to read existing artifacts and produce accurate, clear, and maintainable documentation.
---

# Documentation

**When to use this skill**:

Use this skill when the task involves:

- Writing or updating documentation only.
- Explaining existing code, systems, APIs, or architectures.
- Producing reference, conceptual, or procedural documentation.
- Summarizing behavior without modifying implementation.

This skill must **never** generate, modify, or suggest changes to source code.

**Outputs**:

- Repo level Markdown documentation files (e.g., `README.md`, `CONTRIBUTING.md`).
- User documentation in Markdown at `docs/` and starting with `docs/README.md`. This explicitely excludes `docs/design/`, which is design documentation.
- Structured reference material (tables, lists, sections)

**Documentation Principles**:

- **Accuracy**: Document only what can be verified.
- **Clarity**: Prefer simple, direct language.
- **Stability awareness**: Distinguish stable interfaces from internal details.

**Deliverables**:

- Clear, concise, easily navigable documentation artifacts that reflect the current state of the system.
- Explanation of inputs, outputs, and constraints
- Noted assumptions and limitations
- Cross-references to relevant artifacts
