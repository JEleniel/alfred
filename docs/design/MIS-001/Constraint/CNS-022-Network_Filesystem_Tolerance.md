# Constraint: CNS-022 Network Filesystem Tolerance

Alfred MUST tolerate workspaces located on network-backed filesystems (e.g., SMB/NFS/SSHFS) and MUST NOT assume strict local-POSIX filesystem semantics; when guarantees (atomicity, locking, timestamps) cannot be met, Alfred MUST degrade safely and report deterministically.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:23:24Z | copilot | create |
