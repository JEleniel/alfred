---
name: reviewing
description: The skill of performing thorough reviews of code, documentation, and releases to ensure quality, security, and readiness.
---

# Reviewing

- If the user asks for feedback, treat it as informal feedback (not a formal review).
    - Provide feedback inline in your response.
    - Do not write to the review output files unless the user explicitly asks for a formal review.
- When performing a formal review, do not alter code, documentation, or other files except the review output file.

**Review Principles**:

- Correctness: code matches intent and wiring is complete.
- Reliability: failures are handled intentionally, unhandled errors are logged and/or returned cleanly to the caller.
- Security: apply OWASP guidance (input validation, authz/authn boundaries, secrets handling, least privilege, safe logging).
- Accessibility: WCAG AA (AAA preferred).
- Maintainability: clear names; proper, cohesive modules; minimal complexity.
- Tests: require tests that prove behavior; no null tests.
- Code conciseness: prefer small functions (~20 lines) and cohesive modules (~200 lines) when practical.
    - For newly written or substantially rewritten code, avoid new functions > 50 lines and new files > 500 lines.
- Documentation conciseness: clear, to the point, easily readable documentation

**Deliverables**:

- Record findings in the appropriate review file under `docs/design/`:
    - `docs/design/Review-Code.md`
    - `docs/design/Review-Security.md`
    - `docs/design/Review-Documentation.md`
    - `docs/design/Review-Prerelease.md`
- Group findings by severity and include verification guidance.
- Provide actionable feedback: what, why it matters, and the smallest safe fix.
