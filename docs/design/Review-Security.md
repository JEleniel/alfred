# Security Review — Alfred

**Date:** 2026-03-03
**Scope:** All source files under `src/`, configuration schemas, and design documentation in `docs/design/`.
**Checklist:** [Code Review Checklists and Principles](../../.github/skills/CodeChecklistsAndPrinciples.md) — Secure Code Checklist

---

## Summary

| Severity | Count |
| -------- | ----- |
| P1       | 2     |
| P2       | 3     |

---

## Findings

### SEC-01 — OS username is logged without redaction coverage (P2)

**Checklist item:** Handle Errors Without Information Leakage; Log and Audit Security-Relevant Events.

**Status:** Fail

**Evidence:**

- `src/app.rs:34`: `trace!("bootstrap config workspace={}", config.workspace_config_path.display())`
- `src/app.rs:69`: `info!("booted {} at {} with {} tools", …, self.config.workspace_root.display(), …)`
- `src/app.rs:111`: fallback workspace-hints log contains `self.config.workspace_root.display()`

On macOS and most Linux distributions, `workspace_root` and `workspace_config_path` are absolute paths that include the OS username (e.g. `/Users/<username>/repos/…` or `/home/<username>/repos/…`). The standing project requirement recorded in memory is: **user names are considered non-public technical information and must not be logged.**

The default redaction rules cover emails, bearer tokens, and specific structured field keys (password, secret, token, api_key, authorization, cookie). None of the default rules match path-embedded usernames.

**Risk:** Every Alfred-managed log file contains the OS username of the person running Alfred. The log files are readable by tools that process Alfred logs (including Alfred itself via the `logs` tool) and may be accidentally shared or indexed.

**Smallest safe fix:**

1. Wherever workspace-root or config-path are logged in `src/app.rs`, replace the full path with either (a) a workspace-relative representation or (b) only the last two components (`…/<parent>/<dir>`).
2. Document the path-logging policy in `docs/design/Redaction.md` so future log calls follow the same rule.

**Verification guidance:** Run Alfred in a workspace whose absolute path contains a known username string. Inspect the runtime log to confirm no occurrence of the username. Confirm existing log-tracing tests still pass.

---

### SEC-02 — `workspace_boundary::ensure_within_root` misclassifies boundary escapes as `PermissionDenied` (P1)

**Checklist item:** Trust Boundaries Are Explicit; All Input Untrusted.

**Status:** Fail

**Evidence:** `src/workspace_boundary.rs:26-29`:

```rust
Err(AlfredError::PermissionDenied(
    SYMLINK_JUNCTION_ESCAPE_MESSAGE.to_string(),
))
```

`ErrorTaxonomy.md` specifies that workspace boundary escapes (symlink/junction traversal) MUST use kind `workspace_boundary_violation`, not `permission_denied`. The two kinds have different security semantics:

- `permission_denied` signals an access-control decision.
- `workspace_boundary_violation` signals an attempt to escape the workspace, which a client can use to flag adversarial or misconfigured inputs.

**Risk:** Clients that gate on `workspace_boundary_violation` for alerting or hardening logic will silently miss a symlink-escape attempt. Monitoring or audit systems that track `permission_denied` errors will receive false positives that obscure the true nature of the event.

**Smallest safe fix:** Change `AlfredError::PermissionDenied(…)` to `AlfredError::WorkspaceBoundaryViolation(…)` in `ensure_within_root`. Add `details: {"reason":"workspace_boundary_violation"}` per the taxonomy. If any callers of `ensure_within_root` explicitly match on `PermissionDenied` for boundary scenarios, update them accordingly.

**Verification guidance:** A test that constructs a symlink pointing outside the workspace root and then calls `try_resolve_existing_path_within_workspace_root` asserts that the returned error has `kind: "workspace_boundary_violation"`.

---

### SEC-03 — `tool_disabled_error` details omit the `"tool"` field, breaking deterministic client gating (P2)

**Checklist item:** Handle Errors Without Information Leakage; Trust Boundaries Are Explicit.

**Status:** Fail

**Evidence:** `src/tools.rs:151-156`:

```rust
AlfredError::InvalidArgumentWithDetails {
    message: format!("tool disabled by policy: {name}"),
    details: Some(json!({"reason": "tool_disabled"})),
}
```

`ErrorTaxonomy.md` specifies `details: {"reason":"tool_disabled","tool":"<tool_name>"}`. The `"tool"` field is absent. Without it, enforcing policy gating on a specific tool name requires parsing the human-readable `message` string.

**Risk:** Any client that gates on `details.tool` to enforce allow/deny logic for specific tools cannot distinguish which tool triggered the error without brittle string parsing. This is a minor but real weakening of the deterministic policy contract.

**Smallest safe fix:** Add `"tool": name` to the `json!({…})` object in `tool_disabled_error`.

**Verification guidance:** A test calling a disabled tool asserts `details.tool == "<tool_name>"` and `details.reason == "tool_disabled"`.

---

### SEC-04 — `escargot` in production `[dependencies]` expands supply-chain surface (P2)

**Checklist item:** Continuously Maintain Dependencies; Minimal Surface Area.

**Status:** Fail

**Evidence:** `escargot = "0.5.15"` in `[dependencies]` (not `[dev-dependencies]`) with zero usages in any `.rs` file. `escargot` spawns `cargo` subprocesses at runtime; including it as a production dependency introduces a compile-time and runtime dependency on Cargo infrastructure that is never exercised in production.

**Risk:** An unmaintained or compromised version of an unused dependency with subprocess-invocation capabilities is a latent supply-chain risk. Lockfiles only mitigate, not eliminate, this.

**Smallest safe fix:** Remove `escargot` from `[dependencies]`. If needed for integration tests in future, add it to `[dev-dependencies]`.

**Verification guidance:** `cargo build --release` succeeds. `Cargo.lock` no longer contains the `escargot` entry.

---

### SEC-05 — `memory` store redaction not confirmed at search-result output (P1)

**Checklist item:** All Output Is Secured; Protect Data End-to-End.

**Status:** Fail

**Evidence:** The top-level tool dispatcher in `src/protocol.rs` applies `services.redactor.redact_json_value(&mut structured_content)` to the complete serialized tool response before writing it to stdout. This is the correct location for output-layer redaction.

However, `src/services/memory_store.rs` stores verbatim fact text in the Tantivy index and json backing files. There is no evidence in the reviewed code that redaction is applied **at index ingestion time** for memory facts. `Redaction.md` and `ToolContracts.md` both state that redaction MUST be applied at ingestion, not only at the output layer.

If a secret is written to a memory fact, it may be stored in plaintext in the Tantivy index shards, in the json backing files, and in any archive produced by log rotation.

**Risk:** An agent or user could deliberately or accidentally store a secret (API key, bearer token) as a memory fact. The secret would persist in the memory index on disk and remain readable via direct file access outside of Alfred's output-layer redaction.

**Smallest safe fix:** Apply the `Redactor` to all user-supplied string fields of `MemoryFactInput` (specifically `fact`, `reasoning`, `subject`) during `upsert_in_scope` before the data is written to the index and json store.

**Verification guidance:** A test that creates a memory fact containing a bearer token pattern confirms that the retrieved fact contains the redaction token, not the original secret. The stored json file must also not contain the secret.

---

## Passed items

| Checklist item                 | Evidence                                                                                                                                                            |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Parameterize All Outside Calls | Path inputs are normalized and boundary-checked before use; no shell invocations from user-controlled data paths.                                                   |
| Workspace boundary enforcement | `resolve_write_target_within_workspace_root` and `try_resolve_existing_path_within_workspace_root` are applied consistently across `fs`, `logs`, and `patch` paths. |
| Dependency pinning             | `Cargo.lock` is present; all versions in `Cargo.toml` are pinned to exact patch versions.                                                                           |
| Lean secret lifetime           | No secrets are extracted or manipulated at runtime; the redaction pipeline runs on output strings only.                                                             |
| No debug interface exposed     | No debugging or verbose-diagnostics mode is exposed via the public tool surface.                                                                                    |
