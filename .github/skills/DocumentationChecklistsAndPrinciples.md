# Documentation Review Checklists and Principles

## Approach

- **Accuracy-First Mindset**: Treat every claim as potentially wrong until verified against source-of-truth artifacts.
- **Task-Completion Focus**: Prioritize whether a reader can complete the intended task safely and correctly.
- **Security and Privacy Awareness**: Ensure guidance does not expose secrets, Non-Public Information, or unsafe operational patterns.
- **Accessibility for User-Facing Documentation**: Require WCAG AA at minimum (AAA preferred where feasible).

## Review Criteria

### Documentation Accuracy Checklist

- **Claims Are Verifiable**: Every technical claim can be verified against code, schemas, configuration, protocol contracts, or approved design artifacts.
- **API and Interface Fidelity**: Names, parameters, flags, defaults, return values, and error behaviors match implemented behavior.
- **Version and Scope Precision**: Version, platform, and environment scope are explicitly stated where behavior differs.
- **No Internal Contradictions**: Statements do not conflict across sections or with related documentation.
- **Examples Reflect Reality**: Examples, snippets, and sample outputs represent current behavior and constraints.

### Documentation Operability Checklist

- **Prerequisites Declared**: Required tools, permissions, credentials, environment variables, and setup steps are listed.
- **End-to-End Procedure Coverage**: Instructions cover setup, execution, validation, and cleanup where applicable.
- **Failure Paths Covered**: Common error conditions and recovery steps are documented, not only the happy path.
- **Input/Output Expectations**: Expected inputs, outputs, and side effects are clearly stated.
- **Upgrade and Migration Impact**: Breaking changes, migration steps, and compatibility notes are included when relevant.
- **Copy-Paste Safety**: Commands are safe by default and avoid hidden destructive behavior.
- **Destructive Action Warnings**: Irreversible or high-impact actions are clearly labeled with consequences.
- **Placeholders Are Explicit**: Placeholder values are clearly marked and cannot be mistaken for production values.
- **Sample Output Included When Needed**: Command-focused procedures include representative output for validation.
- **No Secret Leakage**: Examples, logs, and screenshots do not expose secrets or Non-Public Information.
- **Links Resolve Correctly**: Internal and external links resolve to intended targets.
- **Reference Targets Are Specific**: Links point to precise sections/files when practical.
- **No Empty or Misleading Links**: Link text accurately reflects destination content.
- **Stable Cross-References**: References are resilient to routine file movement and repository evolution.
- **Evidence-Backed References**: Critical references cite source artifacts that can be audited.

### Documentation Clarity and Accessibility Checklist

- **Clear Structure**: Heading hierarchy is logical and supports scanning.
- **Concise Wording**: Content is direct, avoids redundancy, and minimizes ambiguity.
- **Consistent Terminology**: The same concept uses the same term throughout the document set.
- **Actionable Steps**: Instructions are imperative, sequenced, and testable by a reviewer.
- **Context Before Action**: Readers are given enough context to understand risk and intent before executing steps.
- **Descriptive Link Text**: Link text is meaningful out of context.
- **Inclusive Language**: Language avoids exclusionary, ambiguous, or culturally narrow phrasing.
- **Readable Formatting**: Lists, tables, and callouts are structured for assistive technology compatibility.
- **Non-Visual Context**: Visual-only cues are supplemented with textual context.
- **Keyboard and Screen Reader Usability**: Documentation structure supports keyboard and screen reader navigation.

## Principles of Elegant Documentation

- **Clarity**: Readers can understand intent and action without external explanation.
- **Simplicity**: The document solves the reader's need with the shortest complete path.
- **Traceability**: Every key claim can be traced to a verifiable source of truth.
- **Composability**: Sections can be reused and linked without duplicating or fragmenting meaning.
- **Predictability**: Similar tasks are documented in similar patterns.
- **Minimal Incidental Complexity**: Content includes only what is necessary to execute safely and correctly.

## Things to Watch For

### Failure and Confusion Multipliers (a.k.a. "Foot-Guns")

- **Hidden Prerequisites**: Critical setup requirements are implied but not stated.
- **Version Ambiguity**: Instructions omit version/platform assumptions.
- **Unsafe Defaults in Examples**: Samples normalize insecure or overly privileged behavior.
- **Happy-Path Only Procedures**: Missing troubleshooting makes real-world execution brittle.
- **Silent Destructive Steps**: Commands with destructive effects are not labeled or explained.
- **Documentation Drift**: Content diverges from implementation over time.
- **Copy-Paste Hazards**: Placeholders or environment-specific values are not clearly marked.
- **Reference Rot**: Links and references are outdated, broken, or non-specific.

### Indications of Poor Documentation

- **Contradictory Guidance**: Different sections recommend incompatible actions.
- **Orphaned Documents**: Documents lack ownership, update cadence, or clear entry points.
- **Unbounded Scope**: A single document attempts to cover unrelated concerns.
- **Terminology Drift**: Multiple names for the same concept increase cognitive load.
- **Non-Deterministic Instructions**: Steps depend on unstated context or implicit state.
- **Leaky Internal Detail**: Readers must infer implementation internals to follow instructions.

## Glossary

- Common review terms (for example "Pass", "Fail", "N/A", "Severity", and "P0"-"P3") are defined in the [Skills glossary](GLOSSARY.md).
- "Source of Truth": The authoritative artifact used to verify a claim (for example code, schema, or protocol contract).
- "Documentation Drift": Divergence between documentation and implemented behavior.
- "Prerequisite": A requirement that must be satisfied before executing a procedure.
- "Happy Path": The expected successful flow under normal conditions.
- "Failure Path": The documented behavior and recovery steps when expected execution fails.
- "Copy-Paste Safe": Content that can be executed as written without causing unintended risk in normal environments.
