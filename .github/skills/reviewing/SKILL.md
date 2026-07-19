---
name: reviewing
description: Use this skill when conducting an informal or formal review (not analysis) of code, documentation, architecture, or pre-release artifacts.
---

# Reviewing Skill

## General guidelines

- Evaluate reviews with an adversarial mindset. Validate trust boundaries, abuse resistance, and failure modes relevant to the requested scope.
- Verify that review findings reflect observable evidence from repository artifacts or verified behavior.
- Record explicit rationale, constraints, assumptions, and trade-offs behind severity and recommendations.
- Prioritize outputs that are auditable, risk-proportionate, and immediately actionable.
- If a `docs/design/aurora/` folder exists, you may read and follow [Aurora Compact Model](../../aurora/Aurora.compact.instructions.md) to understand the design.
- Use the full Aurora instructions only when applying the Architecture skill.
- In order to ensure enough context for the review, break it down into manageable chunks and/or delegate it to subagents as needed. You may also ask for additional information or clarification from the user if necessary.
- If the user asks for a "formal review", treat it as a formal review and write the results to the appropriate files. Otherwise, provide feedback and recommendations in the conversation without writing to the review files.
- If unsure whether it is formal or not, ask.
- You MUST NOT alter code, documentation, or other files except the review output files.
- Reviews recommend improvements and fixes. Reviews do not make decisions and do not record decisions.

## Principles of Effective Review

- Keep review structure and intent understandable without external explanation.
- Solve the review objective with the smallest complete set of evidence-backed findings.
- Compose findings orthogonally without duplicate or overlapping claims.
- Handle similar concerns with consistent severity criteria and verification guidance.
- Keep evidence, constraints, and rationale discoverable and auditable.
- Keep findings focused on risk, readiness, and user or system impact.

### Indications of Poor Review

- Unsupported Findings: Claims are not traceable to observable evidence.
- Scope Creep: Review drifts beyond requested boundaries.
- Severity Inflation: Impact and likelihood are overstated.
- Missing Verification Guidance: Fixes cannot be confidently validated.
- Advisory-as-Decision: Recommendations are presented as binding decisions.
- Checklist Cargo Culting: Boxes are checked without meaningful evaluation.

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

- Root formal review artifacts under `docs/design/analysis/`.
- Keep review files current as scope and findings evolve.
- Record findings in the appropriate review file for the requested review type:
    - `docs/design/analysis/Review-Architecture.md`
    - `docs/design/analysis/Review-Code.md`
    - `docs/design/analysis/Review-Documentation.md`
    - `docs/design/analysis/Review-Prerelease.md`
    - `docs/design/analysis/Review-Security.md`
- Apply the canonical checklist for the review type and record findings according to checklist requirements.
- Ensure each finding includes evidence, risk/severity, and verification guidance.
- For informal reviews, deliver findings in conversation only and do not write review files.

## Operating Procedure

1. Confirm scope, review type, stakeholders, constraints, and whether the request is formal or informal.
2. Inspect relevant artifacts needed to evaluate the requested scope without drifting into unrelated areas.
3. Update or create review outputs in place, using the appropriate formal review file when required.
4. Capture or update assumptions, constraints, risk posture, and severity rationale.
5. Apply review criteria for the artifact type and produce evidence-backed findings.
6. Record findings with issue statement, impact, severity, and smallest-safe recommendation.
7. Include explicit verification guidance for each finding so fixes can be validated objectively.
8. Keep terminology, identifiers, and references consistent across findings and source artifacts.
9. For informal reviews, provide findings in conversation and do not write review files.
10. Validate the result against this skill's deliverables and checklists before completion.

## Validation Checklists

### Review Process Checklist

- Correct review scope and review type.
- Context is sufficient to support every finding.
- Formal versus informal output is handled correctly.
- Findings are recorded in the correct location for formal reviews.

### Finding Quality Checklist

- Evidence-backed findings.
- Severity proportionate to risk and impact.
- Clear explanation of why each finding matters.
- Smallest-safe recommendations.
- Explicit, objective verification guidance.

### Output Quality Checklist

- Findings are grouped and presented clearly.
- No implementation changes outside allowed review output files.
- No decisions recorded under the guise of review findings.
- No unrelated scope drift.

### Review Domain Coverage Checklist

- Scope and boundaries
- Evidence and traceability
- Risk and severity consistency
- Security and trust implications
- Reliability and resilience impact
- Performance and scalability impact
- Operability and observability impact
- Dependency and integration risks
- Testability and verification quality
- Documentation and usability impact

### Reference and Link Integrity Checklist

- Resolvable references to repository artifacts.
- Consistent references between findings and evidence.
- Explicit evidence context for each finding.
- Useful, specific references (not generic pointers).
- Stable links for external references when used.

## Things to Watch For

### Foot-Guns

- Findings without evidence.
- Severity inflation.
- Unclear scope boundaries.
- Recommendations too vague to act on.
- Reviews that silently become implementation work.
- Missing verification guidance.
- Multiple paths for the same functionality.
- Oversized files or functions.
- Small functions that are null functions, thin wrappers over a constant, error type mappers, or otherwise add no value.

## Cross-skill tasks

- If the request includes implementation work, complete review as its own phase first.
- If asked to implement fixes, switch to the appropriate skill after review output is complete.
- If asked to update a project plan, use the Planning skill.
- If asked to produce or update an Aurora architecture model, use the Architecture skill.
- If asked to produce documentation or code changes rather than review findings, switch to the appropriate skill for that work.
