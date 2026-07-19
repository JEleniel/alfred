# Alfred Fixes

This document tracks follow-up changes that need to be applied to Alfred. The protocol contract itself is defined in [`docs/design/Protocol.md`](./Protocol.md).

## Fixes

1. Switch path serialization to percent-encoding for protocol transport.
    - Replace the bespoke `\xNN` and `\u{...}` path rendering format with a standard percent-encoding scheme.
    - Keep protocol handling limited to transport shape; path semantics and platform-specific behavior belong outside the protocol encoding/decoding.
    - Continue to represent `.` literally and keep `Path` / `PathBuf` responsible for native path traversal and joining semantics.
    - Reject unsafe or ambiguous path text at the boundary instead of attempting to round-trip it.
    - Reject `/`, `<`, `>`, `:`, `"`, `\\`, `|`, `?`, `*`, `\0`, ASCII `0..=31`, Unicode control characters, and invalid Unicode characters.
    - Allow spaces.
    - Reject reserved Windows filename names for a file name component: `CON`, `PRN`, `AUX`, `NUL`, `COM_n_`, and `LPT_n_` where `n` is `0..=9`.
    - Reject filenames that end in `.`.
    - Update the relevant code paths and tests to reflect the new reversible encoding.
2. Place tests in a sibling `tests/` subfolder relative to the source file under test, with `_tests` appended to the base filename (for example, `src/module/needs_testing.rs` → `src/module/tests/needs_testing_tests.rs`).
3. Limit visibility to the narrowest required scope.
    - Prefer private items before `pub(crate)` before `pub`.
    - Update all instances to match the narrowest required visibility.
4. Refactor inner blocks, match actions, and `if` / `else` blocks longer than five lines into functions when practical.
5. Forbid one-line helper functions, including preformatted logging helpers.
6. Keep function logic out of `Ok(...)` expressions.
7. Handle internal errors with `thiserror` enums and rectify them when possible; remove error-mapping helper functions; reserve `anyhow` for control boundaries between Alfred and the outside world.
8. Do not create functions that exist only to return a constant, and do not create helper functions whose only job is a one-line wrapper.
9. Do not return `Option<>` when the return value can never be `None`.
10. Keep unrelated functionality out of the same module; split by responsibility, and move stateful behavior into a struct when needed.
11. Prefer existing library functionality over bespoke implementations; for example, use `fern` to create log files and handle rotation instead of reimplementing that logic.
12. Make the log archiver trigger only when a new log file is created in the log folder.
13. Move the archiver into a logging submodule instead of keeping it at the logging module root.
14. Remove support for the bespoke `.ndjson` extension.
15. Implement `anyhow` logging only at the `main` level.
16. Write all error messages as full English sentences.
17. Reorder source items as `mod`, `use`, code, error enum, tests, with `struct` before `impl` and simple `impl` blocks before trait `impl` blocks.
18. Export canonical names to eliminate redundant long paths; prefer `crate::ToolRegistry` over `crate::tools::tool_registry::ToolRegistry` unless the longer form adds clarity.
19. Remove compatibility code, deprecated support, and similar prerelease baggage.
20. Replace repeated `if let Some(...)` extraction with a single extraction followed by `if` / `else if` / `else` when appropriate.
21. Refactor the following over-limit functions:
    - `build_tools_call_response` (`72` lines)
    - `execute_move` (`54` lines)
    - `load_from_paths_with_host` (`201` lines)
    - `extract_redaction_rules` (`129` lines)
    - `remove_path_entries` (`56` lines)
    - `load_snapshot_from_index` (`63` lines)
    - `memory_status` (`52` lines)
    - `revert_single_entry` (`78` lines)
    - `handle_search` (`62` lines)
    - `read_range_from_disk` (`63` lines)
    - `handle_stat` (`53` lines)
