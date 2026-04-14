# As Built

## Current Implementation Snapshot

- Added a minimal Rust crate skeleton with `src/lib.rs`, `src/main.rs`, and focused modules for design constants, protocol types, capabilities, and application state.
- Centralized the public tool surface in `src/capabilities.rs` so routing and capability advertisement read from one canonical table instead of duplicating tool names in multiple places.
- Added an integration-style smoke test under `src/tests/` to verify the public capability list matches the canonical design constants.
- Updated the execution-mode contract so serde uses the enum variant names directly instead of a lowercase rename.
- Added a typed router dispatch boundary that classifies known tool names into canonical routes and rejects unknown tool names.
- Split router route metadata into `src/router/route.rs` so the router file keeps the dispatch boundary and terminal error enum in one place.
- Added a workspace boundary module that normalizes workspace paths, resolves write targets, and rejects boundary escapes, with tests living in `src/workspace_boundary/tests/workspace_boundary_tests.rs`.
- Simplified workspace boundary resolution to use `Path` components and direct workspace-root containment checks instead of string splitting.
- Updated workspace boundary resolution to accept absolute workspace paths when they stay within the workspace root.
- Updated workspace path normalization to allow absolute inputs to reach the workspace-boundary containment check.
- Added a string-prefix fast path for absolute workspace-contained targets so the hot path can skip full normalization when the root match is obvious.
- Made component validation platform-specific so Windows filename rules are enforced only on Windows instead of as a global blacklist.
- Allowed `..` segments to collapse when they remain inside the workspace path, while still rejecting leading escapes beyond the workspace boundary.
- Simplified workspace resolution to a single candidate path flow that normalizes once and then checks containment.
- Declared Cargo package metadata for the MSRV, package description, license, and readme so the Rust environment is explicit before broader implementation continues.
- Added `src/tool_execution.rs` as the canonical execution boundary for the currently implemented `capabilities`, `workspace_dir`, and `status` tool handlers.
- Updated `src/app.rs` so `Alfred::invoke_tool` returns canonical tool envelopes for implemented, unimplemented, and unknown tools.

## Notes

- The stdio transport is still scaffold-only.
- No alternate capability registry exists; the current registry in `src/capabilities.rs` is the single source of truth for the advertised tool list.
