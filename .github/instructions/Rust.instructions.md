---
applyTo: '*.rs'
---

# Rust Coding Guide

If present, the repository's Rust formatting config (`rustfmt.toml`) is the source of truth for formatting.

## Formatting Rules

- **Edition**: Use the edition specified in `Cargo.toml` (assume Rust 2024+ unless specified otherwise).
- **Organization**: Organize code into cohesive modules; minimize top-level `*.rs` sprawl. Do not use `mod.rs`.
- **Formatting**: Use `cargo fmt`.
- **Indentation**: Use spaces; let `rustfmt` enforce indentation and alignment.
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
- Do not use functions that _only_ return a constant value.
- Configure logging to write `TRACE`, `DEBUG`, `INFO`, and `WARN` to stdout and `ERROR` to stderr. Optionally log to a structured file.

## Error Handling

- Library code SHOULD return typed errors (prefer `thiserror`).
- Executables and application boundaries (anywhere control leaves our code) MUST use `anyhow`.
- All errors MUST be either handled or logged. The code should crash only if there is no choice.

## Acceptance Criteria

- Relevant `cargo` checks pass (`fmt`, `clippy`, and targeted `test`).
