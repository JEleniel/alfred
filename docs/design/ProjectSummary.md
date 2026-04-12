# Project Summary

This repository contains Alfred: a local-only Model Context Protocol (MCP) stdio server that provides efficient, safe, and deterministic tools for common agent workflows inside a workspace.

Alfred is designed to run on the workspace host machine (including VS Code Remote Development modes where the host may be remote) and communicate exclusively over stdin/stdout using newline-delimited UTF-8 JSON frames.

## What Alfred provides

Alfred consolidates a small public tool surface to minimize tool-count overhead while still covering typical “agent in a repo” workflows:

- `capabilities`: discover available tools, versions, schema versions, execution modes, and limits.
- `workspace_dir`: return the resolved workspace root.
- `status`: report index readiness, memory usage, and configured runtime paths.
- `search`: deterministic text search (`general`, `full_text`, and `regex` modes), backed by a workspace index.
- `fs`: file/directory operations and inspection (text-only range reads, diff, CRUD with dry-run where applicable), including deterministic bulk move/copy/delete with optional background execution and built-in status polling.
- `patch`: apply one or more text patches with conflict reporting and a duplicate-content safeguard (hard refusal).
- `logs`: deterministic search/tail over logs.
- `plan`: read and update status on the workspace project plan.
- `memory`: offline-only memory CRUD, search, and full-text retrieval.

Execution semantics are deliberately constrained:

- Read-only operations are side-effect free.
- Mutating operations are safe-by-default (dry-run where applicable; atomic per-target where practical).
- Background execution is limited and MUST be pollable via the tool that started it (for example `fs` bulk operations).
- Streaming behavior MAY be supported (for example `logs.follow`). When supported, Alfred MUST allow at most one active stream at a time.

## Determinism, safety, and contract discipline

Alfred’s architecture emphasizes predictable, testable behavior:

- Workspace boundary enforcement: operations must not read or write outside the configured workspace boundary.
- Deterministic results: stable ordering, stable formatting, explicit pagination/cursors, and cross-platform normalization.
- Consistent tool envelopes: tools return a common success/error shape; only background-capable operations may return `status: "pending"`.
- Deterministic error taxonomy: failures are mapped into a small, stable set of deterministic error kinds (see [`docs/design/ErrorTaxonomy.md`](./ErrorTaxonomy.md)).
- Deterministic redaction: non-public information (NPI) is filtered from tool outputs, logs, and indexes using a stable replacement token (default `<-REDACTED->`).
- Path handling: unless a contract says otherwise, paths are workspace-relative and use `/` separators; unrepresentable paths are encoded deterministically to keep protocol output valid UTF-8 JSON.

## Configuration model

Alfred reads configuration at two levels with deterministic precedence:

- User configuration: applies to all workspaces.
- Workspace configuration: applies within a single workspace and overrides user configuration.

Default locations:

- User: OS config directory `alfred/config.json`.
- Workspace: `<workspaceRoot>/.alfred/config.json`.

Configuration is JSON validated against `schemas/alfred.config.schema.json`.

Notable configuration areas include:

- Tool enablement via `tools.disabled`.
- Index enablement and persistence settings.
- Workspace storage root (`workspace.storage.root`, default `.alfred/`).
- Memory storage (explicit user/workspace scope; user store enabled by default; optional workspace store).
- Storage location controls for relocating workspace artifacts into OS user directories or relocating user artifacts into the workspace.
- Redaction behavior (replacement token, length preservation, rule sets).
- Plan location (`plan.path`), defaulting to `docs/design/ProjectPlan.md` when present.

## Local state and artifacts

Alfred uses local, explicit state stores and does not depend on external services.

By default, workspace-scoped artifacts are rooted at `<workspaceRoot>/.alfred/` and may include:

- Workspace index persistence.
- Optional workspace memory store.

Alfred logs are structured NDJSON records intended for deterministic search/tail operations.

Depending on `storage.*` and `logging.*` configuration, some artifacts (including runtime logs and optionally workspace-scoped data) MAY be stored under OS user directories rather than the workspace.

## Non-goals and out of scope

This repository explicitly documents several non-goals:

- No Git or GitHub operations (use dedicated tools).
- No byte-oriented file read/write tooling; Alfred operates on text content only.
- No standalone job-control tool surface; background work is limited and polled through the tool that started it.
- No standalone environment-variable CRUD tool surface.

## Where to look next

- Requirements and constraints: [AlfredOverview](./AlfredOverview.md)
- Architecture and decomposition: [AlfredArchitecture](./AlfredArchitecture.md)
- Protocol envelopes, framing, NDJSON usage: [Protocol](./Protocol.md)
- Tool contracts (inputs/outputs/limits): [ToolContracts](./ToolContracts.md)
- Configuration model: [Configuration](./Configuration.md)
- Default values (canonical reference): [Defaults](./Defaults.md)
- Deterministic error taxonomy: [ErrorTaxonomy](./ErrorTaxonomy.md)
- Deterministic redaction: [Redaction](./Redaction.md)
- Index include/exclude rules: [DefaultIncludeExcludeList](./DefaultIncludeExcludeList.md)
- Design authority (One Source of Truth): [DesignAuthority](./DesignAuthority.md)
- Structural quality policy: [QualityPolicy](./QualityPolicy.md)
- Rendered Aurora model bundle: [MIS-001](./README-MIS-001-Alfred_Local_MCP_Server.md)

> Agent guidance prompts and pre-written instructions for common workflows will be added in a future version.
