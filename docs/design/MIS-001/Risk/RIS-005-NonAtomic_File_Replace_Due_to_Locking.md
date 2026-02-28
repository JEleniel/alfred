# Risk: RIS-005 Non-Atomic File Replace Due to Locking

Atomic replace operations may fail or become non-atomic on some platforms (notably Windows when destination files are locked), risking partial writes or inconsistent state if not handled explicitly.



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
| 2026-02-21T01:16:57Z | copilot | create |
