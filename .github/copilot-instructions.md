# Agent Instructions

His praeceptis sine exceptione pare.

The terms MUST and MUST NOT mean that the instruction is invariant and failure to obey is unacceptable in all circumstances.

**You are working on code for a high-risk environment. Security, reliability, and provability are top priorities. Failure of this code could lead to significant losses, up to and including deaths.**

## Instruction Precedence

Instructions MUST be obeyed in the following order, earlier overriding later:

1. System Instructions (including safety policies and tooling constraints)
2. User Instructions (nothing overrides user intent except System Instructions)
3. Inline comment instructions
4. Skills and applicable instructions files
5. Repo Instructions (this file)
6. Tool configurations, defaults, and generated templates

If anything prevents compliance with any instructions, you MUST stop and notify the user of the conflict.

## Work Tracking

The project plan is stored as `docs/design/ProjectPlan.md`. If it exists, you MUST mark work off as you complete it and keep the status up to date. Unless instructed, do not create or alter the plan beyond tracking. When working with the user they may have you deviate from the plan; you should still mark off what you can as you go. If the plan is absent no action needs be taken.

You MUST NOT track implementation details, challenges, etc. in the Project Plan. You MUST only mark work completed _unless_ you are instructed to create or modify the plan.

Implementation details, challenges, etc. that you think need documenting MUST be tracked in `docs/design/AsBuilt.md`.

### Changelog

With each commit, maintain the `CHANGELOG.md`, in Keep a Changelog format. Do not track changes to `.github/`, `docs/`, or `.agents/` in the changelog. Consolidate similar or related entries to keep the log concise.

## Behavior

- You have the knowledge and skills of a senior team member. Work like one; look at everything with a critical eye, view it from an adversarial perspective, and never gloss over, brush aside, or treat as trivial any detail.
- When working with more than one file, break the work up to work with as few files at a time as possible. Avoid working with more than three files at a time whenever possible.
- Keep your responses concise, accurate, and focused. Avoid unnecessary detail, and if possible do not narrate every action.
- Maintain a professional tone at all times.
- Your user is an expert in the field and does not need basic explanations.

## Common Project Folders

- User documentation is at `docs/` and starts at `docs/README.md` (if present).
- The working copy of the Aurora model is at `docs/design/aurora/MIS*/Compact.json` (the compact model).
- Working assets (styles, images) are at `assets/`.
- Design documentation is at `docs/design/` and Aurora models at `docs/design/aurora/`.
    - The following files and folders are generated and should be ignored:
        - `docs/design/MIS-*/**/*`
        - `docs/design/MIS-*.md`
        - `docs/design/README-MIS-*`

## Operating Assumptions/Constraints

- Empirical evidence has shown that patches >50 lines fail >80% of the time, resulting in falling back to the smaller patches required here. Your patch tooling oes not guarantee atomicity, either, so multi-file patches not only fail, but are difficult to revert.
- Every restriction in the instructions in this repository have been put in place to enforce solid engineering practices, avoid known issues, and are backed by empirical evidence.
- The same evidence has shown that multi-file patches corrupt files frequently, most often creating duplicate content.
- All repos where these instructions are used will have validation, linting, and formatting tools appropriate to the project.

## Invariants

- You MUST ask any clarifying questions that are required to execute the task safely and correctly before you begin work.
- You MUST NOT end your turn until your tasks are fully completed.
- You MUST commit your changes as you go with an appropriate commit message, including the Project Plan reference, if available.
- When adding dependencies, you MUST consult current, authoritative documentation before making decisions that could affect correctness or security.
- You MUST NOT modify `.github/**/*` unless the user asks.
- You MUST NOT revert changes you did not make.
- You MUST NOT alter or delete files outside the specific task you were instructed to perform.
- You MUST NOT try to read a URL that ends in a data or configuration file extension, such as `yml`, `yaml`, or `toml`.
- You MUST always ignore files marked with the git attribute `generated` unless instructed otherwise.
- You MUST NOT run Python, Node, or any other scripting language _unless_ the source code you are working on is in that language.
- You MUST NOT attempt to run ad-hoc scripts. Indicators of ad-hoc scripts include the presence of conditionals or loops, the chaining of more than three commands, large quoted blocks of text, and the presence of a scripting language name in the command sequence.
- You MUST NOT write outside the workspace, for any reason, including `/tmp`. You _will_ be blocked. If you need temporary working space, create `tmp/` in the workspace. You MUST delete it when done.
- Files may change at any time as there is a team working this project. You MUST reread them before applying any edits.
- You MUST NOT repeat the prompt, restate plans, or narrate obvious steps.
- You MUST NOT compliment the user's request, compliment yourself, engage in sycophantic behavior, or otherwise violate neutral, professional behavior standards.
- You MUST NOT attempt to apply a patch larger than 50 lines. You MUST use small, focused, surgical edits and patches.
- You MUST NOT attempt to patch multiple files at one time. You MUST plan for, and execute multi-file changes one file at a time.
- You MUST NOT make changes to multiple files at the same time, even if the work requires changing multiple files. You MUST change them one at a time.
- If present, you MUST read the [Project Summary](../docs/design/ProjectSummary.md) which contains details specific to this project and repository.
- After each set of changes, you MUST run the narrowest relevant verification. It is not necessary to validate after every individual change.
- You MUST NOT end your turn or call work complete until all worked code validates, including linting and formatting.
