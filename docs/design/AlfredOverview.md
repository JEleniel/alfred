# Alfred MCP Server

Alfred provides efficient, safe, and reliable MCP capabilities for common agent workflows.

## Functional Requirements

- Alfred MUST maintain an index of all files in the workspace and their contents.
- Alfred MUST provide a consolidated tool surface that minimizes command-count overhead.
    - Alfred MUST expose `search` for workspace text search (literal and regex).
    - Alfred MUST expose `fs_operations` for file and directory operations (non-bulk).
    - Alfred MUST expose `bulk_fs_operations` for bulk move/copy/delete operations.
    - Alfred MUST expose `patch` for one-or-more patch applications.
    - Alfred MUST expose `log_operations` for deterministic log search/tail operations.
    - Alfred MUST expose `plan_operations` for project-plan CRUD/update operations.
    - Alfred MUST expose `memory` for memory CRUD/search operations.
- Alfred MUST provide location and context awareness.
    - Alfred MUST return the workspace root folder (`workspace_dir`).
- Alfred MUST provide safe file operations.
    - Alfred MUST support atomic CRUD operations where practical.
    - Alfred MUST support patching with conflict reporting and duplicate-content-risk warnings.
    - Alfred MUST support bulk operations with dry-run support.
- Alfred MUST create and maintain a project plan in a common format.
    - Alfred MUST capture tool and diagnostics errors in the plan to track fixes.
    - Alfred MUST track progress in the plan.
- Alfred MUST support constrained background operations.
    - Only `bulk_fs_operations` MAY run in the background.
    - Background status MUST be retrievable using `bulk_fs_operations` itself.
    - Alfred MUST NOT expose standalone job-control commands.
- Alfred MUST provide tooling output as JSON or NDJSON where contractually applicable.
- Alfred MUST provide local, indexed, searchable memory.
    - Alfred MUST support CRUD operations for individual memory facts.
    - Alfred MUST support full-text search over stored memory.
    - The memory system MUST be offline-only and MUST NOT depend on any external service.
- Alfred MUST provide a capability discovery endpoint.
    - Alfred MUST return available tools and capabilities.
    - Alfred MUST return tool and schema versions.
    - Alfred MUST return capability limits and execution modes.
- Alfred MUST operate on text files only.
    - Byte-oriented read/write operations are out of scope.

## Non-Functional Requirements

- Alfred MUST produce concise and complete JSON or NDJSON output suitable for agents and conservative of tokens.
- Alfred MUST provide a normalized diagnostics contract for tooling operations.
    - Diagnostics MUST use a consistent schema regardless of source tool.
    - Diagnostics SHOULD support delta reporting between runs (for example new, unchanged, resolved).
- Alfred MUST use deterministic and clear error taxonomy definitions.
- Alfred MUST provide dry-run behavior for destructive or irreversible actions.
- Alfred MUST be self-contained and MUST NOT depend on any outside service.

## Versioning and Compatibility

- Tool versions MUST use Semantic Versioning (SemVer).
- Schema versions MUST remain in lockstep with tool versions.
- Backward compatibility MUST follow normal SemVer rules.
- No formal deprecation window is defined at this time.

## Execution Semantics

- Every tool MUST declare its execution mode as synchronous or background-capable.
- Read-only operations MUST be side-effect free.
- Mutating operations SHOULD be atomic per target where practical.
- Cancellation MUST be best-effort and MUST return final operation state when available.
- Timeouts MUST return a deterministic timeout error and operation state.

## Service Level Objectives

- Capability discovery p95 latency MUST be <= 100 ms.
- Indexed search p95 latency MUST be <= 500 ms for typical repositories.
- File patch/apply p95 latency MUST be <= 1000 ms for files up to 1 MB.
- Bulk background status freshness p95 latency MUST be <= 2 s.
- These SLOs are initial objectives and SHOULD be tightened as usage data is collected.

## Safety

- Alfred MUST NOT read or write outside the current workspace.
- Alfred MUST provide a permission model compliant with IDE requirements.
- Alfred MUST provide configurable policy guardrails with reasonable defaults.

## Libraries

The following libraries have been chosen, for various reasons:

- `rmcp` for the MCP interface.
- `dir_watcher` for file system watching.
- `tantivy` for indexing and search.
- `imara-diff` for diffing.
- `mpatch` for patching.
- `anyhow`, `thiserror` for error handling.
- `base64`, `hex`, `num-traits`, `regex`, `unicode-normalization`, `uuid` for utilities.
- `chrono` for time and date handling.
- `clap` for CLI interfaces.
- `config` for configuration file handling.
- `dirs` for standard config/data/cache directories.
- `fern`, `log` for logging.
- `r2d2`, `r2d2_sqlite`, `rusqlite` for SQLite (use `rusqlite` with the `bundled` feature).
- `serde` (and sublibraries), `serde_json` for serialization.
- `tokio` (and sublibraries) for async runtime.
- `url`, `urlencoding` for URL handling.
- `sha2`, and `hmac` for hashing.

## Out of Scope

- Alfred MUST NOT implement Git or GitHub operations handled by dedicated tools.
- Alfred MUST NOT expose byte-oriented file operation tools.
- Alfred MUST NOT expose standalone environment-variable CRUD tools.

## Design references

- Protocol expectations: `docs/design/Protocol.md`
- Deterministic error taxonomy: `docs/design/ErrorTaxonomy.md`
- Tool contracts (including memory tooling): `docs/design/ToolContracts.md`
