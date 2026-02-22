# Artifact (Data): ART-008 Memory Fact

A single structured memory entry persisted by Alfred for offline recall and search.



## Attributes

- **formats**: ["json"]
- **schema**: {"category":"string (recommended: general|user_preferences|file_specific|bootstrap_and_build)","citations":"string","created_at":"string (RFC3339 UTC; seconds preferred; max milliseconds)","fact":"string","id":"string (stable, recommended uuid)","reason":"string","subject":"string","tags":"optional array of strings (stable-sorted ascending; recommended lower-case kebab-case)","updated_at":"string (RFC3339 UTC; seconds preferred; max milliseconds)"}
- **search_fields**: ["subject","fact","citations","reason","tags"]


## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-22T12:44:59Z | copilot | create |
| 2026-02-22T12:51:52Z | copilot | change |
| 2026-02-22T13:04:50Z | copilot | change |
