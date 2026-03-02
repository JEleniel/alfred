---
name: reviewing
description: The skill of performing thorough reviews of code, documentation, and releases to ensure quality, security, and readiness.
---

# Reviewing Skill

## General guidelines

- If a `docs/design/aurora/` folder exists, read and follow [Aurora Compact Model](../../aurora/Aurora.compact.instructions.md) to understand the design.
    - The Aurora Compact Model exists to save time and tokens by keeping key information in a single compact (thus the name) file.
    - Use the full Aurora instructions only when applying the Architecture skill.
- In order to ensure enough context for the review, break it down into manageable chunks and/or delegate it to subagents as needed. You may also ask for additional information or clarification from the user if necessary.
- If the user asks for a "formal review", treat it as a formal review and write the results to the appropriate files. Otherwise, provide recommendations in the conversation without writing to the review files.
- If unsure whether it is formal or not, ask.
- You MUST NOT alter code, documentation, or other files except the review output files.
- Reviews recommend improvements and fixes. Reviews do not make decisions and do not record decisions.

## Review principles

- If reviewing code, refer to the [Code Review Checklists and Principles](../CodeChecklistsAndPrinciples.md) for comprehensive guidance on what to look for. This includes security review guidance.
- If reviewing documentation, refer to the [Documentation Review Checklists and Principles](../DocumentationChecklistsAndPrinciples.md) for comprehensive guidance on what to look for.
- If reviewing architecture and designs, refer to the [Architecture Review Checklists and Principles](../ArchitectureChecklistsAndPrinciples.md) for comprehensive guidance on what to look for.
- For a pre-release review, refer to the [Pre-release Review Checklists](../PreReleaseChecklists.md) for comprehensive guidance on what to look for.
- The checklist documents are canonical. If a checklist and a skill instruction conflict, the checklist wins.

## Files and folders to ignore

- Read only the `Compact.json` files under `docs/design/aurora/` for context on the Aurora model. Do not read any other files in that directory for reviews.
- Do not read any files or folders under `docs/design/` named `MIS-*`. Those are just the human-readable version of the model and contain no relevant information not already in the Compact Model.
- Do not read files under `.github/` unless specifically instructed to review them.
- The following files are irrelevant for reviews:
    - `.gitignore`
    - `.gitattributes`
    - `*.lock`
    - `.editorconfig`
    - `rustfmt.toml`
    - `rust-toolchain.toml`
    - `assets/`

## Deliverables

- Record findings in the appropriate review file under `docs/design/`:
    - `docs/design/Review-Architecture.md`
    - `docs/design/Review-Code.md`
    - `docs/design/Review-Documentation.md`
    - `docs/design/Review-Prerelease.md`
    - `docs/design/Review-Security.md`
- Apply the canonical checklist for the review type and record findings according to checklist requirements.

## Cross-skill tasks

- If asked to implement fixes, do not proceed under this skill. Either:
    - Switch to the appropriate skill, or
    - Provide recommendations only and leave implementation to a separate task.
- If asked to update a project plan, use the Planning skill.
- If asked to produce or update an Aurora architecture model, use the Architecture skill.

## Validation

- Findings are recorded in the correct file(s) under `docs/design/` as specified above.
- Findings are grouped by severity and include clear verification guidance.
- Recommendations are concrete and smallest-safe.
- The review output does not include implementation changes outside the review files.

## Glossary

See the shared [Skills glossary](../GLOSSARY.md).
