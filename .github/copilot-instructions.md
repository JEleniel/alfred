# Agent Instructions

His praeceptis sine exceptione pare.

Permission to perform actions uses the following terms:

- NEVER - the action is strictly and unequivocally forbidden
- ALWAYS - the action MUST be taken when the specified conditions are met

This is a high-risk environment. Security, reliability, and provability of the code critical. Failure of this code could lead to deaths.

## Instruction Precedence

ALWAYS obey in the following order:

1. System Instructions
2. User Instructions
3. Inline comment instructions
4. Skills and applicable instructions files
5. Repo Instructions
6. Tool configurations, defaults, and generated templates

ALWAYS stop and notify the user if there is a conflict between instructions.

## Work Tracking

These instructions apply to the Project Plan at `docs/design/ProjectPlan.md` if present:

- ALWAYS mark items complete when applicable.
- NEVER alter the plan in any other way unless instructed.
- If the user has you deviate from the Project Plan, ALWAYS update any items completed even if they are out of order.
- NEVER track implementation details, challenges, or similar in the Project Plan.
- ALWAYS track implementation details, challenges, variances from the design, and similar in `docs/design/tracking/AsBuilt.md`. Create the file if it does not exist.

### Changelog

- Unless the Project Summary indicates the project is prerelease, ALWAYS maintain a `CHANGELOG.md`, in Keep a Changelog format. Create it if absent.
- NEVER track changes to `.github/`, or `docs/` in the changelog.
- ALWAYS consolidate similar or related entries to keep the log concise.
- If available, include a link to the design artifact that defines the change.

## Behavior

- ALWAYS break the work up and change as few files as possible.
- ALWAYS edit one file at a time, serializing multi-file changes.
- ALWAYS keep responses concise, accurate, and focused.
- ALWAYS avoid unnecessary detail.
- NEVER provide basic explanations unless asked.

## Common Project Folders

- User documentation: `docs/` starting with `docs/README.md` (if present)
- Design documentation: `docs/design/`
    - Full Aurora model(s): `docs/design/aurora/`
    - Aurora compact models: `docs/design/aurora/MIS-**/Compact.json`
- Working assets (styles, images): `assets/`
- Analysis and reviews `docs/design/analysis/`.
- Ignore the following:
    - `docs/design/MIS-*/**/*`
    - `docs/design/MIS-*.md`
    - `docs/design/README-MIS-*`

## Invariants

### Pre-Work

- ALWAYS ask any clarifying questions before beginning work.
- ALWAYS consult current documentation before using a new dependency.
- ALWAYS read the [Project Summary](../docs/design/ProjectSummary.md), if present.

### Scope and Boundaries

- NEVER end your turn until your tasks are fully completed.
- NEVER alter files unnecessary to the specific task you were instructed to perform.
- NEVER alter the scope of a task.
- NEVER alter resource ownership or boundaries unless unavoidable to complete the task.
    - Unavoidable means the task cannot be completed without crossing that boundary, not that crossing it would be convenient.

### File and Workspace Operations

- NEVER modify `.github/**/*` unless instructed.
- NEVER delete files without direct permission from the user.
- NEVER try to read a URL that ends in a data or configuration file extension, such as `yml`, `yaml`, or `toml`.
- NEVER write outside the workspace, including `/tmp`.
- If you need temporary working space, ALWAYS create `tmp/` in the workspace.
- ALWAYS delete `tmp/` if you create it.
- NEVER run ad-hoc scripts. This includes:
    - Commands with conditionals or loops
    - Use of the chaining operation (`&` or `&&`)
    - Commands with large quoted blocks of text
    - A scripting language name in the command sequence

### Editing Discipline

- ALWAYS verify that files have not changed before editing them.
- NEVER revert changes you did not make.
- NEVER revert a change that happens between reads of a file.
- NEVER apply a patch larger than 50 lines.
- ALWAYS make small, focused, surgical edits and patches.

### Quality Assurance

- ALWAYS verify changes before ending a turn.
    - Run formatters.
    - Run linters.
    - Run applicable tests.
    - NEVER validate after every individual change.
- NEVER end a turn until all worked code validates.

### Code Quality and Design

- NEVER "optimize things away" unless you fully understand why they are there. If in doubt, ask.
- NEVER violate One Source of Truth: every capability must have one, and only one implementation.
- ALWAYS decompose code into focused, single purpose modules with clear ownership and responsibilities.
