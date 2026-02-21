---
name: coding
description: Guidelines for writing code of any kind.
---

# Coding

## General Guidelines

- The language-specific rules in `../../instructions/*.instructions.md` take precedence over these instructions.
- Follow best practices for the language being edited. Language-specific configs (for example `rustfmt.toml`, `.markdownlint-cli2.jsonc`, `.prettierrc.json`) are authoritative.
- Keep code modular and cohesive (single responsibility). Prefer small functions (~20 lines) and small modules (~200 lines) when practical.
- Prefer small, cohesive changes. Fix root causes, not symptoms.
- Use the shortest acceptable path for local files.
- Prefer mature, well supported dependencies with GPL, MIT, or Apache-2.0 licenses.
    - Well-maintained heuristic (use judgment; not a checklist):
        - Active and responsive maintenance (issues/PRs triaged; CI is healthy).
        - Clear compatibility story (MSRV/edition/features) that matches the workspace.
        - Security posture is solid (no known unfixed advisories; timely fixes when issues occur).
        - Adoption is meaningful (downstream usage and/or strong community reputation).
        - Documentation quality is good (README, examples, and changelog/release notes).
        - Exception: clearly stable/feature-complete crates MAY be acceptable with explicit justification.
- Dependency management:
    - Prefer the latest stable crate versions, unless constrained by compatibility, MSRV, or security response.
    - Add dependencies at the narrowest practical scope (package-level, not workspace-wide) unless multiple crates truly share them.
    - Avoid new dependencies when the standard library or existing dependencies already solve the problem.

- Approved libraries
    - The following libraries are approved for use. Sublibraries include crates that share the parent prefix or are designed as companions.
    - `anyhow`, `thiserror` for error handling
    - `axum` (and sublibraries), `tower` (and sublibraries), `hyper` (and sublibraries) for web servers
    - `base64`, `hex`, `num-traits`, `regex`, `unicode-normalization`, `uuid` for utilities
    - `chrono` for time and date handling
    - `clap` for CLI interfaces
    - `config` for configuration file handling
    - `ctrlc` for signal handling
    - `dirs` (preferred) or `directories` for standard config/data/cache directories
    - `fern` (preferred) or `tracing` (and sublibraries) for logging
    - `log` for logging API
    - `ollama-rs` for Ollama access
    - `openssl` or `rustls` (and sublibraries) for TLS
    - `r2d2`, `r2d2_sqlite`, `rusqlite` for SQLite (use `rusqlite` with the `bundled` feature)
    - `reqwest` for HTTP client calls
    - `serde` (and sublibraries), `serde_json` for serialization
    - `tokio` (and sublibraries) for async runtime
    - `url`, `urlencoding` for URL handling
    - `sha2`, and `hmac` for hashing

## Invariants

- For newly written or substantially rewritten code:
    - You MUST NOT allow any new source file to exceed 500 lines or 50MiB in size, whichever is smaller.
    - You MUST NOT write any new function that exceeds 50 lines in length.

- For pre-existing code that violates these size limits:
    - You SHOULD recommend refactoring when you encounter it.
    - You MUST NOT perform large refactors solely to satisfy the limits unless the task requires it.

- Unimplemented paths MUST fail fast and clearly communicate intent (`todo!`, `unimplemented!`, etc.).

## Errors and Logging

- Prefer typed errors within libraries/modules.
- Add context at application boundaries.
- Avoid unchecked failures (`unwrap`, `expect`, panics) unless justified by an explicit invariant.
- You MUST NOT disable checks/tests (for example `// @ts-nocheck`, `#[allow(...)]`). Fix the underlying issue instead.
- Include helpful TRACE and DEBUG logging where appropriate for troubleshooting.
- Log at boundaries with appropriate severity.
- Never log secrets at any level.

## Tests

For all added code:

- Add tests that prove behavior (positive and negative paths).
- Add tests that prove deterministic behavior, when appropriate (for example, fixed seeds for randomized tests).
- Add tests that prove secure behavior, e.g., malformed input, out of range values, etc.
- If tests need data files, or need to write files, create and use a `testdata` directory at the root of the source tree (e.g. `src/testdata/` for Rust code) and use that for test fixtures. Do not write files outside of the test environment.

## Deliverables

- Source code is modular, clean, readable, idiomatic, and aligned with project conventions.
- Appropriate unit and integration tests are added and passing.
    - Coverage target (aspirational): aim for 90%+ coverage on functional code when practical.
- Notes added for the documentation writer explaining changes, new features, and other relevant information for the project documentation.
- Linting, formatting, and static analysis checks are passing.
