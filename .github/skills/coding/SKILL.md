---
name: coding
description: Guidelines for writing code of any kind.
---

# Coding Skill

## When to use

Use this skill when the task involves writing or modifying source code.

## General guidelines

- If a `docs/design/aurora/` folder exists, read and follow [Aurora Compact Model](../../aurora/Aurora.compact.instructions.md) to understand the design.
    - The Aurora Compact Model exists to save time and tokens by keeping key information in a single compact (thus the name) file.
    - Use the full Aurora instructions only when applying the Architecture skill.
- Code review guidance is canonical. Follow [Code Review Checklists and Principles](../CodeChecklistsAndPrinciples.md). If there is a conflict, the checklist wins.
- The file-specific rules in `../../instructions/*.instructions.md` take precedence over these instructions.
- Follow best practices for the language being edited. Language-specific configs (for example `rustfmt.toml`, `.markdownlint-cli2.jsonc`, `.prettierrc.json`) are authoritative.
- Use the shortest acceptable path for local files.
- Prefer mature, well-supported dependencies with GPL, MIT, or Apache-2.0 licenses.
    - Well-maintained heuristic (use judgment; not a checklist):
        - Active and responsive maintenance (issues/PRs triaged; CI is healthy).
        - Clear compatibility story (MSRV/edition/features) that matches the workspace.
        - Security posture is solid (no known unfixed advisories; timely fixes when issues occur).
        - Adoption is meaningful (downstream usage and/or strong community reputation).
        - Documentation quality is good (README, examples, and changelog/release notes).
        - Exception: clearly stable/feature-complete crates MAY be acceptable with explicit justification.
- Dependency management:
    - Prefer the latest stable crate versions, unless constrained by compatibility, MSRV, or security response.
    - Add dependencies at the narrowest practical scope (package-level, not workspace-wide) unless multiple crates truly share them.
    - Avoid new dependencies when the standard library or existing dependencies already solve the problem.

## Invariants

- Apply file/function size constraints exactly as defined in [Code Review Checklists and Principles](../CodeChecklistsAndPrinciples.md).

- Unimplemented paths MUST fail fast and clearly communicate intent (`todo!`, `unimplemented!`, etc.).

## Errors and logging

- Prefer typed errors within libraries/modules.
- Add context at application boundaries.
- Avoid unchecked failures (`unwrap`, `expect`, panics) unless justified by an explicit invariant.
- You MUST NOT disable checks/tests (for example `// @ts-nocheck`, `#[allow(...)]`). Fix the underlying issue instead.
- Include helpful TRACE and DEBUG logging where appropriate for troubleshooting.
- Log at boundaries with appropriate severity.
- Never log secrets at any level.

## Deliverables

- Source code is modular, clean, readable, idiomatic, and aligned with project conventions.
- Appropriate unit and integration tests are added and passing.
    - Coverage target (aspirational): aim for 90%+ coverage on functional code when practical.
- Notes added for the documentation writer explaining changes, new features, and other relevant information for the project documentation.
- Linting, formatting, and static analysis checks are passing.

## Cross-skill tasks

- If the request is planning-only, do not produce code changes; use the Planning skill.
- If the request is documentation-only, do not modify code; use the Documentation skill.
- If the request includes both code and documentation updates, treat it as a coding task and include the documentation updates as part of the deliverables.
- If the request is to review existing changes, use the Reviewing skill to record findings; implement fixes only when explicitly asked.

## Validation

- The smallest intended change is implemented (no unrelated refactors).
- Unit and integration tests relevant to the change are added/updated and pass.
- Formatting and linting tools for the language/ecosystem pass (for Rust: `cargo fmt`, `cargo clippy`).
- The change does not introduce new panics/unchecked failures unless justified by explicit invariants.
- Logging is useful for troubleshooting and does not leak secrets.
- Security, reliability, and quality expectations remain compatible with [Code Review Checklists and Principles](../CodeChecklistsAndPrinciples.md).

## Glossary

See the shared [Skills glossary](../GLOSSARY.md).
