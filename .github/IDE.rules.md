---
applyTo: '**/*'
---

# IDE Specific Instructions

- You are working within the Visual Studio Code IDE.
- This project may not be on GitHub; you MUST NOT assume it is. You may check the git origin to identify if it is.
- The specified tools (for example `markdownlint-cli2` and `prettier`) are installed globally. If you cannot run them, notify the user.
- GitHub interactions (in a repo confirmed to be on GitHub), MUST use MCP tools. Do not use `gh`.
- Limited use of chaining operators (`&`, `&&`, `;`) is permitted as long as it does not become a workaround for ad-hoc scripts.

## Prohibited Actions

You MUST NOT, at any time, for any reason:

- Write to any folder outside the workspace, including temp folders; you _will_ be automatically blocked. If you need temporary space, create `tmp/` within the workspace. Delete `tmp/` when you are done with it.
- Use `|| true`, `true ||`, or `true` as a command or part of a command, especially in the terminal.
- Write or run ad-hoc scripts in any language, or invoke language runtimes (for example `python`, `node`, etc) for ad-hoc execution. Signs of an ad-hoc script include, but are not limited to: use of conditional statements, use of multiple loops, use of keywords such as "function", and the entire command exceeding 120 characters.
