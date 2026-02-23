# Alfred MCP Server

Alfred provides efficient, safe, and reliable MCP capabilities for common agent workflows.

## Functional Requirements

- Alfred MUST maintain an index of all files in the workspace and their contents
    - Alfred MUST provide file listings (`ls`)
    - Alfred MUST provide text search with regex support (`grep`, `search`)
    - Alfred MUST provide file range extraction
    - Alfred MUST provide diff capabilities
- Alfred MUST provide location and context awareness
    - Alfred MUST return the workspace root folder (`workspace_dir`)
    - Alfred MUST return the configured workspace root folder (`workspace_root`)
- Alfred MUST provide safe file operations
    - Alfred MUST support atomic CRUD operations where practical
    - Alfred MUST support patching with conflict reporting
    - Alfred MUST support bulk move, rename, delete, and copy operations with dry-run support (`path_move`, `path_copy`, `path_delete`)
- Alfred MUST create and maintain a project plan in a common format
    - Alfred MUST capture tool and diagnostics errors in the plan to track fixes
    - Alfred MUST track progress in the plan
- Alfred MUST provide task execution capabilities
    - Alfred MUST run named tasks (for example: `build`, `test`)
    - Alfred MUST run common `cargo` commands
    - Alfred MUST run common `pnpm` commands
    - Alfred MUST run common linting and formatting tools
- Alfred MUST support background operations
    - Alfred MUST support asynchronous operations with streaming output
    - Alfred MUST support cancellation and timeout controls
    - Alfred MUST provide job and session introspection
        - Alfred MUST list active background jobs with status and metadata
        - Alfred MUST return output chunks/streams and current state for a job
        - Alfred MUST return recent tool calls and results (`session_recent`)
- Alfred MUST provide tooling output as JSON or NDJSON
- Alfred MUST provide log handling
    - Alfred MUST tail logs
    - Alfred MUST filter logs
- Alfred MUST support CRUD operations for environment variables
- Alfred MUST provide local, indexed, searchable memory
    - Alfred MUST support CRUD operations for individual memory "facts"
    - Alfred MUST support full-text search over stored memory
    - The memory system MUST be offline-only and MUST NOT depend on any external service
- Alfred MUST provide a capability discovery endpoint
    - Alfred MUST return available tools and capabilities
    - Alfred MUST return tool and schema versions
    - Alfred MUST return capability limits and execution modes
- Alfred MUST support single-call action chaining (for example: search -> patch -> validate)
- Alfred MUST be able to perform all file operations on any size file.
    - For binary or very large file content, Alfred MUST support bounded byte-chunk reads and writes (`file_read_bytes`, `file_create_bytes`, `file_append_bytes`).

## Non-Functional Requirements

- Alfred MUST produce concise and complete JSON or NDJSON output suitable for agents and conservative of tokens
- Alfred MUST provide a normalized diagnostics contract for build, test, lint, format, and related tooling operations
    - Diagnostics MUST use a consistent schema regardless of source tool
    - Diagnostics SHOULD support delta reporting between runs (for example: new, unchanged, resolved)
- Alfred MUST use deterministic and clear error taxonomy definitions
- Alfred MUST provide dry-run behavior for destructive or irreversible actions
- Alfred MUST be self contained; it MUST NOT depend on any outside service.

## Versioning and Compatibility

- Tool versions MUST use Semantic Versioning (SemVer)
- Schema versions MUST remain in lockstep with tool versions
- Backward compatibility MUST follow normal SemVer rules
- No formal deprecation window is defined at this time

## Execution Semantics

- Every tool MUST declare its execution mode as synchronous or asynchronous
- Read-only operations MUST be side-effect free
- Mutating operations SHOULD be atomic per target where practical
- Chained actions MUST execute in declared order
- Chained actions MUST stop on failure by default
- Chained action responses MUST include per-step status and error details
- Cancellation MUST be best-effort and MUST return final operation state when available
- Timeouts MUST return a deterministic timeout error and operation state

## Service Level Objectives

- Capability discovery p95 latency MUST be <= 100 ms
- Indexed search p95 latency MUST be <= 500 ms for typical repositories
- File patch/apply p95 latency MUST be <= 1000 ms for files up to 1 MB
- Background job status update freshness MUST be <= 2 s
- Cancellation acknowledgment p95 latency MUST be <= 3 s
- These SLOs are initial objectives and SHOULD be tightened as usage data is collected

## Safety

- Alfred MUST NOT read or write outside the current workspace
- Alfred MUST provide a permission model compliant with IDE requirements
- Alfred MUST provide configurable policy guardrails with reasonable defaults

## Libraries

The following libraries have been chosen, for various reasons:

- `rmcp` for the MCP interface
- `dir_watcher` for file system watching
- `tantivy` for indexing and search
- `imara-diff` for diffing
- `mpatch` for patching
- `escargot` for cargo operations
- `anyhow`, `thiserror` for error handling
- `base64`, `hex`, `num-traits`, `regex`, `unicode-normalization`, `uuid` for utilities
- `chrono` for time and date handling
- `clap` for CLI interfaces
- `config` for configuration file handling
- `dirs` for standard config/data/cache directories
- `fern`, `log` for logging
- `r2d2`, `r2d2_sqlite`, `rusqlite` for SQLite (use `rusqlite` with the `bundled` feature)
- `serde` (and sublibraries), `serde_json` for serialization
- `tokio` (and sublibraries) for async runtime
    - `tokio::process` can be used for tools like `npm` and `pnpm`
- `url`, `urlencoding` for URL handling
- `sha2`, and `hmac` for hashing

## Out of Scope

- Alfred MUST NOT implement Git or GitHub operations handled by dedicated tools

## Design references

- Protocol expectations: `docs/design/Protocol.md`
- Deterministic error taxonomy: `docs/design/ErrorTaxonomy.md`
- Tool contracts (including Memory tools): `docs/design/ToolContracts.md`
