# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Initial Rust skeleton for Alfred with a canonical application shell, shared protocol types, and the public capability registry.
- Smoke coverage that verifies the advertised tool surface stays aligned with the canonical design constants.
- Execution-mode serialization now uses the Rust enum variant names directly, removing the lowercase serde rename.
- Added a typed router dispatch boundary that resolves known tool names to canonical routes and rejects unknown names.
- Split router route metadata into a dedicated submodule so `src/router.rs` stays focused on dispatch and the terminal error enum.
- Added a workspace boundary module that normalizes workspace paths and rejects boundary escapes.
- Updated workspace boundary resolution to accept absolute workspace paths when they remain inside the workspace root.
- Simplified workspace boundary resolution to use `Path`-based containment checks instead of string splitting.
- Updated workspace path normalization so host-OS-valid inputs are evaluated by the workspace-boundary check instead of being rejected for their surface form.
- Added a string-prefix fast path for absolute workspace-contained targets to reduce work on the common case.
- Restricted filename validation to platform-specific Windows rules instead of a global character blacklist.
- Allowed in-bounds `..` path segments to collapse instead of being rejected outright.
- Simplified workspace resolution to a single candidate path flow that normalizes once and then checks containment.
- Centralized the public tool surface in `src/capabilities.rs` so capability advertisement and routing read from one canonical table.
- Moved workspace-boundary tests to `src/workspace_boundary/tests/workspace_boundary_tests.rs` to match the module test layout convention.
- Added Cargo package metadata for the MSRV, package description, license, and readme so the build environment is declared explicitly.
- Added a canonical tool execution boundary that implements `capabilities`, `workspace_dir`, and `status` and returns deterministic envelopes for implemented, unimplemented, and unknown tools.
