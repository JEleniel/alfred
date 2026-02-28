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
    - Notes: This plan is intentionally protocol-first and synchronous-first. Background work is restricted to `fs` bulk execution per the current design.
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
    - Description: Tighten the existing workspace query tools (`workspace_dir`, `ls`, `read_range`, `file_stat`, `grep`, `search`, `diff`) to match their tool contracts.
    - Deliverables:
        - Ensure output path normalization and stable ordering matches the rules in `docs/design/ToolContracts.md`.
        - Ensure `read_range` fails deterministically for binary/non-text input.
        - Ensure index-not-ready errors for `grep`/`search` emit `tool_unavailable` with `details.reason: "index_not_ready"`.
        - Add missing tool-level tests for `ls`, `file_stat`, and `diff`.
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
    - Description: Update the status of a specific plan item with deterministic, atomic writes.
    - Deliverables:
        - Implement `plan_update` per `docs/design/ToolContracts.md`.
        - Ensure writes are atomic (write temp + rename).
        - Avoid lock files; concurrent writers are unsupported.
        - Add tests for missing ids and successful updates.
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
        - Add tests for id assignment.
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
        - Consolidate command taxonomy to `search`, `fs`, `patch`, `logs`, `plan`, and `memory`.
        - Remove standalone `env_*` and `job_*` command families from the architecture contract.
        - Add `.alfred/` workspace storage root defaults and index single-location persistence rule.
        - Remove byte-oriented filesystem operations from the architecture contract.
        - Add duplicate-content refusal requirements for patch application.
    - Status: Completed

17. [x] Implement consolidated `search` command
    - Priority: 2
    - Cards: "STR-001", "CNS-001", "CNS-015"
    - Description: Merge workspace text search behaviors into one deterministic command.
    - Deliverables:
        - Implement `search` per the consolidated contract.
        - Add tests for literal/regex mode, pagination, and index-not-ready/index-disabled errors.
    - Status: Completed

18. [ ] Implement consolidated `fs` command
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-015"
    - Description: Merge non-bulk file and directory operations into one deterministic command.
    - Deliverables:
        - Implement operation-dispatch for list/read_range/stat/diff/create_file/append_file/delete_file/create_dir/delete_dir.
        - Ensure text-only behavior and deterministic binary-file rejection for line-range reads.
        - Add tests for operation-specific validation and dry-run semantics.
    - Status: Planned

19. [x] Implement consolidated `patch` command
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-017"
    - Description: Replace single-file and multi-file patch tools with one multi-patch interface.
    - Deliverables:
        - Implement array-based patch application with per-file results.
        - Implement `revert` support for the most recently applied patch batch (in-memory only).
        - Add duplicate-content detection and refusal behavior.
        - Add tests for dry-run behavior, revert safety checks, and duplicate-content refusal.
    - Status: Completed

20. [ ] Implement `fs` bulk background execution
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015", "CNS-019"
    - Description: Implement deterministic bulk move/copy/delete and built-in status/cancel semantics under `fs`.
    - Deliverables:
        - Implement execute/status/cancel modes with deterministic operation IDs.
        - Support optional background mode only for this operation.
        - Ensure background acceptance uses the `pending` envelope and is polled via `fs` per `docs/design/Protocol.md`.
        - Add tests for status polling, cancellation, overwrite behavior, and recursion semantics.
    - Status: Planned

21. [ ] Implement consolidated `logs` command
    - Priority: 2
    - Cards: "STR-008", "CNS-012", "CNS-015"
    - Description: Merge log search/tail behaviors into one deterministic command.
    - Deliverables:
        - Implement `search` and bounded `tail` operations.
        - Add tests for filtering, ordering, cursor behavior, and bounds.
    - Status: Planned

22. [ ] Implement consolidated `plan` command
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Merge plan read/add/edit/update/delete behaviors into one deterministic command.
    - Deliverables:
        - Implement operation routing and deterministic validation.
        - Ensure writes are atomic (write temp + rename).
        - Avoid lock files; concurrent writers are unsupported.
        - Add tests for successful writes.
    - Status: Planned

23. [ ] Implement consolidated `memory` command
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-006", "CNS-015"
    - Description: Merge memory CRUD/search operations into one deterministic command surface.
    - Deliverables:
        - Implement operation routing for put/get/delete/list/search.
        - Add `scope: "user" | "workspace"` to memory writes and return `scope` on all reads.
        - Issue UUID ids for new memory facts (create without an id; return issued id).
        - Ensure `get`, `list`, and `search` treat memory as one contiguous corpus across enabled scopes while indicating per-fact `scope`.
        - Add tests for workspace-memory storage options and merge policy behavior.
    - Status: Planned

24. [x] Implement workspace storage-root and index-location controls
    - Priority: 2
    - Cards: "CNS-015", "DST-001", "DST-004"
    - Description: Add configurable workspace storage root and enforce single-location index persistence.
    - Deliverables:
        - Implement `workspace.storage.root` and `.alfred/` defaults.
        - Ensure `.alfred/.gitignore` exists and ignores everything except `config.json`.
        - Implement `index.enabled` gating and ensure index-backed tools emit deterministic `tool_unavailable` reasons for index-disabled/index-not-ready.
        - Implement `index.persistence.location` and non-duplication guarantees.
        - Add tests for workspace/user location selection and migration behavior.
    - Status: Completed

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
    - Status: Planned

27. [ ] Implement encoding-safe path handling and deterministic warnings
    - Priority: 1
    - Cards: "CNS-021", "CTL-009"
    - Description: Ensure all protocol-visible strings are valid UTF-8 JSON/NDJSON, and handle non-text filesystem paths deterministically per `docs/design/Protocol.md`.
    - Deliverables:
        - Incoming `path` values accept `\\` as `/` prior to normalization.
        - Any path that cannot be represented as Unicode text is emitted using the deterministic encoding rules in `docs/design/Protocol.md`.
        - When encoded-path rendering occurs, results include a stable warning (for example `meta.warnings += {"kind":"path_encoded"}`).
        - Ordering and cursor behavior remain consistent when encoded paths are present.
    - Status: Planned

28. [x] Retire byte-oriented filesystem operations from the exposed tool surface
    - Priority: 1
    - Cards: "STR-003", "CNS-015"
    - Description: Align the exposed tool surface with the current design requirement that Alfred operates on text files only and does not expose byte-oriented filesystem operations.
    - Deliverables:
        - `capabilities` and `tools/list` do not advertise byte-oriented filesystem tools.
        - Calls to retired byte-oriented filesystem tools fail deterministically with `invalid_argument` and a stable message.
        - Equivalent workflows are supported via consolidated `fs` (for text) and `patch` where applicable.
    - Status: Completed
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
    - Status: Planned
    - Dependencies: 17, 18, 19, 20, 21, 22, 23, 26, 27

30. [ ] Reconcile generated Aurora model outputs with the consolidated tool surface
    - Priority: 3
    - Cards: "MIS-001"
    - Description: Ensure the rendered MIS-001 model bundle under `docs/design/` remains consistent with the consolidated public tool surface described in `docs/design/ToolContracts.md`.
    - Deliverables:
        - Model render outputs (markdown + views) reflect the effective top-level tool surface and do not contradict `ToolContracts.md`.
        - Any deprecated tool families are either removed from the model or clearly marked as non-exposed/not implemented.
        - The model bundle remains valid and can be regenerated deterministically.
    - Status: Planned

31. [x] Add `status` tool
    - Priority: 2
    - Cards: "STR-003", "CNS-015"
    - Description: Expose a lightweight runtime status snapshot (memory usage, index readiness, key configured paths).
    - Deliverables:
        - Implement `status` tool with optional `verbose` flag.
        - Include system + process memory usage and index snapshot stats.
        - Add tests validating the response shape.
    - Status: Completed

32. [x] Add MCP prompt support
    - Priority: 2
    - Cards: "STR-003", "CNS-015"
    - Description: Provide MCP prompt discovery and retrieval to supply agent guidance.
    - Deliverables:
        - Implement `prompts/list` and `prompts/get` protocol handlers.
        - Advertise prompts capability in `initialize` response.
        - Add tests for prompt classification and prompt retrieval.
    - Status: Completed

33. [ ] Prefer patch-first internal mutations
    - Priority: 3
    - Cards: "STR-003", "CNS-015"
    - Description: Replace internal rewrite/rename mutation paths with patch and append-based workflows where they better align with deterministic, minimal-edit semantics.
    - Deliverables:
        - Identify remaining write paths that rewrite whole files for small edits.
        - Migrate the highest-impact ones to patch/apply flows (preserving atomicity and determinism).
        - Add targeted tests for any migrated mutation path.
    - Status: Planned

34. [ ] Implement storage location controls for workspace and user artifacts
    - Priority: 1
    - Cards: "DST-001", "CNS-015"
    - Description: Support relocating workspace-scoped artifacts into OS user directories (per-workspace) and relocating user-scoped artifacts into the workspace, without changing precedence semantics.
    - Deliverables:
        - Add configuration keys for storage location selection as defined in `docs/design/Configuration.md`.
        - Resolve per-workspace user config/data roots using the stable workspace id.
        - Ensure artifacts follow consistent subfolder conventions (`index/`, `memory/`, `logs/`) under the selected storage roots.
        - Add tests covering all supported layouts and ensuring no workspace boundary regressions.
    - Status: Planned
    - Dependencies: 2

35. [ ] Implement runtime log path selection and stable log naming
    - Priority: 1
    - Cards: "STR-008", "CNS-012", "CNS-015"
    - Description: Create a new runtime log file on server startup in the first writable preferred location (or an explicit override), rotate prior logs to ZIP, and enforce a bounded retention window.
    - Deliverables:
        - Implement the `logging.*` configuration keys defined in `docs/design/Configuration.md`.
        - Create runtime logs using stable, time-sortable filenames (UTC timestamp with second precision) and archive rotated logs as ZIP files.
        - Ensure `status` reports the effective runtime log path.
        - Add tests for location selection, rotation/retention behavior, and deterministic log record formatting.
    - Status: Planned
    - Dependencies: 34

36. [ ] Remove SQLite naming and dependency remnants
    - Priority: 2
    - Cards: "DST-004"
    - Description: Remove unused SQLite dependencies/references and eliminate `.sqlite3` filenames from persisted Alfred artifacts.
    - Deliverables:
        - Remove unused SQLite crates and any references in docs/config/contracts.
        - Change persisted artifact naming to directory-based stores (no `.sqlite3` files).
        - Add deterministic migration behavior for existing `.sqlite3`-named artifacts.
    - Status: Planned
    - Dependencies: 34

37. [ ] Enforce single-location persistence for all active indexes
    - Priority: 1
    - Cards: "CNS-015"
    - Description: Extend the single-location persistence rule to all active indexes (workspace index and memory indexes), ensuring no duplicate persisted copies exist for a given workspace/scope.
    - Deliverables:
        - Ensure for any indexable store (workspace index, user memory, workspace memory) there is exactly one persisted active copy.
        - Add tests that validate no duplicates are created across supported storage layouts.
    - Status: Planned
    - Dependencies: 23, 24, 34
