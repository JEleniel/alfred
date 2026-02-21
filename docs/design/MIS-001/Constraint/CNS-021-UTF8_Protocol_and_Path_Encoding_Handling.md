# Constraint: CNS-021 UTF-8 Protocol and Path Encoding Handling

Alfred’s protocol outputs MUST be valid UTF-8 JSON/NDJSON. When encountering non-UTF8 filesystem paths or OS-specific path encodings, Alfred MUST handle them safely and report them deterministically (e.g., escaped/encoded representations) rather than crashing or producing invalid JSON.



## Attributes

_No attributes defined._

## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
