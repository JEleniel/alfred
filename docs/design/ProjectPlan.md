# Project Plan

## Scope

Advance Alfred from a baseline aligned with validated design intent. This plan is the sole execution tracker.

## Planning Constraints

- Public surface remains consolidated: `capabilities`, `workspace_dir`, `status`, `search`, `fs`, `patch`, `logs`, `plan`, `memory`.
- Normative contract set: `Protocol.md`, `ToolContracts.md`, `ErrorTaxonomy.md`, `Redaction.md`, `Configuration.md`, `Defaults.md`, `StorageLayout.md`.
- Design authority is defined in `docs/design/DesignAuthority.md`.
- Structural quality policy is defined in `docs/design/QualityPolicy.md`.
- One Source of Truth is mandatory for every capability and contract.
- Review and verification order: analysis, architecture, code, documentation.
- No compatibility shims, deprecated aliases, or migration baggage unless explicitly approved.

## Work Items

- [x] P0: Lock governance and acceptance gates
    - Description: Establish explicit entry/exit criteria and non-negotiable controls for delivery.
    - Deliverable:
        - A signed-off gate list for correctness, determinism, security, and conformance.
        - Explicit rejection criteria for duplicate logic paths and contract drift.
        - Sequenced verification gates for analysis, architecture, code, and documentation.
        - A canonical dependency rule stating that no downstream implementation may begin until the governance gate is complete.
        - A completion record that identifies the governance gate as the controlling acceptance checkpoint for the plan.
    - Subtasks:
        - [x] Define the acceptance gate checklist for correctness, determinism, security, and conformance.
        - [x] Define the rejection criteria for duplicate logic paths and contract drift.
        - [x] Record the required verification order: analysis, architecture, code, and documentation.
    - Notes: This gate is a prerequisite for contract normalization and all downstream implementation work.
        - Acceptance gates: correctness, determinism, security, conformance.
        - Rejection gates: duplicate logic paths, contract drift, compatibility shims, and unapproved aliases.
        - Verification gates: analysis first, then architecture, then code, then documentation.
    - References: `docs/design/ProjectSummary.md`, `docs/design/AlfredOverview.md`, `docs/design/aurora/MIS-001/Compact.json`

- [ ] P0: Normalize authoritative contracts before implementation
    - Description: Resolve contradictions across normative docs so implementation has one canonical target.
    - Deliverable:
        - A conflict matrix covering protocol envelope shape, tool I/O schemas, and error kinds.
        - Canonicalized contract decisions reflected in one source per concern.
        - Cross-links from all dependent docs to canonical locations.
    - Subtasks:
        - [ ] Capture protocol-envelope conflicts across `Protocol.md`, `ToolContracts.md`, and `ErrorTaxonomy.md`.
        - [ ] Record canonical decisions for tool I/O schemas and execution-mode semantics.
        - [ ] Record canonical decisions for error kinds and `details.reason` usage.
        - [ ] Add cross-links from dependent design docs to the canonical sources.
    - References: `docs/design/Protocol.md`, `docs/design/ToolContracts.md`, `docs/design/ErrorTaxonomy.md`
    - Depends on: Lock governance and acceptance gates

- [ ] P0: Produce architecture baseline
    - Description: Define module boundaries and ownership aligned to the consolidated tool surface and security controls.
    - Deliverable:
        - A decomposition map assigning each capability to one owning module.
        - Explicit trust boundaries and boundary-enforcement flow.
        - Lifecycle/state mapping for sync, background, and streaming operations.
    - References: `docs/design/AlfredArchitecture.md`, `docs/design/aurora/MIS-001/Compact.json`
    - Depends on: Normalize authoritative contracts before implementation

- [ ] P0: Define deterministic envelope and taxonomy test vectors
    - Description: Convert protocol and error contracts into executable acceptance vectors before building handlers.
    - Deliverable:
        - Golden vectors for `ok`, `error`, and `pending` envelopes.
        - Golden vectors for deterministic error kinds and `details.reason` usage.
        - Negative vectors for boundary escape, disabled tools, mode conflicts, and stream conflicts.
    - References: `docs/design/Protocol.md`, `docs/design/ErrorTaxonomy.md`
    - Depends on: Normalize authoritative contracts before implementation

- [ ] P1: Implement transport and routing skeleton
    - Description: Implement stdio framing, request validation, dispatch, and envelope construction with no tool-specific logic leakage.
    - Deliverable:
        - Stdio request/response loop with deterministic frame handling.
        - Central envelope builder used by all tool handlers.
        - Capability registry wired to declared tool metadata and execution modes.
    - References: `docs/design/Protocol.md`, `docs/design/ToolContracts.md`
    - Depends on: Produce architecture baseline

- [ ] P1: Implement workspace boundary and path normalization core
    - Description: Implement canonical path resolution and boundary enforcement as a shared primitive used everywhere.
    - Deliverable:
        - Resolved-path boundary checks including symlink and junction handling.
        - Uniform path normalization (`/` separators, workspace-relative rules).
        - Deterministic non-UTF path encoding and warning behavior.
    - References: `docs/design/Protocol.md`, `docs/design/AlfredOverview.md`, `docs/design/aurora/MIS-001/Constraint/CNS-001-Workspace_Boundary.json`
    - Depends on: Implement transport and routing skeleton

- [ ] P1: Implement configuration and storage resolution core
    - Description: Implement deterministic configuration precedence and storage path resolution across user and workspace scopes.
    - Deliverable:
        - User/workspace merge logic with deterministic array semantics.
        - Storage location selection implementing `storage.user.location` and `storage.workspace.location`.
        - Stable runtime resolution for index, memory, plan, and logs paths.
    - References: `docs/design/Configuration.md`, `docs/design/Defaults.md`, `docs/design/StorageLayout.md`
    - Depends on: Implement transport and routing skeleton

- [ ] P1: Implement redaction engine and ingestion policy
    - Description: Implement deterministic redaction once and apply it consistently at ingestion and egress boundaries.
    - Deliverable:
        - Deterministic span detection/merge/token-fitting implementation.
        - Redaction at index and memory ingestion.
        - Redaction warnings surfaced via top-level envelope warnings.
    - References: `docs/design/Redaction.md`, `docs/design/Protocol.md`
    - Depends on: Implement configuration and storage resolution core

- [ ] P1: Implement `capabilities` and `workspace_dir`
    - Description: Deliver foundational context and discoverability endpoints from canonical metadata.
    - Deliverable:
        - Deterministic capabilities listing with schema/tool lockstep and execution modes.
        - Workspace root reporting with normalized output guarantees.
    - References: `docs/design/ToolContracts.md`
    - Depends on: Implement workspace boundary and path normalization core

- [ ] P1: Implement `status`
    - Description: Provide deterministic runtime status for index, memory, and configured paths.
    - Deliverable:
        - Contract-conformant `status` output including memory and path fields.
        - Stable verbose and non-verbose behavior.
    - References: `docs/design/ToolContracts.md`
    - Depends on: Implement configuration and storage resolution core

- [ ] P1: Implement `search`
    - Description: Deliver deterministic indexed search with explicit mode semantics and stable ordering.
    - Deliverable:
        - `general`, `full_text`, and `regex` behavior with mode conflict validation.
        - Stable sorting and deterministic cursor behavior.
        - Deterministic failure behavior when index is disabled or not ready.
    - References: `docs/design/ToolContracts.md`, `docs/design/ErrorTaxonomy.md`
    - Depends on: Implement redaction engine and ingestion policy

- [ ] P1: Implement `fs` (non-bulk operations)
    - Description: Deliver deterministic text-file and directory operations with dry-run defaults for mutating actions.
    - Deliverable:
        - `search`, `read_range`, `stat`, `diff`, `create_file`, `append_file`, `delete_file`, `create_dir`, `delete_dir`.
        - Text-only enforcement for read operations.
        - Per-operation deterministic diagnostics and ordering behavior.
    - References: `docs/design/ToolContracts.md`, `docs/design/Protocol.md`
    - Depends on: Implement workspace boundary and path normalization core

- [ ] P1: Implement `patch`
    - Description: Deliver deterministic patch apply/revert with duplicate-content safeguards and retained patch-id behavior.
    - Deliverable:
        - `patch`, `multi-patch`, and `revert` with per-file `patch_id` assignment.
        - Retained-state policy for revert IDs enforced exactly as specified.
        - Duplicate-content hard refusal.
    - References: `docs/design/ToolContracts.md`, `docs/design/ErrorTaxonomy.md`
    - Depends on: Implement `fs` (non-bulk operations)

- [ ] P1: Implement `logs`
    - Description: Deliver deterministic log search/tail and MCP-compliant single-stream follow behavior.
    - Deliverable:
        - `search`, `tail`, `follow` with stream lifecycle and stop semantics.
        - Single-active-stream enforcement and deterministic stream errors.
        - Stable log record filtering and pagination.
    - References: `docs/design/ToolContracts.md`, `docs/design/Protocol.md`
    - Depends on: Implement transport and routing skeleton

- [ ] P1: Implement `plan`
    - Description: Deliver workspace plan retrieval and status updates conforming to canonical operation names.
    - Deliverable:
        - `retrieve` and `update_status` operation support with deterministic result shape.
        - Plan path resolution according to configuration/default rules.
    - References: `docs/design/ToolContracts.md`, `docs/design/Configuration.md`
    - Depends on: Implement configuration and storage resolution core

- [ ] P1: Implement `memory`
    - Description: Deliver local-only scoped memory CRUD and retrieval with deterministic indexing and scope-aware results.
    - Deliverable:
        - `create`, `retrieve`, `update`, `delete` with UUID identity rules.
        - Scope-aware retrieval behavior and deterministic ordering.
        - Redaction-safe persistence and retrieval.
    - References: `docs/design/ToolContracts.md`, `docs/design/Redaction.md`
    - Depends on: Implement redaction engine and ingestion policy

- [ ] P1: Implement `fs` bulk background lifecycle
    - Description: Deliver background bulk execution with in-tool status polling and deterministic lifecycle states.
    - Deliverable:
        - Bulk execute/status/cancel state machine and summaries.
        - `pending` envelope contract (`operation_id`, `state`, `poll_with`).
        - Deterministic terminal states and cancellation behavior.
    - References: `docs/design/ToolContracts.md`, `docs/design/Protocol.md`, `docs/design/ErrorTaxonomy.md`
    - Depends on: Implement `fs` (non-bulk operations)

- [ ] P0: Build contract-conformance test suite
    - Description: Validate every tool contract and envelope shape against golden vectors and deterministic error semantics.
    - Deliverable:
        - Conformance coverage for all tools and operations in the consolidated surface.
        - Golden-output checks for ordering, pagination, warning placement, and error mapping.
        - Cross-platform path and boundary edge-case coverage.
    - References: `docs/design/ToolContracts.md`, `docs/design/Protocol.md`, `docs/design/ErrorTaxonomy.md`
    - Depends on: Implement `capabilities` and `workspace_dir`

- [ ] P0: Run formal analysis gate on architecture
    - Description: Perform formal analysis of module boundaries, coupling, and call structure before final hardening.
    - Deliverable:
        - Updated call map and call graph artifacts.
        - Verification that each capability has a single owning implementation path.
        - Recorded corrective actions for any hotspot or coupling violations.
    - References: `docs/design/analysis/Code.md`, `docs/design/analysis/CallMap.md`, `docs/design/analysis/CallGraph.md`
    - Depends on: Build contract-conformance test suite

- [ ] P0: Run formal code review gate
    - Description: Execute formal review for correctness, safety controls, and adherence to deterministic contracts.
    - Deliverable:
        - Formal review report with evidence-linked findings.
        - Correction list prioritized by safety and contract risk.
        - Closure evidence for all P0 and P1 review findings.
    - References: `docs/design/Review-Code.md`, `docs/design/analysis/Review-Code.md`
    - Depends on: Run formal analysis gate on architecture

- [ ] P1: Run formal documentation review and alignment
    - Description: Validate docs against implemented behavior and remove stale or conflicting statements.
    - Deliverable:
        - Updated documentation set with resolved drift across overview, protocol, contracts, defaults, and configuration.
        - Documentation review report with resolved/remaining items.
    - References: `docs/design/Review-Documentation.md`, `docs/design/ProjectSummary.md`
    - Depends on: Run formal code review gate

- [ ] P0: Final release-readiness gate
    - Description: Confirm that implementation, tests, and documentation meet acceptance criteria.
    - Deliverable:
        - Checklist showing all gates passed in required order.
        - Evidence index linking tests, reviews, and analysis artifacts.
        - Go/No-Go decision record.
    - Depends on: Run formal documentation review and alignment
