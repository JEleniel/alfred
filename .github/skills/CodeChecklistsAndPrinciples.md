# Code Review Checklists and Principles

## Approach

- **Security Review Mindset**: Review security with an adversarial mindset and identify realistic misuse and abuse paths.
- **OWASP-Aligned Security Checks**: Security findings should align with OWASP-style controls and trust-boundary enforcement.
- **Reliability of Failure Handling**: Failures must be handled intentionally; unhandled errors should be logged or returned cleanly.
- **Accessibility for User-Facing Behavior**: For user-facing surfaces, require WCAG AA at minimum (AAA preferred where feasible).

## Review Criteria

### Secure Code Checklist

- **All Input Untrusted**: All data coming into the system from any source is untrusted. Apply strict schemas, type checks, length limits, sanitization, and allow-lists at every trust boundary.
- **All Output Is Secured**: All untrusted data is contextually encoded before rendering or execution at each trust boundary. Encoding ensures data is interpreted strictly as data, never as executable instructions. Cryptographic signatures, hashes, and checksums are used to verify integrity where required.
- **Endpoints and APIs Authenticated, Authorized, and Auditable**: Any point of connection (e.g., APIs) is secured and requires authentication and authorization. A full audit trail, redacted of NPI, is kept for each.
- **Reuse, not Rewrite**: Use established, vetted implementations for security- and protocol-sensitive functionality. Do not reimplement standardized mechanisms.
- **Parameterize All Outside Calls and Queries**: Use prepared structures and statements and safe APIs for SQL, search, shell, template, and other external execution.
- **Protect Data End-to-End**: Data is stored securely and protected by access controls. Sensitive data is encrypted at rest and in transit. Exposure in memory is minimized in duration and scope. Encrypt in transit (TLS) and at rest; hash secrets with modern algorithms (bcrypt, Argon2); minimize retention. Minimize secret lifetime in memory. Zero buffers where possible. Prefer OS / HSM / keychain–managed secrets.
- **Ship with Secure Defaults**: Do not expose debugging interfaces or verbose diagnostics in production. Require explicit configuration, and apply least-privilege access from the start.
- **Handle Errors Without Information Leakage**: Return informative, actionable warning and error messages that have been sanitized of all Non-Public Information (NPI).
- **Continuously Maintain Dependencies**: Dependency inputs and resolved versions are pinned to guarantee 100% reproducible builds and prevent dependency drift across machines and time. Lockfiles are required, and dependency changes must be explicit and reviewed. Advisories are researched and mitigated, and automation maintains this state.
- **Log and Audit Security-Relevant Events**: Create structured, tamper-resistant logs for authentication, access decisions, and configuration changes.
- **Trust Boundaries Are Explicit**: All data transitions between trust zones are identified and enforced in code.
- **Minimal Surface Area**: The public interface is as small as possible for the capability delivered.

### Error and Exception Handling Checklist

- **Test Rigor**: Tests must prove behavior. "Null tests" (tests that do not meaningfully validate outcomes) are a failure.
- **Edge-Case Coverage**: Require tests for critical edge cases and failure paths, not only happy paths.

### General Code Quality Checklist

- **File Length**: Source files SHALL NOT exceed 500 lines. Exceeding this limit is a defect and MUST be corrected before merge.
- **Function Size**: Functions SHALL NOT exceed 50 lines. Exceeding this limit indicates improper decomposition and MUST be corrected before merge.
- **Reliability**: Failures are handled intentionally. Errors are logged or returned cleanly to the caller.
- **Accessibility**: User-facing behavior meets WCAG AA at minimum (AAA preferred where feasible).
- **Meaningful Tests**: Tests MUST validate outcomes and invariants. Null tests are defects.
- **Locality of Reasoning and the "Don't Repeat Yourself" (DRY) Principle**: Code is broken into logical components and modules. A reader can understand a unit without tracing the entire system. For any given capability, there exists a single source of truth implementation.
- **Single Responsibility Principle**: Each unit has a single, well-defined responsibility.
- **Correctness**: Behavior is verified against requirements through tests, invariants, and edge-case handling. If design documentation exists at `docs/design/` and/or an Aurora model is present, code aligns to the designs. If there is a divergence, the code or model MUST be updated to align with the other.
- **Consistency**: Naming, structure, and patterns are uniform across the codebase.
- **Explicitness**: Assumptions are avoided if possible. Assumptions, types, and effects are directly visible.
- **Low Coupling**: Components depend on minimal, stable interfaces. A change in one component should not require recompilation or modification of unrelated components.
- **Testability**: Behavior can be verified deterministically without elaborate setup.
- **Economy of Change**: Modifications require small, predictable edits rather than cascading rewrites.
- **Deterministic**: Given identical inputs and observable state, behavior is deterministic unless nondeterminism is explicitly modeled and documented (e.g., concurrency boundaries). Invalid states are unrepresentable or actively guarded against. Types, schemas, and invariants are designed such that invalid states cannot be constructed or are rejected immediately.
- **Constraint-Driven Design**: Limitations are used to shape the solution, producing focus rather than workaround.

## Principles of Elegance

- **Clarity**: Code is immediately understandable without needing external explanation.
- **Simplicity**: Solves the problem with the simplest, shortest complete answer.
- **Conceptual Compression**: A small set of well-chosen abstractions replaces large amounts of mechanics. Achieved through better modeling, not terseness.
- **Natural Mapping to the Domain**: The structure mirrors the problem space, not the implementation strategy.
- **Absence of Incidental Complexity**: Nothing exists solely to “make it work.” Every element carries semantic weight.
- **Composability**: Parts combine orthogonally without special cases or glue logic.
- **Predictability**: Behavior can be inferred from patterns already present; surprises are rare.
- **Symmetry**: Operations, data shapes, and error handling follow balanced, reusable forms.
- **Effortless Extendibility**: New features fit existing structures instead of requiring new mechanisms.

## Things to Watch For

### Failure and Risk Multipliers (a.k.a. "Foot-Guns")

- **Temporal Coupling**: Code must be called in a specific order not enforced by structure or types.
- **Boolean Parameter Explosion**: Functions controlled by flags (`do_x=true, fast=false`) rather than distinct behaviors.
- **Duplicated Logic**: The same behavior reimplemented instead of factored into a shared construct.
- **Magic Values and Hidden Rules**: Unexplained constants, implicit formats, or behavior encoded without naming.
- **Constant Explosion**: The number of hardcoded values or constants continues to grow.
- **Implicit Defaults**: Undocumented defaults, silent fallbacks, and assumed values (e.g., default timezones, encodings, or null behavior) that change meaning without visibility.
- **Mutable Shared State**: Multiple actors modifying the same data without strict ownership, leading to race conditions and heisenbugs.
- **Stringly-Typed Interfaces**: Using strings for structured data, keys, or commands that should be validated types.
- **Unchecked Error Paths**: Ignoring return values, exceptions, or partial failures that later manifest as corruption.
- **Configuration as Code Without Validation**: Loose JSON/YAML/TOML parsed into maps instead of validated schemas.
- **Global Initialization Side Effects**: Work performed at import/startup that changes behavior depending on load order.

### Indications of Poor Code or Modeling

- **God Objects / God Modules**: One unit knows too much or does too much, becoming a change hotspot.
- **Shotgun Surgery**: A single logical change requires edits scattered across many files.
- **Deeply Nested Logic**: Excessive indentation or branching indicates missing abstractions or guard clauses. No bouncing on the curly brace key required.
- **Primitive Obsession**: Using raw strings, ints, or maps where domain types should exist.
- **Leaky Abstractions**: Callers must understand internal details to use a component correctly.
- **Overgeneralization**: Infrastructure for hypothetical reuse that never materializes.
- **Hidden Complexity in “Convenience” APIs**: Helpers that appear simple but perform expensive or stateful work behind the scenes.
- **Heisencode**: Behavior varies based on hidden state, evaluation order, or side effects not expressed in the interface. Code whose output is non-deterministic but should be.
- **Silent Data Transformation**: Automatic coercions (precision loss, encoding changes, normalization) without explicit acknowledgement.

## Glossary

- Common review terms (for example "Pass", "Fail", "N/A", "Evidence", "Severity", and "P0"-"P3") are defined in the [Skills glossary](GLOSSARY.md).
- "Details": A concise statement of the specific condition that caused the review item to fail.
- "Risk": The expected negative outcome if the failed condition remains unresolved, including impact.
- "Smallest Safe Fix": The minimum change required to eliminate the failure without introducing new risk.
- "Verification Guidance": Explicit steps to prove that a fix works and that regressions were not introduced.
- "Trust Boundary": A point where data crosses between actors, systems, or components with different trust levels.
- "Dependency Drift": Unintended changes in dependency versions or resolution over time or across environments.
- "Reproducible Build": A build process that yields equivalent resolved dependencies and artifacts for the same inputs across machines and time.
- Unit: A unit is the smallest independently testable construct (function, type, class, or module, depending on language).
- Encoding: The process of transforming data so that it is safely interpreted in its target context. Encoded data is strictly treated as data, never as executable instructions. Encoding prevents injection and misinterpretation. The encoding must match the context in which the data will be interpreted; it does not replace signing, validation, or authentication.
- Error: A situation in which the expected state is not the actual state.
- Exception: A situation that could not be predicted and cannot be safely handled in code.
