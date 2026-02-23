# Constraint: CNS-021 UTF-8 Protocol and Path Encoding Handling

Alfred’s protocol outputs MUST be valid UTF-8 JSON/NDJSON. When encountering non-UTF8 filesystem paths or OS-specific path encodings, Alfred MUST handle them safely and report them deterministically (e.g., escaped/encoded representations) rather than crashing or producing invalid JSON.



## Attributes

- **non_text_fs_paths**: {"unix":{"escape_format":"\\xNN"},"windows":{"escape_format":"\\u{XXXX}","note":"ill-formed UTF-16 surrogate units may be escaped"}}
- **protocol_encoding**: {"json_must_be_utf8":true}
- **references**: {"protocol":"docs/design/Protocol.md"}
- **separator_normalization**: {"normalize_backslash_to_slash":true}
- **tool_path_format**: {"encoding":"utf-8","workspace_relative_posix":true}


## Links

_No links defined._

## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
| 2026-02-22T17:35:20Z | copilot | change |
