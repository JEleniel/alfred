# Test Inventory

## `src/tests/integration_skeleton.rs` tests `alfred::app`

- `capabilities_follow_the_canonical_public_tool_surface`: verifies the public capability list is stable, sorted, and aligned with the canonical tool names and version constants.
- `tool_envelope_serializes_the_canonical_shape`: verifies the canonical tool envelope serializes with the expected top-level shape and metadata fields.
- `execution_mode_serializes_as_its_variant_name`: verifies `ExecutionMode` serializes using the Rust enum variant names directly.
- `router_dispatches_known_tools_and_rejects_unknown_names`: verifies the router resolves a known tool name to a route and rejects an unknown tool name.
- `invoke_tool_returns_the_capabilities_payload`: verifies the execution boundary returns an `ok` envelope containing the advertised tool list for `capabilities`.
- `invoke_tool_returns_the_workspace_root_payload`: verifies the execution boundary returns the current workspace root for `workspace_dir`.
- `invoke_tool_returns_the_status_payload`: verifies the execution boundary returns the current runtime status shape for `status`.
- `invoke_tool_reports_unimplemented_and_unknown_tools_deterministically`: verifies unimplemented tools return `tool_unavailable` and unknown tools return `invalid_argument` with deterministic details.

## `src/router/tests/router_tests.rs` tests `alfred::router`

- `dispatch_returns_the_expected_route_for_known_tools`: verifies that the router boundary resolves a known tool name to its canonical route and descriptor.
- `dispatch_rejects_unknown_tools`: verifies that unknown tool names are rejected at the dispatch boundary.

## `src/workspace_boundary/tests/workspace_boundary_tests.rs` tests `alfred::workspace_boundary`

- `normalize_workspace_relative_path_collapses_separators_and_dots`: verifies path normalization collapses separators and dot segments.
- `normalize_workspace_relative_path_accepts_absolute_paths`: verifies absolute paths normalize instead of failing before boundary checks.
- `normalize_workspace_relative_path_collapses_parent_dirs_within_workspace`: verifies `..` collapses when it remains inside the workspace path.
- `normalize_workspace_relative_path_accepts_windows_style_inputs_on_non_windows`: verifies Windows-style path text is treated as ordinary text on non-Windows hosts.
- `normalize_workspace_relative_path_rejects_windows_reserved_and_forbidden_names`: verifies Windows-specific filename rules are enforced only on Windows.
- `normalize_workspace_relative_path_rejects_traversal_outside_workspace`: verifies leading `..` that would escape the workspace boundary is rejected.
- `resolve_write_target_within_workspace_root_collapses_parent_dirs`: verifies write-target resolution collapses `..` segments while staying in the workspace.
- `resolve_write_target_within_workspace_root_accepts_nested_parent_dirs_inside_workspace`: verifies a parent-dir-heavy path still resolves to a location inside the workspace when the final target remains in-bounds.
- `resolve_write_target_within_workspace_root_joins_the_normalized_path`: verifies workspace-root resolution joins a normalized relative path.
- `resolve_write_target_within_workspace_root_accepts_absolute_paths_inside_root`: verifies absolute workspace paths are accepted when they stay within the workspace root.
- `existing_path_resolution_rejects_symlink_escapes`: verifies existing-path resolution refuses symlink escapes outside the workspace root.

### Gaps

- The stdio transport layer is still scaffold-only, so there are no behavioral tests for request framing or MCP-compliant tool-call handling yet.
