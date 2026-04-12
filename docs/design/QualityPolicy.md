# Quality Policy

This document is the canonical quality and maintainability policy for Alfred design and implementation artifacts.

## Core Policy

1. One Source of Truth is mandatory for all capabilities, policies, code, and contracts.
2. Public behavior is defined only by canonical design contracts.
3. Pre-release compatibility baggage is prohibited unless explicitly approved.
4. Deterministic behavior is required for ordering, pagination, errors, and envelope shape.
5. Security and boundary controls are first-class requirements, not implementation details.

## Structural Constraints

- Source files must remain at or below 500 lines.
- Functions must remain at or below 50 lines.
- Module responsibilities must be cohesive and single-purpose.
- Shared policy logic (for example path and boundary rules) must be centralized.
- Duplicate behavior paths for the same capability are prohibited.

## API And Contract Discipline

- Tool envelopes must conform to `docs/design/Protocol.md` exactly.
- Tool schemas and operation names must conform to `docs/design/ToolContracts.md` exactly.
- Errors must use `docs/design/ErrorTaxonomy.md` kinds and deterministic `details.reason` values.
- Deprecated aliases and compatibility pathways are prohibited unless canonically documented.

## Error And Logging Discipline

- Internal errors must use typed `thiserror` forms at internal boundaries.
- `anyhow` is reserved for top-level control boundaries.
- Error messages must be full English sentences.
- Non-public information, including usernames, must not be logged.
- Errors should be resolved where possible. If escalated, each level should attempt resolution.

## Design-Level Findings Baseline

The following findings are normative quality requirements:

- Protocol envelope alignment (`ok`/`error`/`pending`) with canonical shape.
- Single-source path-policy and workspace boundary enforcement.
- Canonical operation vocabulary across tools, with no drift.
- Canonical memory schema (`reasoning` only; no legacy shadow fields).
- Removal of dead or stub runtime surfaces from active architecture.
- Decomposition of oversized modules/functions and oversized test modules.

## Design-Level Integration Of Fix Tracking

The following policy requirements are integrated from fix tracking and are now canonical:

- Path transport must use percent-encoding policy and reject unsafe path text at boundaries.
- Tests should be colocated under sibling `tests/` folders by module concern.
- Visibility must be minimized (`private` before `pub(crate)` before `pub`).
- Long nested blocks should be decomposed into focused functions when practical.
- One-line wrapper/helper anti-patterns are prohibited.
- `Option` should not be used where `None` is impossible.
- Use mature library capabilities over bespoke mechanisms where suitable.
- Remove deprecated support and prerelease compatibility baggage.

## Verification Requirements

- Every policy in this file must map to at least one verification artifact (test, conformance case, or formal review check).
- Violations must be recorded in formal review artifacts and linked to tracked tasks in `docs/design/ProjectPlan.md`.
