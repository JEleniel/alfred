---
applyTo: '*.rs'
---

# Rust Coding Guide

Formatting and style conventions for Rust source code in this repository.

If present, the repository's Rust formatting config (`rustfmt.toml`) is the source of truth for formatting.

## Formatting Rules

- **Edition**: Use Rust 2024+ unless `Cargo.toml` specifies older. Do not use `mod.rs`.
- **Organization**: Use cohesive modules; minimize top-level `*.rs` sprawl.
- **Formatting**: use `cargo fmt`.
- **Indentation**: Prefer hard tabs.
- **Line endings**: Use `\n`.
- **Comments and docs**:
    - Keep comments accurate and up to date.
    - Wrap comment text at 100 characters.
    - Use `//!` for module/crate docs and `///` for item docs.
    - Follow the [rustdoc book](https://doc.rust-lang.org/rustdoc/).
- **Imports**: Group standard/external/crate. Put `mod` declarations first (after module docs), then a blank line, then `use`.
- **Patterns**: Use `_` for single-item wildcards and `..` for rest patterns.
- **Initialization**: Use field init shorthand when possible.
- Baseline: [2024 Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/index.html).

## Coding Rules

- Apply these rules to Rust code you write or modify. Do not rewrite unrelated existing code solely for conformance.
- Do not use `unwrap`, `expect`, `panic`, or similar in non-test code unless explicitly instructed.
    - Exception: `todo!()` and `unimplemented!()` are permitted for clearly unimplemented paths that must fail fast and communicate intent.
- Add documentation comments for new modules and new public items.
- Avoid `unsafe` unless a specific API requires it.

## Error Handling

- Library code SHOULD return typed errors (prefer `thiserror`).
- Executables and application boundaries MUST use `anyhow` for ergonomic context (`anyhow::Context`) and `anyhow::Result`.
- Prefer `?` plus `#[from]` when mapping between error types.
- All errors MUST be either handled or logged. The code should crash only if there is no choice.

## Notes

- For services, configure TLS to use TLS 1.3 unless the user explicitly requires otherwise.
- For services, configure logging to write `TRACE`, `DEBUG`, `INFO`, and `WARN` to stdout and `ERROR` to stderr.

## Acceptance Criteria

- Relevant `cargo` checks pass (`fmt`, `clippy`, and targeted `test`).
