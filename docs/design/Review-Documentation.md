# Documentation Review — Alfred

**Date:** 2026-03-03
**Scope:** `docs/design/` markdown files (excluding `MIS-*` generated outputs and `aurora/`).
**Checklist:** [Documentation Review Checklists and Principles](../../.github/skills/DocumentationChecklistsAndPrinciples.md)

---

## Summary

| Severity | Count |
| -------- | ----- |
| P1       | 1     |
| P2       | 5     |
| P3       | 1     |

---

## Accuracy findings

### DOC-01 — Internal contradiction: `total_memory` / `indexed_memories` field names (P1)

**Checklist item:** No Internal Contradictions; Claims Are Verifiable.

**Status:** Fail

**Evidence:**

- `docs/design/ProjectPlan.md` item 44 specifies `memory.memories: integer` as the field to add to the `status` output.
- `docs/design/ToolContracts.md` (status section) specifies `memory.indexed_memories: integer`.

These are the same intended field but with different names. `ToolContracts.md` is the authoritative tool contract; `ProjectPlan.md` is a work-tracking artifact. The plan item uses a shorter alias that disagrees with the spec.

**Risk:** An implementer following the project plan will produce `memory.memories` while the spec demands `memory.indexed_memories`, producing a divergence between implementation and the authoritative contract.

**Smallest safe fix:** Correct plan item 44's deliverable to say `memory.indexed_memories: integer` to align with `ToolContracts.md`.

**Verification guidance:** Both documents use the same field name. A conformance test asserting `status` output passes.

---

### DOC-02 — `ToolContracts.md` plan section uses pre-item-46 operation names (P2)

**Checklist item:** API and Interface Fidelity; Claims Are Verifiable.

**Status:** Fail

**Evidence:** `docs/design/ToolContracts.md` (plan section):

```
`operation`: one of `"retrieve" | "add" | "update_status" | "remove"`.
```

Plan item 46 targets a canonical rename: `create | retrieve | update_status | delete`. The plan section of `ToolContracts.md` therefore documents names that are targeted for replacement but has not been updated. Furthermore, the current implementation in `src/tools/plan.rs` echoes `"get"` (not `"retrieve"`) and `"delete"` (not `"remove"`), creating a three-way mismatch: code ≠ current spec ≠ target spec.

**Risk:** Any reader of `ToolContracts.md` who calls the plan tool and pattern-matches on `"retrieve"` or `"remove"` in the echoed operation will receive `"get"` or `"delete"` respectively — an invisible failure.

**Smallest safe fix:** Per plan item 46: update `ToolContracts.md` to use `create | retrieve | update_status | delete` as the canonical set and update the implementation to match. Until then, add a forward reference in the existing plan section noting that operation names are being revised per item 46.

**Verification guidance:** The plan section of `ToolContracts.md` and implementation output match the same set of operation names.

---

### DOC-03 — `ToolContracts.md` memory section is correct; `MemoryFact` implementation diverges (P2)

**Checklist item:** API and Interface Fidelity; Examples Reflect Reality.

**Status:** Fail (documentation is correct; implementation diverges — tracked here for traceability)

**Evidence:** `docs/design/ToolContracts.md` memory fact shape correctly specifies `reasoning: optional string` with no `citations` field. The deployed code in `src/services/memory_store.rs:35,43` uses `reason` and includes `citations`. This divergence means that documentation accurately describes the intended API but the running system does not conform to it.

**Risk:** A developer reading the design doc and testing against the live server will observe different field names than specified, eroding documentation trust.

**Smallest safe fix:** See code finding CODE-05 and plan item 47. The documentation does not need to change; the implementation does.

**Verification guidance:** Memory create/retrieve/search responses serialize `reasoning` (not `reason`); no `citations` field appears.

---

### DOC-04 — `ToolContracts.md` patch section documents `"patch"/"multi-patch"` but implementation uses `"apply"` (P2)

**Checklist item:** API and Interface Fidelity.

**Status:** Fail

**Evidence:** `docs/design/ToolContracts.md` (patch section) specifies `operation` as one of `"patch" | "multi-patch" | "revert"`. The implementation in `src/tools/patch.rs` only accepts `"apply"` and `"revert"`, neither matching `"patch"` nor `"multi-patch"`. No migration note or alias is documented.

**Risk:** A developer or agent following `ToolContracts.md` will call `operation: "patch"` and receive an `invalid_argument` error with no clear indication of the correct value.

**Smallest safe fix:** Add a note to `ToolContracts.md` referencing plan item 43 and stating that `"patch"/"multi-patch"` are the intended accepted names once that item is resolved. The implementation must be corrected per plan item 43.

**Verification guidance:** `ToolContracts.md` and the implementation agree on accepted `operation` values.

---

### DOC-05 — `ToolContracts.md` `plan` section accepts `"retrieve"` but implementation echoes `"get"` (P2)

**Checklist item:** API and Interface Fidelity; No Internal Contradictions.

**Status:** Fail

**Evidence:** `docs/design/ToolContracts.md` lists `"retrieve"` as an accepted operation but the implementation (`src/tools/plan.rs:29, 40`) accepts `"retrieve"` as a silent alias that still echoes `"get"`. The contract says the canonical name is `"retrieve"`; the code violates that by echoing `"get"`.

**Risk:** Clients that verify the echoed `operation` field against `"retrieve"` (as documented) will receive `"get"` and experience a dispatch or routing failure.

**Smallest safe fix:** Correct the echo value in `PlanOperation::as_str`. This is part of plan item 46; the fix SHOULD land together.

**Verification guidance:** Calling `plan` with `operation: "retrieve"` returns `"operation": "retrieve"` in the response.

---

### DOC-06 — `ToolContracts.md` status section does not document `pid` or `workspace_index_root` output fields (P2)

**Checklist item:** API and Interface Fidelity; Examples Reflect Reality.

**Status:** Fail

**Evidence:** The current `status` implementation in `src/tools/status.rs` emits `pid` (process id) and `paths.workspace_index_root` at the top level. Neither field is listed in the `ToolContracts.md` status output spec (which lists only `version`, `total_memory`, `workspace_root`, `paths.plan`, `paths.alfred_logs`, `index.*`, `memory.*`).

**Risk:** Clients that consume `pid` or `paths.workspace_index_root` are relying on undocumented fields that may change without a compatibility notice.

**Smallest safe fix:** Either (a) add `pid` and `paths.workspace_index_root` to the `ToolContracts.md` status output spec with a note on their semantics, or (b) remove them from the implementation as part of plan item 44. Record the decision explicitly.

**Verification guidance:** Every field emitted by `status` appears in `ToolContracts.md` and every field documented in `ToolContracts.md` is emitted by the implementation.

---

## Clarity findings

### DOC-07 — `AlfredOverview.md` contains a typo (P3)

**Checklist item:** Concise Wording; Actionable Steps.

**Status:** Fail

**Evidence:** `docs/design/AlfredOverview.md` line 20: `"Alfred MUST provide commanst to create and maintain a project plan"` — `commanst` is not a word; the intended word is `commands`.

**Risk:** Minor reader confusion; no functional impact.

**Smallest safe fix:** Correct the typo to `commands`.

**Verification guidance:** File does not contain `commanst`.

---

## Passed items

| Checklist item                | Evidence                                                                                                                                             |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Links resolve correctly       | Spot-checked cross-document links in `ToolContracts.md`, `Protocol.md`, and `Redaction.md`; all target files exist at the referenced relative paths. |
| No secrets in examples        | No configuration examples or sample outputs in any design document contain placeholder values that could be mistaken for real secrets.               |
| Heading hierarchy             | All reviewed documents use ATX headings without skipping levels.                                                                                     |
| Failure paths covered         | `ErrorTaxonomy.md` documents all error kinds with retry guidance and `details.reason` discriminators.                                                |
| Redaction contract documented | `Redaction.md` gives a complete, deterministic algorithm specification.                                                                              |
