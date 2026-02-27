---
name: architecture
description: Guidelines for modeling architecture using the Aurora model.
---

# Architecture Skill

## When to use

Use this skill when you need to model and reason about the architecture of a system, project, or organization. The Aurora modeling system provides a structured way to capture architectural elements (like missions, drivers, requirements) and their relationships in a formalized model.

Details on the Aurora modeling system are in the [Aurora instructions](../../aurora/Aurora.instructions.md).

## General guidelines

- Use the full Aurora instructions when applying this skill.
- The Aurora Compact Model exists to save time and tokens by keeping key information in a single compact (thus the name) file. It is a fast reference, but it is not a substitute for the full Aurora instructions when producing or updating architecture.

## Goal

Provide an architecture that enables implementation without further interpretation.

## Deliverables

- One or more Aurora models and supporting documentation that capture the architecture of the project. To manage complexity in larger projects, multiple models may be used and may refer to each other.

## Scope

In modeling the project, consider the following aspects:

- System context
    - Define system boundary.
    - Identify external actors and dependencies.
    - State assumptions and invariants.
- Component model
    - Responsibility.
    - Inputs / outputs.
    - State ownership.
    - Scaling characteristics.
    - Failure impact.
- Data design
    - Authoritative data sources.
    - Data flow and transformation points.
    - Consistency model (strong, eventual, bounded).
    - Schema evolution strategy.
    - Data provenance and lineage requirements.
- Interfaces and contracts
    - Define APIs/events by purpose, not syntax.
    - Identify versioning and compatibility strategy.
    - State trust boundaries.
- Operational qualities
    - Availability model and degradation behavior.
    - Observability (what must be measurable).
    - Security controls mapped to risks.
    - Performance envelope and bottlenecks.
    - Deployment and rollback strategy.
- Threat and risk analysis
    - Failure modes:
        - What breaks.
        - How it is detected.
        - How it is contained.
        - Recovery path.
    - Threat models:
        - Diamond Model analysis.
        - What can be attacked.
        - How it is attacked.
        - What controls mitigate the attack.
        - Residual risk and monitoring strategy.
- ADRs, tradeoffs, and rejected alternatives.
- Testability requirements.

## Cross-skill tasks

- If the request includes implementation work (code changes), treat architecture modeling as its own phase first (produce/confirm the model and supporting documentation), then switch to the Coding skill for implementation.
- If the request is to review an existing architecture or model without changing it, use the Reviewing skill to record findings and recommendations.

## Validation

- The architecture is implementable without requiring additional interpretation (the stated goal).
- System boundary, external actors/dependencies, and assumptions/invariants are explicit.
- Components have clear responsibilities, interfaces/contracts, and state ownership.
- Data sources, transformation points, and consistency expectations are explicit.
- Threat and failure modes include detection and recovery paths.
- The model and supporting documentation are internally consistent.

## Glossary

See the shared [Skills glossary](../GLOSSARY.md).
