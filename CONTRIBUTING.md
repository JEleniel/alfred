# Contributing

Thanks for taking the time to contribute.

By participating in this project, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Quick rules

- Be kind, be clear, and assume good intent.
- Keep changes small and focused.
- Prefer tests and documentation with behavior changes.
- All contributions must include a [DCO sign-off](DCO.md).

## Ways to contribute

- Report bugs.
- Propose features or improvements.
- Improve documentation.
- Review pull requests.

If you are unsure where to start, look for issues labeled “good first issue” or “help wanted”.

## Before you start

- Check existing issues and pull requests (if the repository is hosted on GitHub) to avoid duplication.
- For security issues, do **not** open a public issue. Follow [SECURITY.md](SECURITY.md).

## Development workflow

1. Create a branch with a descriptive name (for example `fix-null-pointer` or `feature-rate-limits`).
2. Make a focused set of changes.
3. Add or update tests as needed.
4. Update documentation where behavior changes.
5. Open a pull request and respond to review feedback.

Project-specific setup, build, and test commands belong in `README.md`. Keep this file focused on contribution expectations.

## Issues

When filing a bug, include:

- Expected vs actual behavior.
- Minimal reproduction steps.
- Version/commit and environment details.
- Logs and screenshots (redact secrets).

For feature requests, include:

- The user story / problem statement.
- Non-goals and constraints.
- Alternatives considered.

## Pull requests

### Quality bar

- PRs should do one thing.
- Keep diffs readable: avoid drive-by refactors.
- Update tests and docs as part of the same PR when applicable.
- Ensure CI is green before requesting review.

### What to include

- A clear description of the change and why it is needed.
- Any relevant issue/discussion links.
- Notes for reviewers (tradeoffs, follow-ups, rollout concerns).

## Code standards

- Follow the language/framework style guides for this project.
- Prefer clear names and small, composable functions.
- Comment only where intent is not obvious.
- Avoid adding dependencies without strong justification.
- Write tests that fail before they pass.

## Commits

### Commit messages

Write commit messages in the imperative mood and keep them specific.

Good examples:

```text
feat: add rate limit headers (#123)
fix: handle empty input in parser (#456)
docs: document configuration keys (#789)
```

Bad examples:

```text
fixed stuff
updated code
WIP
quick fix
```

### DCO sign-off (required)

All commits must include a DCO sign-off line.

- Use your real name and email.
- Use `git commit -s` to add the sign-off automatically.

Example:

```text
Signed-off-by: Jane Doe <jane.doe@example.com>
```

## Code review

For authors:

- Be responsive to feedback.
- Explain intent, not implementation trivia.
- Prefer follow-up issues over scope creep.

For reviewers:

- Be respectful and concrete.
- Ask questions early when something is unclear.
- Approve when the change meets the bar, not when it is “perfect”.

## Getting help

If you get stuck, see [SUPPORT.md](SUPPORT.md).
