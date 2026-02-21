# Alfred MCP Server

Alfred provides efficient, safe, and reliable MCP capabilities for common agent workflows.

## Functional Requirements

- Alfred MUST maintain an index of all files in the workspace and their contents
    - Alfred MUST provide file listings (`ls`)
    - Alfred MUST provide text search with regex support (`grep`, `rg`)
    - Alfred MUST provide keyword and symbol lookup
    - Alfred MUST provide file range extraction
    - Alfred MUST provide diff capabilities
- Alfred MUST provide location and context awareness
    - Alfred MUST return the current working folder (`pwd`)
    - Alfred MUST return the workspace root folder
- Alfred MUST provide safe file operations
    - Alfred MUST support atomic CRUD operations where practical
    - Alfred MUST support patching with conflict reporting
    - Alfred MUST support bulk move, rename, delete, and copy operations with dry-run support
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
        - Alfred MUST return recent tool calls and results
- Alfred MUST provide tooling output as JSON or NDJSON
- Alfred MUST provide log handling
    - Alfred MUST tail logs
    - Alfred MUST filter logs
- Alfred MUST support CRUD operations for environment variables
- Alfred MUST provide a capability discovery endpoint
    - Alfred MUST return available tools and capabilities
    - Alfred MUST return tool and schema versions
    - Alfred MUST return capability limits and execution modes
- Alfred MUST support single-call action chaining (for example: search -> patch -> validate)
- Alfred MUST be able to perform all file operations on any size file.

## Non-Functional Requirements

- Alfred MUST produce concise and complete JSON or NDJSON output suitable for agents and conservative of tokens
- Alfred MUST provide a normalized diagnostics contract for build, test, lint, format, and related tooling operations
    - Diagnostics MUST use a consistent schema regardless of source tool
    - Diagnostics SHOULD support delta reporting between runs (for example: new, unchanged, resolved)
- Alfred MUST use deterministic and clear error taxonomy definitions
- Alfred MUST provide dry-run behavior for destructive or irreversible actions
- Alfred MUST include a conformance suite and the suite MUST pass for releases
    - The suite MUST validate response schemas (JSON and NDJSON)
    - The suite MUST validate deterministic error taxonomy behavior
    - The suite MUST validate dry-run guarantees for destructive operations
    - The suite MUST validate workspace boundary enforcement
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
- Indexed search and symbol lookup p95 latency MUST be <= 500 ms for typical repositories
- File patch/apply p95 latency MUST be <= 1000 ms for files up to 1 MB
- Background job status update freshness MUST be <= 2 s
- Cancellation acknowledgment p95 latency MUST be <= 3 s
- These SLOs are initial objectives and SHOULD be tightened as usage data is collected

## Safety

- Alfred MUST NOT read or write outside the current workspace
- Alfred MUST provide a permission model compliant with IDE requirements
- Alfred MUST provide configurable policy guardrails with reasonable defaults

## Out of Scope

- Alfred MUST NOT implement Git or GitHub operations handled by dedicated tools
