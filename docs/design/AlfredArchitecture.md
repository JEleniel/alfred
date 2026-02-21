# Alfred Architecture

This document summarizes Alfred’s architecture as modeled in Aurora, and points to the generated views and key decisions.

## Key artifacts

| Artifact                                                        | Purpose                                                                                               |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| [AlfredOverview](./AlfredOverview.md)                           | Requirements and constraints that drive the architecture.                                             |
| [Aurora model home](./aurora/)                                  | Source-of-truth architecture model (cards + audit log).                                               |
| [Rendered model markdown](./MIS-001-Alfred_Local_MCP_Server.md) | Human-readable rendering of all cards in the model.                                                   |
| [Model README](./README-MIS-001-Alfred_Local_MCP_Server.md)     | Entry point for the generated model bundle.                                                           |
| [Views (SVG)](./MIS-001/Views/)                                 | Diagrams rendered from the model (context, components, deployment, traceability, etc.).               |
| [ADR-0001: Rust](./adr/0001-rust.md)                            | Decision record: Rust selected for performance, safety, flexibility, and cross-platform distribution. |

## System context

Alfred is a **local-only stdio server** designed to be run on the *workspace host* machine and invoked by a local client (for example, an IDE extension). In VS Code Remote Development modes, the workspace host (extension host) may be remote (SSH/WSL/Dev Container), but Alfred still runs *locally relative to that host*.

The server:

- Communicates over **stdin/stdout** using deterministic JSON/NDJSON contracts.
- Operates **within a configured workspace boundary** (no “escape hatches” to arbitrary filesystem access).
- Prioritizes **predictability and safety** (dry-run support; atomic mutations where practical; normalized diagnostics; bounded background work).

For the model’s context view, see [Context.svg](./MIS-001/Views/Context.svg).

## Core decomposition

At a high level, the application is a stdio transport + a tool router + a set of tool-handlers, with shared cross-cutting services.

- Transport / protocol edge
    - Stdio transport and message framing.
    - Protocol contract validation and deterministic error taxonomy.
- Routing and orchestration
    - Tool router dispatches requests to tool-handlers.
    - Capability discovery advertises supported tools and versions.
- Tool-handlers
    - Workspace indexing and search.
    - File operations (read/write/patch) constrained to workspace.
    - Task execution (local process runner) and background job controls.
    - Plan management (produce and update a project plan; support partial completion).

For the model’s component view, see [Component.svg](./MIS-001/Views/Component.svg).

## Data and state

Alfred is intentionally local-first and uses small, explicit state stores:

- **Workspace Index**: derived, rebuildable index of workspace structure/content used to support fast search and navigation.
- **Plan Store**: records plan state and completion status to support incremental progress reporting.
- **Job Store**: tracks background jobs (start/stop/status) and their logs/results.

These stores are modeled as local data stores backed by the host filesystem (no external services).

## Cross-cutting concerns

The architecture encodes several non-negotiables:

- **Deterministic outputs**: stable ordering and stable formatting to make results diffable and testable.
- **Diagnostics normalization**: consistent shape for errors/warnings and predictable error taxonomy.
- **Guardrails**: workspace boundary enforcement, explicit out-of-scope behaviors, and local-only execution.
- **Versioning discipline**: SemVer with schema/version lockstep and conformance testing.
- **Cross-platform semantics**: filesystem/path normalization, atomic write semantics, task execution defaults, cancellation behavior, and encoding-safe path reporting are modeled explicitly via constraints `CNS-015`..`CNS-021`.

For the traceability view (drivers → requirements → capabilities → features → components), see [Traceability.svg](./MIS-001/Views/Traceability.svg).

## Security and threat modeling

The Aurora model includes an explicit threat model rooted at `THM-001`. It assumes (at minimum) one scenario where the **agent/client is malicious or compromised** and attempts forbidden actions such as workspace boundary escape or dangerous command execution.

Cross-platform caveats that can impact safety and determinism (symlinks/junctions, atomic replace differences, shell portability, cancellation semantics, and encoding/path handling) are captured as model constraints/controls and reflected in the security view.

For the rendered security view, see [Security.svg](./MIS-001/Views/Security.svg).

## Deployment

Alfred is deployed as a single local stdio process on the workspace host machine and communicates over stdio. This remains true even when the user is in VS Code Remote Development modes (where the host is remote). It is intended to run on **Linux, macOS, and Windows**; **mobile platforms (iOS/Android) are out of scope**.

For the model’s deployment view, see [Deployment.svg](./MIS-001/Views/Deployment.svg).

## Regenerating model outputs (optional)

The diagrams and markdown in `docs/design/` are generated from the Aurora cards in `docs/design/aurora/`.

```text
aurora_cli validate -i docs/design/aurora
aurora_cli render-all -i docs/design/aurora
aurora_cli compact -i docs/design/aurora
```
