# Constraint: CNS-016 Symlink and Junction-Safe Boundary Checks

Workspace boundary enforcement MUST be robust to symlinks/junctions and path traversal tricks; boundary checks MUST use canonicalized/resolved paths and treat path inputs as hostile.



## Attributes

- **error_on_escape**: {"kind":"permission_denied","message":"symlink/junction escapes workspace"}
- **must_enforce_boundary_after_resolution**: true
- **platform_notes**: {"windows":{"junctions_must_be_validated":true}}
- **treat_as_user_intent**: true


## References

_No references defined._

## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
| 2026-02-22T17:35:20Z | copilot | change |
