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
- If the user asks for "feedback", treat it as informal feedback (not a formal review) and provide your feedback in your response.
- If the user asks for a "review", treat it as a formal review and write the results to the appropriate files.
- If unsure whether it is formal or not, ask.
- You MUST NOT alter code, documentation, or other files except the review output files.
- Reviews recommend improvements and fixes. Reviews do not make decisions and do not record decisions.

## Review principles

- Correctness: code matches intent and wiring is complete.
- Reliability: failures are handled intentionally; unhandled errors are logged and/or returned cleanly to the caller.
- Security: apply OWASP guidance (input validation, authz/authn boundaries, secrets handling, least privilege, safe logging).
- Accessibility: WCAG AA (AAA preferred).
- Maintainability: clear names; proper, cohesive modules; minimal complexity.
- Tests: require tests that prove behavior; no null tests.
- Code conciseness: prefer small functions (~50 lines) and cohesive modules (~500 lines) when practical.
    - For newly written or substantially rewritten code, avoid new functions > 50 lines and new files > 500 lines.
- Documentation conciseness: clear, to the point, easily readable documentation.
- Compliance: if an Aurora model is present, code aligns to the model. If there is a divergence, the code or model should be updated to align with the other.
- Gaps and 'foot-guns': identify missing edge cases, potential for misuse, and areas where future maintainers may struggle.

## Specific points of review

- Be very strict about security issues. Approach code reviews with an adversarial mindset, looking for ways the code could be misused or abused.
- Verify test coverage is sufficient to prove the behavior of the code. Look for null tests, test gaps, and missing tests for edge cases.
- For documentation, verify it is clear, concise, and accurate. Look for missing information, inaccuracies, and areas that could be clarified or simplified.
- For prerelease reviews, verify all release criteria are met, including documentation, testing, and any necessary approvals.

## Files and folders to ignore

- Read only the `Compact.json` files under `docs/design/aurora/` for context on the Aurora model. Do not read any other files in that directory.
- Do not read any files or folders under `docs/design/aurora/` named `MIS-*`. Those are just the human-readable version of the model and contain no relevant information not already in the Compact Model.
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
    - `docs/design/Review-Code.md`
    - `docs/design/Review-Security.md`
    - `docs/design/Review-Documentation.md`
    - `docs/design/Review-Prerelease.md`
- Group findings by severity and include verification guidance.
- Provide actionable feedback: what, why it matters, and the smallest safe fix.

## Cross-skill tasks

- If asked to implement fixes, do not proceed under this skill. Either:
    - Switch to the Coding skill (if implementation is desired and allowed), or
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
