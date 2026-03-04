# Code Review — Alfred

**Date:** 2026-03-03
**Scope:** All source files under `src/` and `benches/`.
**Checklist:** [Code Review Checklists and Principles](../../.github/skills/CodeChecklistsAndPrinciples.md)

---

## Summary

| Severity | Count |
| -------- | ----- |
| P1       | 6     |
| P2       | 6     |
| P3       | 1     |

---

## Findings

### CODE-01 — File size limit violations (P1)

**Checklist item:** File Length — source files SHALL NOT exceed 500 lines.

**Status:** Fail

**Evidence:** The following files exceed the 500-line hard limit:

| File                               | Lines | Over limit by |
| ---------------------------------- | ----- | ------------- |
| `src/configuration.rs`             | 1 732 | 3.5×          |
| `src/services/indexer.rs`          | 1 333 | 2.7×          |
| `src/tools/fs.rs`                  | 908   | 1.8×          |
| `src/services/job_manager.rs`      | 775   | 1.6×          |
| `src/services/memory_store.rs`     | 655   | 1.3×          |
| `src/services/plan_store.rs`       | 596   | 1.2×          |
| `src/tools/memory.rs`              | 570   | 1.1×          |
| `src/protocol.rs`                  | 569   | 1.1×          |
| `src/redaction.rs`                 | 509   | 1.0×          |
| `src/services/workspace_ignore.rs` | 504   | 1.0×          |

**Risk:** God-module anti-pattern; each file becomes a change hotspot and increases the cost of review and reasoning. Functions buried inside large files are harder to test in isolation.

**Smallest safe fix:** Decompose each oversized module into cohesive sub-modules. For example, `configuration.rs` can be split into `configuration/loading.rs`, `configuration/storage.rs`, `configuration/redaction.rs`, `configuration/memory.rs`, and `configuration/paths.rs`.

**Verification guidance:** `wc -l src/**/*.rs` returns no line above 500 after refactoring; all existing tests continue to pass.

---

### CODE-02 — Protocol envelope shape deviates from `Protocol.md` (P1)

**Checklist item:** Correctness — code aligns to design documentation.

**Status:** Fail

**Evidence:**

- `ToolResponse::Error` serializes as `{"status":"error","error":{…}}` (`error` singular). `Protocol.md` requires `errors` (non-empty array). (`src/protocol.rs:60`)
- `warnings` is nested inside `meta` both in `ToolMeta` (`src/protocol.rs:34`) and in `inject_warning` (`src/protocol.rs:453`). `Protocol.md` requires `warnings` as a **top-level** sibling of `status`, `data`, and `meta`.
- `ToolResponse::Pending` carries a non-spec top-level `job_id` field (`src/protocol.rs:64`). `Protocol.md` requires `data.operation_id`, `data.state`, `data.poll_with: "fs"` with no `job_id` at the envelope level.

**Risk:** Every client that follows `Protocol.md` will fail to parse error and warning fields correctly.

**Smallest safe fix:** See plan item 42 for the correction list.

**Verification guidance:** Protocol conformance tests assert `errors` array, top-level `warnings`, and `data.operation_id` / `data.state` / `data.poll_with` without `job_id`.

---

### CODE-03 — `patch` API diverges from `ToolContracts.md` (P1)

**Checklist item:** Correctness — code aligns to design documentation.

**Status:** Fail

**Evidence:**

- `PatchOperation::parse` accepts `"apply"` and echoes `"apply"`. `ToolContracts.md` specifies `"patch"` (single file) and `"multi-patch"` (multiple files) as distinct accepted names. Neither is accepted. (`src/tools/patch.rs:20-30`)
- There is no per-file `patch_id` UUID in apply results. The design requires `patch_id: string (uuid)` in every per-file result.
- `revert` calls `revert_last_patch` (global state, not per-id). The design requires `args.patch_ids: array<uuid>` and per-id stored reverse patches. (`src/tools/patch.rs:108-119`)

**Risk:** Agents following `ToolContracts.md` cannot invoke `patch` or `revert` correctly and have no stable reference with which to revert a specific applied batch.

**Smallest safe fix:** See plan item 43.

**Verification guidance:** Conformance tests verify `"patch"`, `"multi-patch"`, and `"revert"` operation names; assert `patch_id` UUID in each file result; assert `patch_ids` array input is accepted by `revert`.

---

### CODE-04 — `plan` operation names echo non-canonical values (P1)

**Checklist item:** Correctness — code aligns to design documentation.

**Status:** Fail

**Evidence:**

- `PlanOperation::parse` accepts `"get"` and `"retrieve"` but always echoes `"get"`. `ToolContracts.md` names the canonical accepted value as `"retrieve"`. (`src/tools/plan.rs:29-36`, `src/tools/plan.rs:40`)
- `PlanOperation::parse` accepts `"delete"` and `"remove"` but echoes `"delete"`. `ToolContracts.md` names `"remove"` as the canonical value. (`src/tools/plan.rs:32-33`, `src/tools/plan.rs:43`)
- The validation error message lists `"get, add, update_status, delete"` rather than the canonical `"retrieve, add, update_status, remove"`. (`src/tools/plan.rs:34-37`)

**Risk:** Clients that echo-match the returned `operation` field against the documented canonical names will fail deterministic routing.

**Smallest safe fix:** See plan item 46 for the updated canonical name set. Correct the echo values and error message together.

**Verification guidance:** Plan tool tests assert that `"retrieve"` echoes `"retrieve"`, `"remove"` echoes `"remove"` (or whichever canonical names are chosen per plan item 46), and the error message matches the accepted name list.

---

### CODE-05 — Memory store field names diverge from `ToolContracts.md` (P1)

**Checklist item:** Correctness — code aligns to design documentation.

**Status:** Fail

**Evidence:**

- `MemoryFact.reason` and `MemoryFactInput.reason` persist and return data as `"reason"`. `ToolContracts.md` memory fact shape specifies `"reasoning"`. (`src/services/memory_store.rs:35`, `src/services/memory_store.rs:43`)
- `MemoryFact.citations` and `MemoryFactInput.citations` exist in the persisted schema and in the memory search scoring path (`src/tools/memory.rs:341`). `ToolContracts.md` does not include `citations` in the memory fact shape and it was removed from the public design.
- `MemoryCreateArgs` and `MemoryUpdateArgs` accept both `reasoning` and `reason` (dual-field fallback) which creates a stringly-typed ambiguity. (`src/tools/memory.rs:80-90`, `src/tools/memory.rs:97-107`)

**Risk:** Serialized memory facts in the store have the wrong field names. Any client that stores or searches `reasoning` will get no match against the underlying `reason` field.

**Smallest safe fix:** See plan item 47. Migration of existing stored facts may be required.

**Verification guidance:** Memory tests assert `"reasoning"` (not `"reason"`) in create, retrieve, update, and search results. No `"citations"` field appears in any output.

---

### CODE-06 — `workspace_boundary::ensure_within_root` uses wrong error kind (P1)

**Checklist item:** Correctness — behavior must match `ErrorTaxonomy.md`.

**Status:** Fail

**Evidence:** `ensure_within_root` in `src/workspace_boundary.rs:26-29` returns `AlfredError::PermissionDenied` for a boundary escape. `ErrorTaxonomy.md` specifies that boundary escapes use kind `workspace_boundary_violation`.

**Risk:** Clients that gate retry or downgrade logic on `workspace_boundary_violation` errors will not detect directory-traversal or symlink-escape attempts and may retry unnecessarily.

**Smallest safe fix:** Change the return to `AlfredError::WorkspaceBoundaryViolation` with a `details: {"reason":"workspace_boundary_violation"}` payload. Verify the same change is applied to all callers of the workspace boundary module.

**Verification guidance:** Integration tests for path traversal and symlink-escape attempts assert `kind: "workspace_boundary_violation"` in the error response.

---

### CODE-07 — `status` output shape diverges from `ToolContracts.md` (P2)

**Checklist item:** Correctness — output fields must match design.

**Status:** Fail

**Evidence:**

- `paths.runtime_log` should be `paths.alfred_logs`. (`src/tools/status.rs:64`)
- `total_memory` (integer, process RSS bytes) is absent from the top-level output.
- `memory.ready` (bool) and `memory.indexed_memories` (integer) are absent.
- The verbose memory breakdown is embedded directly in the `memory` key; per plan item 44 it should move to a `system_memory` key, freeing `memory` for the spec-compliant shape.

**Risk:** Clients following `ToolContracts.md` will find a missing or misnamed field causing parse failures or silent nil values.

**Smallest safe fix:** See plan item 44.

**Verification guidance:** Status tool tests assert all required output fields with their exact names as specified in `ToolContracts.md`.

---

### CODE-08 — Dead code: `"log_search"` branch in `logs::dispatch_tool_call` (P2)

**Checklist item:** Minimal Surface Area; Single Responsibility Principle.

**Status:** Fail

**Evidence:** `src/tools/logs.rs:82-83` contains a match arm for `"log_search"`. This arm is unreachable because the top-level dispatcher in `src/tools.rs` calls `is_deprecated_tool_name` before dispatching to any tool group, and `"log_search"` is listed in `DEPRECATED_TOOL_NAMES`. Every call to `"log_search"` is refused with a `tool_disabled` error before reaching `logs::dispatch_tool_call`.

**Risk:** Dead code inflates the apparent code surface, misleads reviewers into thinking `"log_search"` is live, and may regress if the deprecated-name guard is ever bypassed.

**Smallest safe fix:** Remove the `"log_search"` match arm from `logs::dispatch_tool_call`.

**Verification guidance:** `cargo check` passes; a test asserting that `"log_search"` returns `tool_disabled` continues to pass.

---

### CODE-09 — `tool_disabled_error` omits the `"tool"` field from `details` (P2)

**Checklist item:** Correctness — error details must match `ErrorTaxonomy.md`.

**Status:** Fail

**Evidence:** `tool_disabled_error` in `src/tools.rs:151-156` constructs `details: {"reason":"tool_disabled"}`. `ErrorTaxonomy.md` specifies `details: {"reason":"tool_disabled","tool":"<tool_name>"}`.

**Risk:** Clients cannot identify which tool was disabled without parsing the human-readable message, breaking deterministic client-side downgrade/retry logic.

**Smallest safe fix:** Add `"tool": name` to the details object in `tool_disabled_error`.

**Verification guidance:** Test for a disabled tool asserts that the error details contain both `"reason": "tool_disabled"` and `"tool": "<name>"`.

---

### CODE-10 — `escargot` is an unused production dependency (P2)

**Checklist item:** Continuously Maintain Dependencies — dependency inputs must be minimal and reviewed.

**Status:** Fail

**Evidence:** `escargot = "0.5.15"` appears in `[dependencies]` in `Cargo.toml` with no usages in any `.rs` file in the codebase. It is a Cargo process-building library suitable only for integration tests.

**Risk:** Unnecessary production dependencies expand the supply-chain attack surface. `escargot` invokes `cargo` subprocesses and is inappropriate as a production runtime dependency.

**Smallest safe fix:** Remove the `escargot` entry from `[dependencies]`. If it is needed for future integration tests, add it to `[dev-dependencies]`.

**Verification guidance:** `cargo build` succeeds without the `escargot` entry. No functionality is lost.

---

### CODE-11 — `tools/list` response returns stub descriptions and no input schema (P3)

**Checklist item:** Correctness; API and Interface Fidelity.

**Status:** Fail

**Evidence:** `build_tools_list_response` in `src/protocol.rs:267-281` generates `"description": "Alfred tool: {name}"` for every tool and uses `"inputSchema": {"type":"object","additionalProperties":true}` (no real schema). MCP tooling clients that rely on descriptions for tool selection receive no useful guidance.

**Risk:** Agents using `tools/list` for tool discovery cannot make informed selections. The `additionalProperties: true` schema provides no constraint on inputs, defeating schema-driven validation at the client layer.

**Smallest safe fix:** Define per-tool description strings and at minimum a minimal required-properties input schema for each tool, matching the contracts in `ToolContracts.md`.

**Verification guidance:** `tools/list` response for each tool includes a non-trivial description and a schema that lists at minimum the required `operation` property.

---

## Non-applicable items

| Checklist item           | Rationale                                                                                                                                             |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Accessibility (WCAG AA)  | Alfred is a CLI/stdio server with no user-facing UI.                                                                                                  |
| Function size (50 lines) | A per-function survey was not exhaustive within this review pass; file-level decomposition (CODE-01) is the prerequisite for function-level analysis. |
