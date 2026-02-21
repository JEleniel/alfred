# Constraint: CNS-017 Cross-Platform Atomic Write Semantics

When claiming atomic mutations, Alfred MUST implement an atomic write/replace strategy that is correct on Linux/macOS/Windows; if atomic replacement cannot be guaranteed (e.g., locked destination on Windows), Alfred MUST fail explicitly rather than partially applying changes.



## Attributes

_No attributes defined._

## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
