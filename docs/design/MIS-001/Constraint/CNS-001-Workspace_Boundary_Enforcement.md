# Constraint: CNS-001 Workspace Boundary Enforcement

Alfred MUST NOT read or write outside the current workspace. All filesystem operations MUST validate workspace-relative paths and MUST prevent path traversal and symlink/junction escapes.



## Attributes

- **error_on_violation**: {"kind":"permission_denied","message":"path escapes workspace"}
- **path_rules**: {"normalize_backslash_to_slash":true,"path_separator":"/","reject_absolute_paths":true,"reject_path_traversal":true,"tool_paths_are_workspace_relative":true}
- **policy**: {"enforcement_points":["request_validation","filesystem_resolution","index_ingestion"],"scope":"workspace_root_only","windows":{"treat_junction_as_symlink":true}}
- **workspace_escape_prevention**: {"must_verify_resolved_target_under_root":true,"resolve_to_realpath_before_use":true}


## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-22T17:35:20Z | copilot | change |
