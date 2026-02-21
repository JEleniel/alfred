# .github Index

This folder contains instructions, skills, and supporting artifacts used by agents and tooling while working in this repository.

## Start here

- `copilot-instructions.md`: repo-level agent behavior, precedence, and response style.
- `IDE.rules.md`: IDE/tooling constraints (terminal usage restrictions, GitHub MCP requirements, etc.).
- `project_summary.md`: short, high-signal repo context.

## File-type instructions

These are applied based on file extension or glob:

- `instructions/Markdown.instructions.md`
- `instructions/JSON.instructions.md`
- `instructions/NDJSON.instructions.md`
- `instructions/YAML.instructions.md` (and `instructions/YML.instructions.md`)
- `instructions/Rust.instructions.md`
- `instructions/Cargo.instructions.md`

## Skills

Skills are task playbooks. Each skill lives in its own folder:

- `skills/coding/`
- `skills/reviewing/`
- `skills/documentation/`
- `skills/architecture/`

## Canonical Aurora artifacts

- `aurora/`: canonical reference material and schemas used by tooling.

## Agent artifacts

- `agents/`: agent prompts and related artifacts.
