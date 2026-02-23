# Plan: Implement remaining Alfred commands

1. [x] Conform MCP stdio interface envelope and error taxonomy
    - Priority: 0
    - Cards: "INT-001", "ART-001", "ART-002", "CNS-005", "CNS-007", "CNS-015", "STR-007"
    - Description: Align the runtime MCP stdio protocol envelope and deterministic error taxonomy with the design so tool contracts can be implemented consistently.
    - Deliverables:
        - Update the tool error payload to support structured `details` (see `docs/design/ErrorTaxonomy.md`).
        - Align `ErrorKind` coverage to the design’s taxonomy and update error conversions.
        - Align `pending` envelopes to the protocol requirements in `docs/design/Protocol.md` (including transport-equivalent metadata).
        - Update/extend `src/tests/protocol_tests.rs` and `src/tests/workspace_query_tests.rs` to assert the updated envelope and error shapes.
    - Notes: This plan is intentionally protocol-first and synchronous-first. Job tooling is planned later.
    - Status: completed

2. [x] Implement configuration loading and deterministic policy guardrails
    - Priority: 0
    - Cards: "CNS-010", "CNS-015", "STR-014", "STR-009"
    - Description: Load and merge user/workspace configuration and use it to deterministically govern tool exposure and safety posture.
    - Deliverables:
        - Load and merge JSON config from the default user and workspace config paths.
        - Implement tool enable/disable policy (including default-disable for mutating tools).
        - Ensure both `tools/list` and runtime dispatch reflect policy.
        - Add/extend tests in `src/tests/configuration_tests.rs` for config precedence and gating behavior.
    - Status: completed

3. [x] Implement `capabilities` tool and make `tools/list` truthful
    - Priority: 1
    - Cards: "CNS-008", "CNS-010", "STR-010", "ART-002"
    - Description: Provide deterministic capability discovery and ensure the MCP `tools/list` surface matches what is actually implemented and enabled.
    - Deliverables:
        - Implement the `capabilities` tool contract from `docs/design/ToolContracts.md`.
        - Ensure `tools/list` only advertises tools that are implemented and enabled by policy.
        - Publish limits required by tool contracts (for example, file chunk limits).
        - Add protocol-level tests asserting the `capabilities` payload shape and stable ordering.
    - Status: completed

4. [ ] Bring workspace query tools up to contract conformance
    - Priority: 1
    - Cards: "ART-007", "CNS-001", "CNS-015", "STR-001", "STR-015"
    - Description: Tighten the existing workspace query tools (`workspace_dir`, `ls`, `read_range`, `file_stat`, `file_read_bytes`, `grep`, `search`, `diff`) to match their tool contracts.
    - Deliverables:
        - Ensure output path normalization and stable ordering matches the rules in `docs/design/ToolContracts.md`.
        - Ensure `read_range` fails deterministically for binary/non-text input.
        - Add bounded limits for `file_read_bytes` and surface them in `capabilities.limits`.
        - Ensure index-not-ready errors for `grep`/`search` emit `tool_unavailable` with `details.reason: "index_not_ready"`.
        - Add missing tool-level tests for `ls`, `file_stat`, `file_read_bytes`, and `diff`.
    - Status: planned

5. [ ] Implement `log_search`
    - Priority: 1
    - Cards: "STR-008", "CNS-012", "CNS-015"
    - Description: Implement deterministic log searching over Alfred’s structured runtime logs.
    - Deliverables:
        - Implement `log_search` per `docs/design/ToolContracts.md`.
        - Add stable pagination with `cursor` and `next_cursor`.
        - Add tests covering filtering, pagination, and deterministic ordering.
    - Status: planned

6. [ ] Implement `plan_get`
    - Priority: 1
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Return the current plan items from the workspace-scoped project plan file.
    - Deliverables:
        - Implement `plan_get` per `docs/design/ToolContracts.md`.
        - Enforce deterministic parsing and normalization.
        - Add tests for “file missing”, parse errors, and successful reads.
    - Status: planned

7. [ ] Implement `plan_update`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Update the status of a specific plan item with serialized writes and deterministic conflicts.
    - Deliverables:
        - Implement `plan_update` per `docs/design/ToolContracts.md`.
        - Implement lock acquisition and `conflict` errors with `details.reason: "locked"`.
        - Add tests for locked writes, missing ids, and successful updates.
    - Status: planned

8. [ ] Implement `plan_edit`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Edit a full plan item deterministically.
    - Deliverables:
        - Implement `plan_edit` per `docs/design/ToolContracts.md`.
        - Preserve stable ids and deterministic formatting.
        - Add tests for edit validation and stable round-tripping.
    - Status: planned

9. [ ] Implement `plan_add`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Append a new plan item deterministically (server assigns id).
    - Deliverables:
        - Implement `plan_add` per `docs/design/ToolContracts.md`.
        - Ensure sequential id assignment starting at 1.
        - Add tests for id assignment and concurrent add conflict behavior.
    - Status: planned

10. [ ] Implement `plan_delete`
    - Priority: 2
    - Cards: "ART-004", "STR-004", "CNS-015"
    - Description: Remove a plan item by id deterministically.
    - Deliverables:
        - Implement `plan_delete` per `docs/design/ToolContracts.md`.
        - Add tests for missing ids and successful deletes.
    - Status: planned

11. [ ] Implement `memory_put`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-006", "CNS-015"
    - Description: Upsert a memory fact into the local persistent store.
    - Deliverables:
        - Implement `memory_put` per `docs/design/ToolContracts.md`.
        - Ensure deterministic upsert semantics and timestamps.
        - Add tests for required fields and id stability.
    - Status: planned

12. [ ] Implement `memory_get`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-015"
    - Description: Retrieve a single memory fact by id.
    - Deliverables:
        - Implement `memory_get` per `docs/design/ToolContracts.md`.
        - Add tests for missing facts and successful retrieval.
    - Status: planned

13. [ ] Implement `memory_delete`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-003", "CNS-015"
    - Description: Delete a memory fact deterministically (with optional dry-run).
    - Deliverables:
        - Implement `memory_delete` per `docs/design/ToolContracts.md`.
        - Add tests for dry-run behavior and delete semantics.
    - Status: planned

14. [ ] Implement `memory_list`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-015"
    - Description: Deterministically list memory facts with stable ordering and pagination.
    - Deliverables:
        - Implement `memory_list` per `docs/design/ToolContracts.md`.
        - Add tests for stable ordering, pagination, and tag filtering.
    - Status: planned

15. [ ] Implement `memory_search`
    - Priority: 2
    - Cards: "ART-008", "STR-017", "CNS-015"
    - Description: Full-text search over memory facts.
    - Deliverables:
        - Implement `memory_search` per `docs/design/ToolContracts.md`.
        - Add tests for deterministic ranking/ordering and tag filtering.
    - Status: planned

16. [ ] Implement `env_list`
    - Priority: 2
    - Cards: "CNS-020", "STR-009", "CNS-015"
    - Description: List Alfred-managed environment variables (scoped to Alfred-controlled contexts).
    - Deliverables:
        - Implement `env_list` per `docs/design/ToolContracts.md`.
        - Ensure values are treated as non-public information in logs.
        - Add tests for deterministic output.
    - Status: planned

17. [ ] Implement `env_get`
    - Priority: 2
    - Cards: "CNS-020", "STR-009", "CNS-015"
    - Description: Retrieve a single Alfred-managed environment variable.
    - Deliverables:
        - Implement `env_get` per `docs/design/ToolContracts.md`.
        - Add tests for missing keys and successful reads.
    - Status: planned

18. [ ] Implement `env_set`
    - Priority: 2
    - Cards: "CNS-020", "STR-009", "CNS-003", "CNS-015"
    - Description: Set an Alfred-managed environment variable (supports dry-run).
    - Deliverables:
        - Implement `env_set` per `docs/design/ToolContracts.md`.
        - Enforce deterministic validation and dry-run semantics.
        - Add tests for dry-run and persistence semantics.
    - Status: planned

19. [ ] Implement `env_unset`
    - Priority: 2
    - Cards: "CNS-020", "STR-009", "CNS-003", "CNS-015"
    - Description: Unset an Alfred-managed environment variable (supports dry-run).
    - Deliverables:
        - Implement `env_unset` per `docs/design/ToolContracts.md`.
        - Add tests for missing keys and dry-run behavior.
    - Status: planned

20. [ ] Implement `list_tasks`
    - Priority: 2
    - Cards: "ART-003", "STR-005", "CNS-018", "CNS-010"
    - Description: Deterministically enumerate safe tasks available in the workspace.
    - Deliverables:
        - Implement `list_tasks` per `docs/design/ToolContracts.md`.
        - Add tests for filtering, pagination, and deterministic ordering.
    - Status: planned

21. [ ] Implement `task_run` (sync-only initial implementation)
    - Priority: 2
    - Cards: "ART-003", "STR-005", "CNS-018", "CNS-010"
    - Description: Run a named task under guardrails, defaulting to argv-only execution and emitting normalized diagnostics.
    - Deliverables:
        - Implement the synchronous `task_run` path per `docs/design/ToolContracts.md`.
        - Enforce timeouts and output bounding; surface limits via `capabilities`.
        - Implement a safe external tool availability probe for external runners.
        - Add tests for timeouts, failure modes, and diagnostics schema conformance.
    - Notes: Async/pending mode is planned later alongside job tooling.
    - Status: planned

22. [ ] Implement `file_patch` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-017"
    - Description: Apply deterministic single-file patches within the workspace boundary, defaulting to dry-run.
    - Deliverables:
        - Implement `file_patch` per `docs/design/ToolContracts.md`.
        - Enforce workspace boundary and atomic write semantics.
        - Add tests for dry-run, conflicts, and boundary enforcement.
    - Status: planned

23. [ ] Implement `multi_file_patch` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-017"
    - Description: Apply deterministic patches across multiple files with per-file conflict reporting.
    - Deliverables:
        - Implement `multi_file_patch` per `docs/design/ToolContracts.md`.
        - Ensure deterministic per-file results, including conflict objects.
        - Add tests for partial conflicts and dry-run behavior.
    - Status: planned

24. [ ] Implement `file_create` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-017"
    - Description: Create files deterministically within the workspace boundary.
    - Deliverables:
        - Implement `file_create` per `docs/design/ToolContracts.md`.
        - Add tests for existing targets, parents missing, and dry-run.
    - Status: planned

25. [ ] Implement `file_append` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-004", "CNS-017"
    - Description: Append to files deterministically within the workspace boundary.
    - Deliverables:
        - Implement `file_append` per `docs/design/ToolContracts.md`.
        - Add tests for dry-run, missing targets, and deterministic byte counts.
    - Status: planned

26. [ ] Implement `file_delete` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015"
    - Description: Delete files deterministically, defaulting to dry-run.
    - Deliverables:
        - Implement `file_delete` per `docs/design/ToolContracts.md`.
        - Add tests for missing paths and dry-run behavior.
    - Status: planned

27. [ ] Implement `dir_create` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015"
    - Description: Create directories deterministically within the workspace boundary.
    - Deliverables:
        - Implement `dir_create` per `docs/design/ToolContracts.md`.
        - Add tests for parents semantics and dry-run.
    - Status: planned

28. [ ] Implement `dir_delete` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015"
    - Description: Delete empty directories deterministically, defaulting to dry-run.
    - Deliverables:
        - Implement `dir_delete` per `docs/design/ToolContracts.md`.
        - Add tests for non-empty directories and dry-run behavior.
    - Status: planned

29. [ ] Implement `path_move` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015", "CNS-017"
    - Description: Perform deterministic bulk move/rename operations with explicit overwrite behavior.
    - Deliverables:
        - Implement `path_move` per `docs/design/ToolContracts.md`.
        - Add tests for `overwrite: false` target exists warnings, boundary enforcement, and parents creation.
    - Status: planned

30. [ ] Implement `path_copy` (sync-only initial implementation; gated by config)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015"
    - Description: Perform deterministic bulk copy operations with bounded synchronous behavior.
    - Deliverables:
        - Implement the synchronous path for `path_copy` per `docs/design/ToolContracts.md`.
        - Ensure symlinks are copied as leaf nodes and not followed.
        - Add tests for symlink handling, overwrite semantics, and deterministic warnings.
    - Notes: Async/pending mode is planned later alongside job tooling.
    - Status: planned

31. [ ] Implement `file_create_bytes` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015"
    - Description: Create files from base64-encoded bytes with bounded per-call limits.
    - Deliverables:
        - Implement `file_create_bytes` per `docs/design/ToolContracts.md`.
        - Enforce per-call decoded byte bounds and publish limits in `capabilities.limits`.
        - Add tests for invalid base64, size bounds, and dry-run.
    - Status: planned

32. [ ] Implement `file_append_bytes` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015"
    - Description: Append base64-encoded bytes to a file with bounded per-call limits.
    - Deliverables:
        - Implement `file_append_bytes` per `docs/design/ToolContracts.md`.
        - Enforce per-call decoded byte bounds and publish limits in `capabilities.limits`.
        - Add tests for invalid base64, size bounds, and dry-run.
    - Status: planned

33. [ ] Implement `path_delete` (gated by config; default disabled)
    - Priority: 2
    - Cards: "STR-003", "CNS-001", "CNS-003", "CNS-015"
    - Description: Perform deterministic bulk delete operations with explicit recursion behavior.
    - Deliverables:
        - Implement `path_delete` per `docs/design/ToolContracts.md`.
        - Ensure symlinks are deleted as leaf nodes and not followed.
        - Add tests for non-empty directory warnings, recursion, and dry-run.
    - Status: planned

34. [ ] Implement `session_recent` (deferred)
    - Priority: 3
    - Cards: "STR-014", "CNS-015"
    - Description: Provide deterministic session introspection over recent tool calls and results (redacted and bounded).
    - Deliverables:
        - Implement `session_recent` per `docs/design/ToolContracts.md`.
        - Add tests for redaction, payload bounding, and pagination.
    - Status: planned

35. [ ] Implement `chain` (deferred)
    - Priority: 3
    - Cards: "STR-011", "CNS-009", "CNS-015"
    - Description: Execute deterministic tool chains with stop-on-failure defaults.
    - Deliverables:
        - Implement `chain` per `docs/design/ToolContracts.md`.
        - Add tests covering failure propagation and deterministic step results.
    - Status: planned

36. [ ] Implement background job tooling and async-only tools (deferred)
    - Priority: 3
    - Cards: "ART-006", "CNS-019", "STR-006", "STR-008"
    - Description: Add background job tools (`job_status`, `job_statuses`, `job_cancel`, `job_list`, `job_read`) and implement async-only tools like `log_tail`.
    - Deliverables:
        - Implement job tool contracts per `docs/design/ToolContracts.md`.
        - Implement `log_tail` as an async/pending log tail that streams via `job_read`.
        - Add tests for job cursor semantics, cancellation behavior, and bounded streaming.
    - Status: planned
