# Alfred

[![License](https://img.shields.io/github/license/JEleniel/alfred?style=plastic)](#license)
[![Release](https://img.shields.io/github/v/release/JEleniel/alfred?style=plastic)](https://github.com/JEleniel/alfred/releases)
[![Issues](https://img.shields.io/github/issues/JEleniel/alfred?style=plastic)](https://github.com/JEleniel/alfred/issues)
[![Pull Requests](https://img.shields.io/github/issues-pr/JEleniel/alfred?style=plastic)](https://github.com/JEleniel/alfred/pulls)

[![Rust 2024](https://img.shields.io/badge/2024-gray?style=plastic&logo=rust&logoColor=white&label=Rust&labelColor=orange)](https://rust-lang.org)

Alfred is a local-only [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) stdio server that provides deterministic, workspace-scoped tools for common “agent in a repo” workflows.

Alfred is designed to be launched by an MCP host (for example an IDE extension) as a subprocess and to communicate exclusively over stdin/stdout using newline-delimited UTF-8 JSON frames.

## What Alfred provides

- Workspace boundary enforcement (tools operate within the current workspace).
- Deterministic outputs (stable ordering and conservative, testable result shapes).
- Safe-by-default posture (mutating capabilities are disabled by policy in the default build).
- A small, focused tool surface for querying workspace contents, searching, inspecting files, and reading logs and plan/memory state.

## Tool surface (current build)

The currently registered tools are:

- `workspace_dir`
- Workspace queries: `ls`, `read_range`, `file_stat`, `grep`, `search`, `diff`
- Logs: `log_search`
- Project plan: `plan_get`, `plan_update`, `plan_edit`, `plan_add`, `plan_delete`
- Memory: `memory_put`, `memory_get`, `memory_delete`, `memory_list`, `memory_search`
- `capabilities`

Notes:

- Some tools are intentionally disabled by policy in the default configuration (for example plan and memory mutation).
- `grep` and `search` are index-backed and can return a deterministic `tool_unavailable` error while the index is initializing.

## Getting started

### Prerequisites

- Rust stable toolchain

### Build

```bash
cargo build
```

### Run

Alfred uses the process current working directory as its workspace root. Run it from the repository root (or ensure your MCP host sets `cwd` accordingly).

```bash
./target/debug/alfred
```

Because MCP uses stdout for protocol frames, prefer launching the compiled binary from your MCP host configuration (rather than `cargo run`) to avoid toolchain output interfering with protocol handling.

## Configuration

Alfred reads optional JSON configuration files and currently uses them primarily for tool gating (`tools.disabled`).

Default locations:

- Workspace: `.alfred/config.json`
- User: OS config directory `alfred/config.json` (exact path depends on platform)

You can disable additional tools by adding them to `tools.disabled`:

```json
{
    "tools": {
        "disabled": ["search"]
    }
}
```

For the intended full configuration schema, see [`schemas/alfred.config.schema.json`](./schemas/alfred.config.schema.json).

### Environment variables

| Variable                                | Description                                                                                                                            | Default   |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| `ALFRED_RUNTIME_LOG_RETENTION_DAYS`     | Days to keep rotated runtime log archives.                                                                                             | `7`       |
| `ALFRED_INDEX_PERSISTENCE_LOCATION`     | Where to persist the workspace index (`workspace` by default for `.alfred/index`; set to `user` for `.alfred/index/<workspace_hash>`). | workspace |
| `ALFRED_INDEX_PERSIST_INTERVAL_SECONDS` | Persist interval for the workspace index writer.                                                                                       | `5`       |
| `ALFRED_INDEX_WATCH_DEBOUNCE_MILLIS`    | Debounce window for filesystem watch events.                                                                                           | `250`     |

## Development

### Test

```bash
cargo test
```

### Design and contracts

Most design/contract documentation lives under `docs/design/`:

- [`docs/design/ProjectSummary.md`](./docs/design/ProjectSummary.md)
- [`docs/design/AlfredOverview.md`](./docs/design/AlfredOverview.md)
- [`docs/design/Protocol.md`](./docs/design/Protocol.md)
- [`docs/design/ToolContracts.md`](./docs/design/ToolContracts.md)

## Versioning

The package version is defined in [`Cargo.toml`](./Cargo.toml) (currently `0.1.0`). Until Alfred reaches `1.0.0`, expect breaking changes as the tool surface and contracts evolve.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). This project uses the [Developer Certificate of Origin](DCO.md) and expects a DCO sign-off on commits.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.

## Support

See [SUPPORT.md](SUPPORT.md) for support channels and troubleshooting pointers.

## Acknowledgements

- Badges by [Shields.io](https://shields.io/)
- Protocol: [Model Context Protocol](https://modelcontextprotocol.io/)

## License

This project is licensed under the GNU GPL v3.0. See [LICENSE.md](LICENSE.md).
