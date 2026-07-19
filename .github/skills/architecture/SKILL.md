---
name: architecture
description: Use this skill when creating or updating the architecture of a system, project, or organization, including Aurora models and supporting design artifacts.
---

# Architecture Skill

## General guidelines

- Evaluate architecture with an adversarial mindset. Validate trust boundaries, privilege boundaries, and abuse resistance.
- Verify that architecture artifacts represent the implemented system and intended evolution.
- Record explicit rationale, constraints, and trade-offs for architecture decisions.
- Prioritize designs that are observable, resilient, and changeable without cascading rework.

## Principles of Elegant Architecture

- Keep structure and intent understandable without external explanation.
- Solve requirements with the smallest complete set of mechanisms.
- Compose components orthogonally without special-case glue.
- Handle similar concerns with consistent patterns and constraints.
- Keep decisions, constraints, and trade-offs discoverable and auditable.
- Keep mechanisms focused on domain value, not implementation convenience.

### Indications of Poor Architecture or Modeling

- God Components: A single component owns too many unrelated responsibilities.
- Leaky Layers: Callers must understand internals across boundaries to use interfaces correctly.
- Inconsistent Vocabulary: Different names are used for the same architecture concepts.
- Model-Implementation Mismatch: Aurora artifacts and architecture docs disagree with observed behavior.
- Unbounded Interfaces: Public interfaces are broader than required capability.
- Decision Orphans: Important architecture decisions lack rationale or traceability.

## Aurora

- Use the full [Aurora instructions](../../aurora/Aurora.instructions.md).
- Do not use the Aurora compact model when using this skill.

### Aurora Model Fidelity Checklist

- Aurora Coverage Is Complete
- No Model Drift
- Constraint Traceability
- Decision-to-Model Traceability
- Terminology Consistency

## Deliverables

- Root the architecture in `docs/design/Requirements.md`.
- Keep `docs/design/Requirements.md` current as requirements and constraints evolve.
- Produce Aurora model(s) and supporting documentation with enough detail to guide implementation.
- Use multiple models only when they reduce complexity and remain clearly related.
- Capture architectural decisions, trade-offs, and rationale in supporting documentation.
- Include ADRs, hardware specifications, and other reference material when needed to justify the model.
- Store Aurora models under `docs/design/aurora/`, following the standard Aurora structure and naming conventions.
- Store supporting documentation under `docs/design/`, organized into appropriate subdirectories (for example, `docs/design/adrs/` for Architectural Decision Records).

## Operating Procedure

1. Confirm scope, stakeholders, constraints, and success criteria from `docs/design/Requirements.md` and the user request. If critical information is missing, stop and get clarification.
2. Inspect existing artifacts under `docs/design/` and `docs/design/aurora/`.
3. Update artifacts in place unless a new architectural boundary requires a separate model.
4. Capture or update assumptions, constraints, external dependencies, trust boundaries, and key decisions in supporting documentation.
5. Create or update Aurora models under `docs/design/aurora/` using the full Aurora instructions.
6. Prefer canonical card types and relationships. Use non-canonical elements only when no canonical option is semantically correct and the user agrees.
7. Keep terminology, identifiers, and references consistent across requirements, supporting docs, and Aurora artifacts.
8. Include external references when they materially support the architecture, and link them to relevant model elements.
9. Validate the result against this skill's deliverables and checklists before completion.
10. If the request includes implementation work, complete architecture first, then switch to the Coding skill.

## Validation Checklists

### General Architecture Checklist

- ADRs, trade-offs, and rejected alternatives captured.
- Authoritative and non-authoritative data sources identified.
- Components decomposed with clear responsibilities, separation of concerns, and isolated state.
- Clearly stated assumptions and invariants.
- Data provenance and lineage requirements captured.
- Defined versioning and compatibility strategy.
- Intentional dependencies.
- Planned deployment and rollback strategy.
- Documented inputs and outputs.
- Failure modes and remediation analysis completed.
- Identified stakeholders and external actors.
- Loosely coupled interdependencies with clear boundaries and contracts.
- Modeled data flows and transformation points.
- Concrete NFR coverage.
- Defined performance, availability, and degradation expectations.
- Planned horizontal and vertical scaling characteristics.
- Planned observability (what must be measurable).
- Purpose-driven APIs.
- Risks, threat models, security controls, and mitigations captured.
- System evolution strategy.
- Testability requirements.
- Traceable ownership and responsibility.
- Well-defined system, domain, and trust boundaries.

### Architectural Domain Coverage Checklist

- Domains and Capabilities
- Use-Case and Workflow
- Components, Interfaces, and Services
- APIs and Contracts
- Data Architecture
- State Machines and Events
- Security and Trust
- Reliability and Resilience
- Performance and Scalability
- Deployment and Runtime
- Observability and Operations
- Dependencies and Integration
- Migration and Evolution
- Testing and Verification

### External References and Link Integrity Checklist

- Resolvable Aurora external references.
- Resolvable accompanying document links.
- Consistent model-to-document links.
- Explicit reference context.
- Clear authoritative references.
- Specific, useful references.
- Stable reference links outside the workspace.

## Cross-skill tasks

- If the request includes implementation work, complete architecture modeling as its own phase first.
- Produce or confirm the model and supporting documentation before switching skills.
- Switch to the Coding skill only after the architecture phase is complete.
- If the request is to review an existing architecture or model without changing it, use the Reviewing skill.
