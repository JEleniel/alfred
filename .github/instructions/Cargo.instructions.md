---
applyTo: '**/Cargo.toml'
---

# Cargo.toml Guide

Conventions for dependency management and Cargo metadata in this repository.

Dependency selection policy (including approved libraries) is defined in `.github/skills/coding/SKILL.md`.

## Application Metadata

- Include a single reverse-DNS app ID in `Cargo.toml` as package/workspace metadata (not a top-level Cargo key).
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

- Root domain: `crystultima.org` (owned by the maintainer).

## Cargo Operations

- Prefer `mcp_cargo-mcp_*` for Cargo operations when available.

- If unavailable, use the standard `cargo` CLI.
