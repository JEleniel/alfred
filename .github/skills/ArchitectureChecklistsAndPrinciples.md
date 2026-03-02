# Architecture Review Checklists and Principles

## Approach

- **Security-First Architecture Mindset**: Evaluate architecture with an adversarial mindset and validate trust boundaries, privilege boundaries, and abuse resistance.
- **Model-to-Reality Focus**: Verify that architecture artifacts represent the implemented system and intended evolution.
- **Decision Traceability Focus**: Require explicit rationale, constraints, and trade-offs for architecture decisions.
- **Operability and Evolvability Focus**: Prioritize designs that are observable, resilient, and changeable without cascading rework.

## Review Criteria

### Architecture Coverage Checklist

- **Domain and Capability Coverage**: Core domains, bounded contexts, and major business capabilities are identified with clear ownership and boundaries.
- **Use-Case and Workflow Coverage**: Primary, secondary, and failure workflows are modeled end-to-end, including cross-domain interactions.
- **Component and Service Coverage**: All major components/services are represented, including responsibilities, interfaces, and lifecycle boundaries.
- **API and Contract Coverage**: External and internal contracts are specified with protocol, schema/versioning strategy, idempotency expectations, and error model.
- **Data Architecture Coverage**: Canonical entities, storage choices, consistency model, data ownership, retention, and lineage are explicit.
- **State and Event Coverage**: State transitions, event flows, ordering assumptions, replay behavior, and compensation/saga behavior are modeled where applicable.
- **Security and Trust Coverage**: Authentication, authorization, trust boundaries, secret handling, auditability, and abuse-case mitigations are mapped.
- **Reliability and Resilience Coverage**: Timeouts, retries, circuit-breaking, backpressure, degradation, failover, and recovery procedures are defined.
- **Performance and Scalability Coverage**: Capacity assumptions, performance budgets/SLOs, bottleneck risks, and scale-out strategy are specified.
- **Deployment and Runtime Coverage**: Topology, environment differences, configuration strategy, rollout strategy, rollback path, and compatibility windows are explicit.
- **Observability and Operations Coverage**: Required logs, metrics, traces, alerts, runbooks, and operational ownership are defined.
- **Dependency and Integration Coverage**: Third-party systems, upstream/downstream dependencies, failure contracts, and contingency behavior are captured.
- **Migration and Evolution Coverage**: Incremental migration plan, coexistence strategy, deprecation path, and backward/forward compatibility are documented.
- **Testing and Verification Coverage**: Architecture-level verification strategy includes contract, integration, resilience, performance, and security testing.
- **Implementation-Readiness Detail**: The model includes sufficient detail to implement without additional architecture decisions: interfaces, invariants, constraints, acceptance criteria, and decision rationale are explicit.

### Architecture Integrity Checklist

- **Boundaries Are Explicit**: Component, service, and data boundaries are clearly defined and consistently applied.
- **Responsibilities Are Cohesive**: Architectural units have clear responsibilities and avoid overlap and ambiguity.
- **Dependencies Are Intentional**: Dependency direction is explicit, stable, and aligned with layering constraints.
- **Coupling Is Controlled**: High coupling and hidden temporal dependencies are identified and justified or remediated.
- **Failure Domains Are Defined**: The architecture identifies isolation boundaries and blast-radius containment.
- **Data Flow Is Explicit**: Data movement between trust zones and architectural units is described and justified.
- **NFR Coverage Is Concrete**: Security, reliability, performance, and scalability requirements are mapped to architecture mechanisms.

### Aurora Model Fidelity Checklist

- **Aurora Coverage Is Complete**: Required Aurora model artifacts are present and coherent, conforming to the Aurora invariants.
- **Model-Artifact Alignment**: Aurora model content aligns with architecture documentation and implementation reality.
- **No Model Drift**: Divergence between model, documentation, and implementation is identified and addressed.
- **Constraint Traceability**: Architectural constraints expressed in the model are visible in design and implementation.
- **Decision-to-Model Traceability**: Significant architecture decisions are represented in Aurora artifacts and rationale.
- **Terminology Consistency**: Component names and architectural terms are consistent across model and documentation.

### External References and Link Integrity Checklist

- **Aurora External References Resolve**: External links embedded in Aurora artifacts resolve, are reachable, and point to the intended authoritative source. Web links MUST use `HTTPS`, and may not be directly verifiable due to security controls (corporate proxies, etc.).
- **Accompanying Document Links Resolve**: Local external links in architecture-adjacent documents resolve and point to the intended target.
- **Model-to-Document Link Consistency**: References cited in Aurora artifacts are consistent with references in accompanying architecture documents.
- **Stable and Auditable References**: References prefer stable/versioned targets (for example immutable specs, tagged docs, or permalinks) when available.
- **Reference Context Is Explicit**: Each external reference has sufficient surrounding context to explain why it is cited and how it constrains the architecture.

### Architecture Risk and Operability Checklist

- **Threat Surfaces Identified**: Entry points, trust boundaries, privileged flows, and abuse paths are documented.
- **Security Controls Mapped**: Authentication, authorization, validation, redaction, and audit controls are mapped to architectural elements.
- **Resilience Strategy Defined**: Retry, timeout, backpressure, degradation, and recovery strategies are explicit.
- **Observability by Design**: Logs, metrics, traces, and diagnostics are sufficient to operate and troubleshoot the architecture.
- **Deployment and Environment Assumptions Explicit**: Runtime assumptions, infrastructure constraints, and environment-specific behaviors are documented.
- **Change Safety**: The architecture enables incremental, reversible changes with bounded impact.
- **Migration and Compatibility Plan**: Versioning and migration paths are specified for architecture-affecting changes.

## Principles of Elegant Architecture

- **Clarity**: Architectural structure and intent are understandable without external explanation.
- **Simplicity**: The architecture solves requirements with the smallest complete set of mechanisms.
- **Composability**: Components compose orthogonally without special-case glue.
- **Predictability**: Similar concerns are handled with consistent patterns and constraints.
- **Traceability**: Decisions, constraints, and trade-offs are discoverable and auditable.
- **Minimal Incidental Complexity**: Mechanisms exist for domain value, not accidental implementation convenience.

## Things to Watch For

### Failure and Risk Multipliers (a.k.a. "Foot-Guns")

- **Implicit Boundaries**: Trust or component boundaries are assumed but not specified.
- **Architecture Drift**: Implementation evolves while architecture artifacts remain stale.
- **Undefined Failure Modes**: Error propagation and recovery behavior are not modeled.
- **Over-Coupled Components**: A change in one unit triggers widespread architecture churn.
- **Undocumented Privilege Paths**: Elevated operations and critical data flows are not explicit.
- **Hidden Shared State**: Shared mutable state creates non-deterministic behavior and brittle coordination.
- **NFR Hand-Waving**: Security, reliability, or scalability requirements are asserted without mechanisms.
- **One-Way Migration Assumptions**: Rollback, compatibility, or coexistence paths are missing.

### Indications of Poor Architecture or Modeling

- **God Components**: A single component owns too many unrelated responsibilities.
- **Leaky Layers**: Callers must understand internals across boundaries to use interfaces correctly.
- **Inconsistent Vocabulary**: Different names are used for the same architecture concepts.
- **Model-Implementation Mismatch**: Aurora artifacts and architecture docs disagree with observed behavior.
- **Unbounded Interfaces**: Public interfaces are broader than required capability.
- **Decision Orphans**: Important architecture decisions lack rationale or traceability.

## Glossary

- Common review terms (for example "Pass", "Fail", "N/A", "Evidence", "Severity", and "P0"-"P3") are defined in the [Skills glossary](GLOSSARY.md).
- "Architecture Drift": Divergence between architecture artifacts and implementation reality over time.
- "Aurora Model Fidelity": The degree to which Aurora artifacts accurately and consistently represent intended and implemented architecture.
- "Boundary": A defined separation of responsibility, authority, or trust between architectural units.
- "Failure Domain": A region of the system where failures can propagate without explicit containment.
- "Decision Traceability": The ability to trace architecture decisions to rationale, constraints, and impacted artifacts.
- "NFR": A non-functional requirement such as security, reliability, scalability, operability, or performance.
- "Blast Radius": The potential scope of impact from a fault, compromise, or change.
