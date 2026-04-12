# Alfred Architecture

This document summarizes Alfred's architecture as modeled in Aurora and points to generated views and key decisions.

## Key artifacts

| Artifact                                                        | Purpose                                                                                                                      |
| --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| [AlfredOverview](./AlfredOverview.md)                           | Requirements and constraints that drive the architecture.                                                                    |
| [Aurora model home](./aurora/)                                  | Source-of-truth architecture model (cards + audit log).                                                                      |
| [Rendered model markdown](./MIS-001-Alfred_Local_MCP_Server.md) | Human-readable rendering of all cards in the model.                                                                          |
| [Model README](./README-MIS-001-Alfred_Local_MCP_Server.md)     | Entry point for the generated model bundle.                                                                                  |
| Views (SVG)                                                     | Diagrams rendered from the model into `docs/design/MIS-001/Views/`. **These are generated outputs** and are not hand-edited. |

## System context

Alfred is a local-only stdio server designed to run on the workspace host machine and be invoked by a local client (for example, an IDE extension). In VS Code Remote Development modes, the workspace host (extension host) may be remote (SSH/WSL/Dev Container), but Alfred still runs locally relative to that host.

The server:

- Communicates over stdin/stdout using deterministic JSON contracts.
- Operates within a configured workspace boundary (no escape hatches to arbitrary filesystem access).
- Prioritizes predictability and safety (dry-run support, atomic mutations where practical, normalized diagnostics, bounded background work).

Context, component, deployment, traceability, and security views are available as generated SVGs under `docs/design/MIS-001/Views/`.

## Core decomposition

At a high level, the application is a stdio transport, a tool router, and a consolidated set of tool handlers with shared cross-cutting services.

- Transport and protocol edge
    - Stdio transport and message framing.
    - Protocol contract validation and deterministic error taxonomy.
- Routing and orchestration
    - Tool router dispatches requests to handlers.
    - Capability discovery advertises supported tools and versions.
- Indexing and Memory
    - The tool indexes all files in the workspace except those in one of the ignore lists (permanent, default, user, workspace)
    - The tool provides memory functions using the same indexing technology, supporting search, including full text, for memories.
    - All indices are periodically persisted to disk to reduce startup indexing overhead.
- Consolidated tool handlers
    - `search`: workspace text search over the index.
    - `fs`: non-bulk text-file and directory operations.
    - `patch`: one-or-more patch application with conflict reporting and a duplicate-content safeguard (hard refusal).
    - `logs`: deterministic search/tail over structured logs.
    - `plan`: project-plan read/write workflows.
    - `memory`: local/offline memory CRUD and retrieval.

The component view is available as a generated SVG under `docs/design/MIS-001/Views/`.

## Data and state

Alfred is intentionally local-only and uses small, explicit state stores:

- Workspace index: derived, rebuildable index of workspace structure/content used for fast search and navigation.
- Plan file: records plan state and completion status to support incremental progress reporting.
- Bulk operation state: tracks background bulk operation lifecycle and progress (`queued`, `running`, `succeeded`, `failed`, `canceled`, `partial`) in memory.
- Memory store: persists agent memory facts and an associated local search index for offline recall.

These stores are modeled as local data stores backed by the host filesystem with no external services.

By default, workspace-scoped Alfred artifacts are rooted at `<workspaceRoot>/.alfred/`.

## Cross-cutting concerns

The architecture encodes several non-negotiables:

- Deterministic outputs: stable ordering and stable formatting to make results diffable and testable.
- Diagnostics normalization: consistent shape for errors/warnings and predictable error taxonomy.
- Guardrails: workspace boundary enforcement, explicit out-of-scope behaviors, and local-only execution.
- Versioning discipline: SemVer with schema/version lockstep and conformance testing.
- Cross-platform semantics: filesystem/path normalization, safe mutation semantics (no partial writes; no temp-file replace/rename), cancellation behavior, and encoding-safe path reporting are modeled explicitly via constraints `CNS-015` through `CNS-021`.

For concrete contracts (including memory CRUD/search), see:

- [Protocol](./Protocol.md)
- [Error taxonomy](./ErrorTaxonomy.md)
- [Tool contracts](./ToolContracts.md)
- [Default values reference](./Defaults.md)
- [Design authority matrix](./DesignAuthority.md)
- [Quality policy](./QualityPolicy.md)

The traceability view (drivers → requirements → capabilities → features → components) is available as a generated SVG under `docs/design/MIS-001/Views/`.

## Security and threat modeling

The Aurora model includes an explicit threat model rooted at `THM-001`. It assumes (at minimum) one scenario where the agent/client is malicious or compromised and attempts forbidden actions such as workspace boundary escape or unauthorized destructive mutations.

Cross-platform caveats that can impact safety and determinism (symlinks/junctions, file locking and mutation differences, shell portability, cancellation semantics, and encoding/path handling) are captured as model constraints/controls and reflected in the security view.

The security view is available as a generated SVG under `docs/design/MIS-001/Views/`.

## Deployment

Alfred is deployed as a single local stdio process on the workspace host machine and communicates over stdio. This remains true even when the user is in VS Code Remote Development modes (where the host is remote). It is intended to run on Linux, macOS, and Windows; mobile platforms (iOS/Android) are out of scope (but may still work for most operations).

The deployment view is available as a generated SVG under `docs/design/MIS-001/Views/`.

## Design Goals

- **Minimize per-call token cost**: consolidate the most context-expensive agentic operations into a small, composable surface with compact, deterministic responses.
- **Deterministic by design**: stable ordering, stable error taxonomy, explicit pagination, and consistent output across runs and platforms. Results must be diffable and testable.
- **Safe and conservative by default**: dry-run defaults on all mutating operations, atomic writes where practical, workspace boundary enforcement, and no implicit destructive behavior.
- **Search performance through local indexing**: the single most-used operation (workspace text search) is backed by a persistent local index delivering sub-500 ms p95 latency without spawning external processes.
- **Offline and self-contained**: no external service dependencies; all state (index, memory, logs) is local, explicit, and fully rebuildable from the workspace.
- **Cross-platform reliability**: full support for Linux, macOS, and Windows semantics, including path encoding edge cases, symlink/junction safety, and filesystem mutation strategies.
    - Add a local-only memory capability for security, speed, and independence from connectivity issues and corporate whims.
- The entire server is a single, self-contained executable with no outside dependencies.
- Alfred is configurable, allowing the user to customize as many aspects of operation as feasible.
- The server is designed to be secure by default:
    - Write operations for commands are limited to the workspace. This is a much harder limit than the default for most IDEs, as it will not even ask permission.
    - Read operations for commands are limited to the workspace and alfred logs. The log exception exists to support develoment, troubleshooting, and support.
    - All operations that modify the workspace have a "dry-run" capability, on by default.
- Several elements have been implemented to prevent blocking. Indexing is on a separate thread, allowing it to run independantly. Indices are persisted periodically to disk and loaded at startup, making the initial indexavailability even faster.
- Alfred is designed to prevent several comnmon failure modes of agents:
    - Patches that would create duplicate content are blocked.
    - Complicate write and replace processes are replaced with deterministic, in place, reversible patching.
