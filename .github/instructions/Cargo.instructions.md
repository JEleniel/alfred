---
applyTo: '**/Cargo.toml'
---

# Cargo.toml Best Practices

## Application Metadata

- For applications targeting Apple or mobile platforms, include a single reverse-DNS app ID in `Cargo.toml` as package/workspace metadata (not a top-level Cargo key).
- Use a domain you control for reverse-DNS identifiers.
- For package-level metadata:

```toml
[package.metadata.crystultima]
app_id = "org.crystultima.<package_name>"
```

- If the app ID is workspace-wide:

```toml
[workspace.metadata.crystultima]
app_id = "org.crystultima.<workspace_name>"
```

## Core Settings

- Set `edition` to the latest stable value (`2024`) and declare `rust-version` (MSRV).
- Include `license`, `repository`, `readme`, and a clear `description`.
- Do not use the deprecated `authors` field.
- Use SPDX license identifiers.
- Use explicit major semver ranges; never `"*"`.
- Minimize dependency count; separate `dev-` and `build-` dependencies.
- Disable defaults when not required: `default-features = false`.
- Features must be additive only; never change existing behavior.
- Gate optional/heavy deps behind features.
- Keep `default` small and broadly usable.
- Avoid git dependencies in releases.
- Avoid `[patch]` except as a temporary override.
- Centralize versions in `[workspace.dependencies]`.
- Keep dependency scope as narrow as possible. Promote dependencies to workspace only if they are actually shared.
- Keep crates focused; avoid “catch-all” packages.
- Tune release builds intentionally (`lto`, `codegen-units`, `panic = "abort"` when appropriate).
- Optimize `[profile.dev]` for faster iteration.
- Regularly audit dependencies.
- Avoid hidden behavior in defaults or build scripts.

## Approved libraries

The following libraries are approved for use. Sublibraries include crates that share the parent prefix or are designed as companions.

- `anyhow`, `thiserror` for error handling
- `axum` (and sublibraries), `tower` (and sublibraries), `hyper` (and sublibraries) for web servers
- `base64`, `hex`, `num-traits`, `regex`, `unicode-normalization`, `uuid` for utilities
- `chrono` for time and date handling
- `clap` for CLI interfaces
- `config` for configuration file handling
- `ctrlc` for signal handling
- `dirs` (preferred) or `directories` for standard config/data/cache directories
- `tracing` and `tracing-subscriber` for logging
- `log` for logging API
- `ollama-rs` for Ollama access
- `openssl` or `rustls` (and sublibraries) for TLS
- `r2d2`, `r2d2_sqlite`, `rusqlite` for SQLite (use `rusqlite` with the `bundled` feature)
- `reqwest` for HTTP client calls
- `serde` (and sublibraries), `serde_json` for serialization
- `tokio` (and sublibraries) for async runtime
- `url`, `urlencoding` for URL handling
- `sha2`, and `hmac` for hashing
