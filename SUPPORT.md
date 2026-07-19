# Support

Need help? This document explains the best way to get support for this project.

Please follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all project spaces.

## Where to start

Before opening a new thread, check:

- Project overview and contracts:
    - [Project summary](./docs/design/ProjectSummary.md)
    - [Protocol notes](./docs/design/Protocol.md)
    - [Tool contracts](./docs/design/ToolContracts.md)
    - [Configuration model](./docs/design/Configuration.md)

If you’re debugging an integration problem, it’s often helpful to confirm that the MCP host is launching Alfred with the expected working directory (Alfred uses the process CWD as its workspace root).

### Troubleshooting

- Alfred writes a structured json runtime log under the OS user-data directory (subfolder `alfred/logs/`, file `runtime.json`).
- If you are using Alfred via an MCP host, you can usually retrieve relevant log lines with the `log_search` tool.

## Questions

For “how do I…” questions and usage help, use GitHub Discussions:

- Browse discussions in your repository host (if enabled).
- Ask a question in the Q&A category (if available).

When asking a question, include:

- What you expected to happen and what happened instead.
- The version/commit you are using.
- Your environment (OS, runtime versions, configuration).
- Logs or screenshots (redact secrets).

## Bugs and feature requests

For confirmed bugs and actionable work items, open an issue:

- Use the repository’s issue tracker (if enabled).

Before opening an issue:

- Search existing issues and pull requests to avoid duplicates.
- Make sure you are on a supported version.

## Security issues

Do not open public issues for security vulnerabilities.

See [SECURITY.md](SECURITY.md) for responsible disclosure instructions.

## Response times

There is no guaranteed response-time SLA for this project.
