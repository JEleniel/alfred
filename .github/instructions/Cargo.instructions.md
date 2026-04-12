---
description: 'Agent directives for Cargo.toml metadata, dependencies, and package configuration.'
applyTo: '**/Cargo.toml'
---

# Cargo.toml Instructions

## Tooling

- ALWAYS use `cargo-mcp` for Cargo operations when available.
- If unavailable, use the standard `cargo` CLI.

## Core Settings

- ALWAYS set `edition` to the current stable value supported by the repo or toolchain and declare `rust-version` (MSRV).
- ALWAYS include a clear `description`.
- Include `license` and `readme` if those files exist.
- NEVER use the deprecated `authors` field.
- ALWAYS use SPDX license identifiers. By default, use MIT/Apache dual licensing.
- ALWAYS use explicit semver requirements; never `"*"`.
- NEVER pin the patch or minor version unless a specific fix or compatibility need requires it.
- ALWAYS minimize the dependency count.
- ALWAYS separate `dev-` and `build-` dependencies.
- NEVER use git dependencies.
- NEVER use `patch`.
- In a multi-project repository, ALWAYS centralize versions in `[workspace.dependencies]` when used by more than one component.
- Avoid hidden behavior in defaults or build scripts.

## Approved Crates and Default Purposes

The following crates are approved by default for the listed purposes. Transitive dependencies are not automatically approved and must be evaluated case by case:

- `anyhow`: at boundaries where control passes outside the source, e.g., library interfaces and `main()`
- `thiserror`: all internal error handling with fully typed errors
- `axum`, `tower`, `hyper`: hosting web services
- `base64`, `hex`, `num-traits`, `unicode-normalization`, `uuid`: useful utilities
- `regex`: if the size and complexity cost of regular expressions is justified
- `chrono`, `chrono-tz`: time and date handling
- `clap`: CLI interfaces
- `config`: configuration file handling
- `ctrlc`: exit/termination signal handling
- `dirs`: standard config/data/cache directories
- `fern`, `log`: logging
- `r2d2`, `r2d2_sqlite`, `rusqlite`: SQLite (use `rusqlite` with the `bundled` feature)
- `reqwest`: HTTP(S) client calls
- `serde`, `serde_json`: serialization/deserialization
- `tokio`: asynchronous runtime
- `url`, `urlencoding`: URL handling
- `sha2` and `hmac`: hashing
