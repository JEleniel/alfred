# Plan: Implement remaining Alfred commands

This plan is aligned to the current design artifacts in `docs/design/` (notably `AlfredOverview.md`, `ToolContracts.md`, `Protocol.md`, `Configuration.md`, and `Redaction.md`). The public tool surface is intentionally consolidated; items 1–16 are retained for historical traceability, while items 17+ track the consolidated tool surfaces and any newly specified cross-cutting requirements.

## Item format

```markdown
N. [ ] P{priority}: {title} - {One-sentence description of the gap or goal; context reference.} - Deliverable: specific, verifiable action or outcome. - Deliverable: ... - Aurora cards: [ID](aurora/MIS-001/{Category}/{ID}-{Name}.json) ... - Dependencies: N (omit if none)
```

- **Priority** is an integer; lower is higher priority (`P0` ships before `P1`, etc.).
- The **description** line is a single sentence; it may cite a design doc but does not list deliverables.
- **Deliverables** are indented one level deeper than the description and are each independently verifiable.
- **Aurora cards** link to the relevant model cards relative to this file's location.
- **Dependencies** lists item numbers that must be complete before this item can start.
- Completed items (`[x]`) retain only their header line; detail is dropped when marked done.

1. [x] Conform MCP stdio interface envelope and error taxonomy

2. [x] Implement configuration loading and deterministic policy guardrails

3. [x] Implement `capabilities` tool and make `tools/list` truthful

4. [x] Consolidate workspace query tool surface into `fs`

5. [x] Implement `log_search`

6. [x] Implement `plan_get`

7. [x] Implement `plan_update`

8. [x] Implement `plan_edit`

9. [x] Implement `plan_add`

10. [x] Implement `plan_delete`

11. [x] Implement `memory_put`

12. [x] Implement `memory_get`

13. [x] Implement `memory_delete`

14. [x] Implement `memory_list`

15. [x] Implement `memory_search`

16. [x] Refine architecture for consolidated command surface

17. [x] Implement consolidated `search` command

18. [x] Implement consolidated `fs` command

19. [x] Implement consolidated `patch` command

20. [x] Implement `fs` bulk background execution

21. [x] Implement consolidated `logs` command

22. [x] Implement consolidated `plan` command

23. [x] Implement consolidated `memory` command

24. [x] Implement workspace storage-root and index-location controls

25. [x] Retire deprecated orchestration scope

26. [x] Implement deterministic redaction end-to-end

27. [x] Implement encoding-safe path handling and deterministic warnings

28. [x] Retire byte-oriented filesystem operations from the exposed tool surface

29. [x] Add a conformance suite for consolidated tool contracts

30. [x] Reconcile generated Aurora model outputs with the consolidated tool surface

31. [x] Add `status` tool

32. [x] Add MCP prompt support

33. [x] Prefer patch-first internal mutations

34. [x] Implement storage location controls for workspace and user artifacts

35. [x] Implement runtime log path selection and stable log naming

36. [x] Remove SQLite naming and dependency remnants

37. [x] Enforce single-location persistence for all active indexes

38. [x] Retire deprecated tool families from the public surface

39. [x] Enforce symlink/junction-safe workspace boundary checks

40. [x] Implement network-filesystem safe write strategy

41. [x] Enforce and test capability limits

42. [ ] P0: Fix protocol envelope shape to match design
    - Several deviations from `docs/design/Protocol.md` were introduced during implementation; correct all tools to return conforming shapes.
        - Change `status: "error"` from singular `error` to `errors` (non-empty array of error objects).
        - Move `warnings` to a top-level envelope field (sibling of `status`, `data`, `meta`), not nested inside `meta`.
        - Fix `status: "pending"`: add `data.operation_id`, `data.state`, `data.poll_with: "fs"`; remove non-spec `job_id`.
        - Update `inject_warning` in `src/protocol.rs` to write to the top-level `warnings` array.
        - Update all conformance and protocol tests to assert the corrected shapes.
    - Aurora cards: [INT-001](aurora/MIS-001/Interface/INT-001-MCP_Stdio_Interface.json) [ART-001](aurora/MIS-001/Artifact/ART-001-MCP_Request.json) [ART-002](aurora/MIS-001/Artifact/ART-002-MCP_Response.json) [CNS-005](aurora/MIS-001/Constraint/CNS-005-Deterministic_Error_Taxonomy.json)

43. [ ] P1: Fix `patch` per-file `patch_id` assignment and reversion contract
    - `patch` assigns no per-file `patch_id`, stores no per-id revert state, and does not accept `patch_ids`-based reversion; correct to match `docs/design/ToolContracts.md`.
        - Assign a UUID `patch_id` to every file result from `patch` (single and multi-patch).
        - Store the reverse patch keyed per `patch_id`; any new apply call replaces all prior revert state.
        - Change `revert` input to `args.patch_ids: array<uuid>`; apply reversal for each id, then discard that id's state.
        - Return `patch_id` in revert per-file result objects.
        - Echo `"patch"` or `"multi-patch"` for apply operations; echo `"revert"` for reversal.
        - Update `src/tools/patch.rs`, `src/services/file_ops.rs`, and conformance tests.
    - Aurora cards: [STR-003](aurora/MIS-001/Story/STR-003-Edit_Files_Safely.json) [CNS-017](aurora/MIS-001/Constraint/CNS-017-Cross_Platform_Atomic_Write_Semantics.json)

44. [ ] P2: Fix `status` output shape
    - As-built `status` response diverges from `docs/design/ToolContracts.md` in field names and structure.
        - Add top-level `total_memory` scalar (integer bytes; process RSS) present for all calls regardless of `verbose`.
        - Rename the memory subsystem block to `system_memory` (containing existing `system.*` and `process.*` breakdown).
        - Add `memory.ready: bool` and `memory.indexed_memories: integer` (count of indexed facts across enabled scopes).
        - Rename `paths.runtime_log` to `paths.alfred_logs`.
        - Decide whether `pid` and `paths.workspace_index_root` are retained (and documented in `ToolContracts.md`) or removed; apply the chosen outcome consistently.
        - Retain `verbose`-only fields (`system_load`, `cpu_count`) inside `system_memory`.
        - Update `src/tools/status.rs` and conformance tests.
    - Aurora cards: [STR-003](aurora/MIS-001/Story/STR-003-Edit_Files_Safely.json) [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)

45. [ ] P1: Implement `logs.follow` as a true streaming operation
    - The current `follow` handler returns a single synchronous snapshot; replace with ongoing incremental emission per `docs/design/Protocol.md` streaming semantics.
        - After emitting the initial `tail` records, continue reading and emitting new records as the log file grows until a stop signal.
        - Each emission MUST use the standard envelope (`status: "ok"`, `operation: "follow"`, `result.records`, `result.stopped`).
        - Streaming MUST remain MCP-compliant (valid JSON-RPC/MCP frames only; no raw NDJSON on stdio).
        - Retain the at-most-one-active-stream invariant.
        - `stop: true` terminates the stream and MUST return `result.stopped: true`.
        - Update `docs/design/ToolContracts.md` `follow` behavioral notes if needed.
        - Add or extend tests covering stream start, incremental emission, and stop.
    - Aurora cards: [STR-008](aurora/MIS-001/Story/STR-008-Handle_Logs_Predictably.json) [CNS-012](aurora/MIS-001/Constraint/CNS-012-Service_Level_Objectives.json) [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)
    - Dependencies: 21

46. [ ] P1: Standardize `plan` operation names to `create / retrieve / update_status / delete`
    - `plan` tool echoes non-canonical names (`get`, `add`, `delete`); standardize accepted input and echoed `operation` field to match `docs/design/ToolContracts.md`.
        - Update dispatch to accept and echo: `create` (was `add`), `retrieve` (was `get`), `update_status`, `delete` (was `remove`).
        - Remove undocumented aliases, or retain as silent aliases that still echo the canonical name.
        - Update `docs/design/ToolContracts.md` to use `create | retrieve | update_status | delete` throughout the `plan` section.
        - Update conformance and plan tests to use and assert the canonical names.
    - Aurora cards: [ART-004](aurora/MIS-001/Artifact/ART-004-Project_Plan.json) [STR-004](aurora/MIS-001/Story/STR-004-Track_a_Project_Plan.json) [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)

47. [ ] P1: Rename `reasoning` internal field and remove `citations` from implementation
    - Memory store uses `reason` (not `reasoning`) internally and includes a `citations` field removed from the public design; both leak into serialized data and search scoring.
        - Rename `MemoryFact.reason` and `MemoryFactInput.reason` to `reasoning` in `src/services/memory_store.rs`.
        - Remove `MemoryFact.citations`, `MemoryFactInput.citations`, and all references (schema fields, indexing, redaction, serialization).
        - Update `src/tools/memory.rs`: remove `citations` from `MemoryCreateArgs` and `MemoryUpdateArgs`; map `reasoning` end-to-end.
        - Ensure `memory_output` serializes as `"reasoning"` throughout.
        - Update conformance, persistence, and memory tool tests for `citations` and `reason` references.
        - Remove any remaining `citations` references from design or configuration docs.
    - Aurora cards: [ART-008](aurora/MIS-001/Artifact/ART-008-Memory_Fact.json) [STR-017](aurora/MIS-001/Story/STR-017-Remember_Facts_Offline.json) [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)

48. [ ] P2: Add `--reset` CLI option to delete saved indexes and memories
    - Provide a `--reset` flag that deletes persisted workspace and user artifacts before starting normally, enabling a clean-state startup without hand-deletion.
        - Add a `--reset` CLI flag to `src/main.rs` (or the argument parser).
        - When supplied, delete the workspace index directory, workspace memory store directory, and (if configured) the user memory store directory before accepting requests.
        - Log a structured record for each artifact deleted.
        - Do not delete configuration files or log files.
        - Add tests verifying reset deletes expected paths and the server starts cleanly afterward.
    - Aurora cards: [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)

49. [ ] P1: Fix workspace boundary escape error kind
    - `workspace_boundary::ensure_within_root` returns `AlfredError::PermissionDenied` for symlink/junction escapes; the correct kind per `docs/design/ErrorTaxonomy.md` is `workspace_boundary_violation`.
        - Change the return in `src/workspace_boundary.rs::ensure_within_root` to `AlfredError::WorkspaceBoundaryViolation` with `details: {"reason":"workspace_boundary_violation"}`.
        - Audit all other callers of workspace boundary helpers to confirm they surface the correct kind.
        - Update or add integration tests that assert `kind: "workspace_boundary_violation"` for symlink-escape and traversal attempts.
    - Aurora cards: [CNS-016](aurora/MIS-001/Constraint/CNS-016-Symlink_and_JunctionSafe_Boundary_Checks.json) [CNS-005](aurora/MIS-001/Constraint/CNS-005-Deterministic_Error_Taxonomy.json)

50. [ ] P1: Apply redaction at memory store ingestion
    - Memory facts are stored verbatim in the Tantivy index shards and NDJSON backing files; `Redaction.md` requires redaction at ingestion, not only at output time.
        - Apply the `Redactor` to all user-supplied string fields of `MemoryFactInput` (`fact`, `reasoning`, `subject`) inside `upsert_in_scope` before writing to the index and store.
        - Ensure the `Redactor` instance is accessible to the `MemoryStore` (via constructor injection or a shared reference).
        - Add tests confirming that a fact created with a bearer-token pattern stores and retrieves the redaction token, not the original secret.
        - Verify the NDJSON backing file also does not contain the original secret.
    - Aurora cards: [CNS-009](aurora/MIS-001/Constraint/CNS-009-Deterministic_Redaction.json) [STR-017](aurora/MIS-001/Story/STR-017-Remember_Facts_Offline.json)

51. [ ] P1: Decompose oversized source modules
    - Ten source files exceed the 500-line hard limit from the code checklist, reducing reviewability and testability.
        - Decompose `src/configuration.rs` (1 732 lines) into sub-modules: `loading`, `storage`, `redaction`, `memory`, `paths`.
        - Decompose `src/services/indexer.rs` (1 333 lines) into at least two cohesive sub-modules.
        - Decompose `src/tools/fs.rs` (908 lines) into operation-specific sub-modules (read, write, bulk).
        - Decompose `src/services/job_manager.rs` (775 lines) into at least two sub-modules.
        - Decompose remaining oversized files (`memory_store.rs`, `plan_store.rs`, `memory.rs`, `protocol.rs`, `redaction.rs`, `workspace_ignore.rs`) to bring each under 500 lines.
        - All existing tests MUST continue to pass unchanged.
    - Aurora cards: [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)

52. [ ] P2: Redact OS username from startup log messages
    - The startup log messages in `src/app.rs` emit absolute workspace-root and config paths that contain the OS username; user names are non-public technical information and MUST NOT be logged.
        - In `src/app.rs`, replace full-path log arguments with workspace-relative paths or path tail segments (e.g. last two components) that do not expose the OS username.
        - Document the path-logging policy (no full absolute paths to workspace root) in a comment near the log call sites, referencing `docs/design/Redaction.md`.
        - Add a test that mounts Alfred against a workspace whose absolute path contains a known token and confirms the token does not appear in the runtime log.
    - Aurora cards: [CNS-009](aurora/MIS-001/Constraint/CNS-009-Deterministic_Redaction.json)

53. [ ] P2: Fix `tool_disabled_error` missing `"tool"` field in details
    - `tool_disabled_error` in `src/tools.rs` returns `details: {"reason":"tool_disabled"}`, omitting the `"tool"` field required by `docs/design/ErrorTaxonomy.md`.
        - Add `"tool": name` to the details object in `tool_disabled_error`.
        - Update conformance and protocol tests that assert the disabled-tool error shape.
    - Aurora cards: [CNS-005](aurora/MIS-001/Constraint/CNS-005-Deterministic_Error_Taxonomy.json)

54. [ ] P2: Remove unused `escargot` production dependency
    - `escargot = "0.5.15"` is listed in `[dependencies]` with no usages anywhere in the codebase; it is a process-spawning Cargo helper inappropriate for a production runtime dependency.
        - Remove the `escargot` entry from `[dependencies]` in `Cargo.toml`.
        - Confirm `cargo build` and `cargo test` both succeed after removal.
    - Aurora cards: [CNS-005](aurora/MIS-001/Constraint/CNS-005-Deterministic_Error_Taxonomy.json)

55. [ ] P3: Remove dead `"log_search"` branch in `logs::dispatch_tool_call`
    - The `"log_search"` match arm in `src/tools/logs.rs` is unreachable dead code because the top-level dispatcher refuses deprecated tool names before reaching any tool group.
        - Remove the `"log_search" =>` match arm from `logs::dispatch_tool_call`.
        - Confirm that callers of `"log_search"` still receive the expected `tool_disabled` error.
    - Aurora cards: [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)

56. [ ] P3: Add meaningful `tools/list` descriptions and input schemas
    - `build_tools_list_response` returns `"Alfred tool: {name}"` for all tools and `additionalProperties: true` with no real input schema; both weaken MCP client discoverability.
        - Define per-tool description strings in `src/protocol.rs` (or a companion module) sourced from `docs/design/ToolContracts.md`.
        - Add minimal input schemas listing at minimum the `operation` required property for each multi-operation tool.
        - Verify `tools/list` conformance tests pass with the updated descriptions and schemas.
    - Aurora cards: [INT-001](aurora/MIS-001/Interface/INT-001-MCP_Stdio_Interface.json) [CNS-015](aurora/MIS-001/Constraint/CNS-015-Deterministic_Output_Normalization.json)

57. [ ] P3: Fix typo in `AlfredOverview.md`
    - `docs/design/AlfredOverview.md` contains `"Alfred MUST provide commanst to create and maintain a project plan"`.
        - Correct `commanst` to `commands`.

58. [x] Correct defaults in `Defaults.md` for storage, memory, and logging

59. [x] Add canonical `StorageLayout.md` and cross-reference from `Configuration.md` and `Defaults.md`
