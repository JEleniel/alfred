# Plan: Implement remaining Alfred commands

This plan is aligned to the current design artifacts in `docs/design/` (notably `AlfredOverview.md`, `ToolContracts.md`, `Protocol.md`, `Configuration.md`, and `Redaction.md`). The public tool surface is intentionally consolidated; items 1–16 are retained for historical traceability, while items 17+ track the consolidated tool surfaces and any newly specified cross-cutting requirements.

1. [x] Conform MCP stdio interface envelope and error taxonomy
    - Priority: 0
    - Cards: "INT-001", "ART-001", "ART-002", "CNS-005", "CNS-007", "CNS-015", "STR-007"
    - Description: Align the runtime MCP stdio protocol envelope and deterministic error taxonomy with the design so tool contracts can be implemented consistently.
    - Deliverables:
        - Update the tool error payload to support structured `details` (see `docs/design/ErrorTaxonomy.md`).
        - Align `ErrorKind` coverage to the design’s taxonomy and update error conversions.
        - Align `pending` envelopes to the protocol requirements in `docs/design/Protocol.md` (including transport-equivalent metadata).
        - Update/extend `tests/protocol_tests.rs` and `tests/workspace_query_tests.rs` to assert the updated envelope and error shapes.
    - Notes: This plan is intentionally protocol-first and synchronous-first. Background work is restricted to `bulk_fs_operations` per the current design.
    - Status: Completed

2. [x] Implement configuration loading and deterministic policy guardrails
    - Priority: 0
    - Cards: "CNS-010", "CNS-015", "STR-014", "STR-009"
    - Description: Load and merge user/workspace configuration and use it to deterministically govern tool exposure and safety posture.
    - Deliverables:
        - Load and merge JSON config from the default user and workspace config paths.
        - Implement tool enable/disable policy (including default-disable for mutating tools).
        - Ensure both `tools/list` and runtime dispatch reflect policy.
        - Add/extend tests in `tests/configuration_tests.rs` for config precedence and gating behavior.
    - Status: Completed

3. [x] Implement `capabilities` tool and make `tools/list` truthful
    - Priority: 1
    - Cards: "CNS-008", "CNS-010", "STR-010", "ART-002"
    - Description: Provide deterministic capability discovery and ensure the MCP `tools/list` surface matches what is actually implemented and enabled.
    - Deliverables:
        - Implement the `capabilities` tool contract from `docs/design/ToolContracts.md`.
        - Ensure `tools/list` only advertises tools that are implemented and enabled by policy.
        - Publish limits required by tool contracts (for example, file chunk limits).
        - Add protocol-level tests asserting the `capabilities` payload shape and stable ordering.
    - Status: Completed

4. [x] Bring workspace query tools up to contract conformance
    - Priority: 1
    - Cards: "ART-007", "CNS-001", "CNS-015", "STR-001", "STR-015"
    - Description: Tighten the existing workspace query tools (`workspace_dir`, `ls`, `read_range`, `file_stat`, `file_read_bytes`, `grep`, `search`, `diff`) to match their tool contracts.
    - Deliverables:
        - Ensure output path normalization and stable ordering matches the rules in `docs/design/ToolContracts.md`.
        - Ensure `read_range` fails deterministically for binary/non-text input.
        - Add bounded limits for `file_read_bytes` and surface them in `capabilities.limits`.
        - Ensure index-not-ready errors for `grep`/`search` emit `tool_unavailable` with `details.reason: "index_not_ready"`.
        - Add missing tool-level tests for `ls`, `file_stat`, `file_read_bytes`, and `diff`.
    - Status: Completed

5. [x] Implement `log_search`
    - Priority: 1
    - Cards: "STR-008", "CNS-012", "CNS-015"
    - Description: Implement deterministic log searching over Alfred’s structured runtime logs or logs at a path provided by the agent, within the workspace. This is one of the few exceptions where we let the agent see outside the sandbox, but _only_ Alfred's logs.
    - Deliverables:
        - Implement `log_search` per `docs/design/ToolContracts.md`.
        - Add stable pagination with `cursor` and `next_cursor`.
        - Add tests covering filtering, pagination, and deterministic ordering.
    - Status: Completed

6. [x] Implement `plan_get`
    - Priority: 1
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Return the current plan items from the workspace-scoped project plan file.
    - Deliverables:
        - Implement `plan_get` per `docs/design/ToolContracts.md`.
        - Enforce deterministic parsing and normalization.
        - Add tests for “file missing”, parse errors, and successful reads.
    - Status: Completed

7. [x] Implement `plan_update`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Update the status of a specific plan item with serialized writes and deterministic conflicts.
    - Deliverables:
        - Implement `plan_update` per `docs/design/ToolContracts.md`.
        - Implement lock acquisition and `conflict` errors with `details.reason: "locked"`.
        - Add tests for locked writes, missing ids, and successful updates.
    - Status: Completed

8. [x] Implement `plan_edit`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Edit a full plan item deterministically.
    - Deliverables:
        - Implement `plan_edit` per `docs/design/ToolContracts.md`.
        - Preserve stable ids and deterministic formatting.
        - Add tests for edit validation and stable round-tripping.
    - Status: Completed

9. [x] Implement `plan_add`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Append a new plan item deterministically (server assigns id).
    - Deliverables:
        - Implement `plan_add` per `docs/design/ToolContracts.md`.
        - Ensure sequential id assignment starting at 1.
        - Add tests for id assignment and concurrent add conflict behavior.
    - Status: Completed

10. [x] Implement `plan_delete`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Remove a plan item by id deterministically.
    - Deliverables:
        - Implement `plan_delete` per `docs/design/ToolContracts.md`.
        - Add tests for missing ids and successful deletes.
    - Status: Completed

11. [x] Implement `memory_put`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-006", "CNS-015"
    - Description: Upsert a memory fact into the local persistent store.
    - Deliverables:
        - Implement `memory_put` per `docs/design/ToolContracts.md`.
        - Ensure deterministic upsert semantics and timestamps.
        - Add tests for required fields and id stability.
    - Status: Completed

12. [x] Implement `memory_get`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-015"
    - Description: Retrieve a single memory fact by id.
    - Deliverables:
        - Implement `memory_get` per `docs/design/ToolContracts.md`.
        - Add tests for missing facts and successful retrieval.
    - Status: Completed

13. [x] Implement `memory_delete`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-003", "CNS-015"
    - Description: Delete a memory fact deterministically (with optional dry-run).
    - Deliverables:
        - Implement `memory_delete` per `docs/design/ToolContracts.md`.
        - Add tests for dry-run behavior and delete semantics.
    - Status: Completed

14. [x] Implement `memory_list`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-015"
    - Description: Deterministically list memory facts with stable ordering and pagination.
    - Deliverables:
        - Implement `memory_list` per `docs/design/ToolContracts.md`.
        - Add tests for stable ordering, pagination, and tag filtering.
    - Status: Completed

15. [x] Implement `memory_search`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-015"
    - Description: Full-text search over memory facts.
    - Deliverables:
        - Implement `memory_search` per `docs/design/ToolContracts.md`.
        - Add tests for deterministic ranking/ordering and tag filtering.
    - Status: Completed

16. [x] Refine architecture for consolidated command surface
    - Priority: 0
    - Cards: "STR-001", "STR-003", "STR-004", "STR-008", "STR-017", "CNS-015"
    - Description: Consolidate command families to reduce exposed tool count and align protocol/configuration contracts.
    - Deliverables:
        - Consolidate command taxonomy to `search`, `fs_operations`, `bulk_fs_operations`, `patch`, `log_operations`, `plan_operations`, and `memory`.
        - Remove standalone `env_*` and `job_*` command families from the architecture contract.
        - Add `.alfred/` workspace storage root defaults and index single-location persistence rule.
        - Remove byte-oriented filesystem operations from the architecture contract.
        - Add duplicate-content-risk warning requirements for patch application.
    - Status: Completed

17. [ ] Implement consolidated `search` command
    - Priority: 2
    - Cards: "STR-001", "CNS-001", "CNS-015"
    - Description: Merge workspace text search behaviors into one deterministic command.
    - Deliverables:
        - Implement `search` per the consolidated contract.
        - Add tests for literal/regex mode, pagination, and index-not-ready/index-disabled errors.
    - Status: Not Started

18. [ ] Implement consolidated `fs_operations` command
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-015"
    - Description: Merge non-bulk file and directory operations into one deterministic command.
    - Deliverables:
        - Implement operation-dispatch for list/read_range/stat/diff/create_file/append_file/delete_file/create_dir/delete_dir.
        - Ensure text-only behavior and deterministic binary-file rejection for line-range reads.
        - Add tests for operation-specific validation and dry-run semantics.
    - Status: Not Started

19. [ ] Implement consolidated `patch` command
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-017"
    - Description: Replace single-file and multi-file patch tools with one multi-patch interface.
    - Deliverables:
        - Implement array-based patch application with per-file results.
        - Add duplicate-content-risk warning detection and reporting.
        - Add tests for conflicts, dry-run behavior, and duplicate-content warnings.
    - Status: Not Started

20. [ ] Implement consolidated `bulk_fs_operations` command
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015", "CNS-019"
    - Description: Replace bulk move/copy/delete and standalone job tooling with one command that can execute and report status.
    - Deliverables:
        - Implement execute/status/cancel modes with deterministic operation IDs.
        - Support optional background mode only for this command.
        - Ensure background acceptance uses the `pending` envelope and is polled via `bulk_fs_operations` per `docs/design/Protocol.md`.
        - Add tests for status polling, cancellation, overwrite behavior, and recursion semantics.
    - Status: Not Started

21. [ ] Implement consolidated `log_operations` command
    - Priority: 2
    - Cards: "STR-008", "CNS-012", "CNS-015"
    - Description: Merge log search/tail behaviors into one deterministic command.
    - Deliverables:
        - Implement `search` and bounded `tail` operations.
        - Add tests for filtering, ordering, cursor behavior, and bounds.
    - Status: Not Started

22. [ ] Implement consolidated `plan_operations` command
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Merge plan read/add/edit/update/delete behaviors into one deterministic command.
    - Deliverables:
        - Implement operation routing and deterministic validation.
        - Keep lock semantics and update lock path to `.alfred/locks`.
        - Add tests for conflict and successful writes.
    - Status: Not Started

23. [ ] Implement consolidated `memory` command
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-006", "CNS-015"
    - Description: Merge memory CRUD/search operations into one deterministic command surface.
    - Deliverables:
        - Implement operation routing for put/get/delete/list/search.
        - Add tests for workspace-memory storage options and merge policy behavior.
    - Status: Not Started

24. [ ] Implement workspace storage-root and index-location controls
    - Priority: 2
    - Cards: "CNS-015", "DST-001", "DST-004"
    - Description: Add configurable workspace storage root and enforce single-location index persistence.
    - Deliverables:
        - Implement `workspace.storage.root` and `.alfred/` defaults.
        - Implement `index.enabled` gating and ensure index-backed tools emit deterministic `tool_unavailable` reasons for index-disabled/index-not-ready.
        - Implement `index.persistence.location` and non-duplication guarantees.
        - Add tests for workspace/user location selection and migration behavior.
    - Status: Not Started

25. [x] Retire deprecated orchestration scope
    - Priority: 3
    - Cards: "CNS-015"
    - Description: Deprecated multi-step orchestration support was removed from the architecture design and should not be implemented.
    - Deliverables:
        - Remove deprecated orchestration references from design requirements/contracts.
        - Keep this plan item as historical traceability only.
    - Status: Completed

26. [ ] Implement deterministic redaction end-to-end
    - Priority: 0
    - Cards: "CNS-015", "CNS-021"
    - Description: Apply deterministic NPI redaction per `docs/design/Redaction.md` across tool outputs, logs, and at ingestion time for any persisted indexes (workspace index and memory index).
    - Deliverables:
        - Redaction detection and replacement behavior matches `docs/design/Redaction.md` (including length-fitting replacement).
        - Redaction is applied before persistence for any indexable content (workspace index and memory index).
        - Tool responses and structured logs are redacted deterministically and do not leak inbound request payloads.
        - When redaction occurs, results surface a stable warning in metadata (counts only; never the original value).
        - Configuration keys under `redaction.*` are honored as defined in `docs/design/Configuration.md`.
    - Status: Not Started

27. [ ] Implement encoding-safe path handling and deterministic warnings
    - Priority: 1
    - Cards: "CNS-021", "CTL-009"
    - Description: Ensure all protocol-visible strings are valid UTF-8 JSON/NDJSON, and handle non-text filesystem paths deterministically per `docs/design/Protocol.md`.
    - Deliverables:
        - Incoming `path` values accept `\\` as `/` prior to normalization.
        - Any path that cannot be represented as Unicode text is emitted using the deterministic encoding rules in `docs/design/Protocol.md`.
        - When encoded-path rendering occurs, results include a stable warning (for example `meta.warnings += {"kind":"path_encoded"}`).
        - Ordering and cursor behavior remain consistent when encoded paths are present.
    - Status: Not Started

28. [ ] Retire byte-oriented filesystem operations from the exposed tool surface
    - Priority: 1
    - Cards: "STR-003", "CNS-015"
    - Description: Align the exposed tool surface with the current design requirement that Alfred operates on text files only and does not expose byte-oriented filesystem operations.
    - Deliverables:
        - `capabilities` and `tools/list` do not advertise byte-oriented filesystem tools.
        - Calls to retired byte-oriented filesystem tools fail deterministically with `invalid_argument` and a stable message.
        - Equivalent workflows are supported via consolidated `fs_operations` (for text) and `patch`/`bulk_fs_operations` where applicable.
    - Status: Not Started
    - Dependencies: 18

29. [ ] Add a conformance suite for consolidated tool contracts
    - Priority: 1
    - Cards: "CAP-013", "CNS-005", "CNS-015"
    - Description: Provide an automated conformance suite that validates contract shapes, deterministic ordering, dry-run guarantees, workspace-boundary enforcement, and redaction behavior for the consolidated tool surface.
    - Deliverables:
        - Conformance tests cover: success/error/pending envelopes, stable ordering, pagination cursors, and deterministic error kinds.
        - Tests cover policy gating for disabled tools (omitted from `capabilities`; calls fail with `invalid_argument`).
        - Tests cover the index-not-ready and index-disabled failure modes for index-backed tools.
        - Redaction and path-encoding behaviors are validated as part of conformance.
    - Status: Not Started
    - Dependencies: 17, 18, 19, 20, 21, 22, 23, 26, 27

30. [ ] Reconcile generated Aurora model outputs with the consolidated tool surface
    - Priority: 3
    - Cards: "MIS-001"
    - Description: Ensure the rendered MIS-001 model bundle under `docs/design/` remains consistent with the consolidated public tool surface described in `docs/design/ToolContracts.md`.
    - Deliverables:
        - Model render outputs (markdown + views) reflect the effective top-level tool surface and do not contradict `ToolContracts.md`.
        - Any deprecated tool families are either removed from the model or clearly marked as non-exposed/not implemented.
        - The model bundle remains valid and can be regenerated deterministically.
    - Status: Not Started
