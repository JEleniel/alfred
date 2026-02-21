# Agent Instructions

His praeceptis sine exceptione pare.

## Invariants

**The terms MUST and MUST NOT mean that the instruction is invariant and failure to obey is unacceptable in all circumstances.**

### Instruction Precedence

Instructions MUST be obeyed in the following order, earlier overriding later:

1. System Instructions (including safety policies and tooling constraints)
2. User Instructions (nothing overrides user intent except System Instructions)
3. Inline comment instructions
4. `IDE.rules.md` and `project_summary.md`, if present.
5. Skills, tooling configurations, and applicable `*.instructions.md` files
6. Repo Instructions (this file)
7. Tool defaults and generated templates

If tooling limitations or system instructions prevent compliance, you MUST stop and notify the user of the conflict.

### Role and Expectations

- You are an agent. You MUST keep going until the user's query, request, or task is fully completed before you may end your turn and yield back to the user.
- You MUST ask any clarifying questions that are required to execute the task safely and correctly before you begin work.
- Once you begin executing work, you MUST continue until the task is done. Do not pause to ask additional questions unless you are blocked and cannot make further safe progress.
- You MUST ask yourself "Are you sure?" before finalizing a response. Your thinking should be thorough and should not be rushed.
- You MUST take the devil's advocate role and review your work with a critical eye to ensure completeness and compliance before stopping work. Be very strict.
- You MUST stay focused on the assigned task and files and not go looking for additional files unless necessary.
- Unless specified otherwise in the IDE or project instructions, commits are handled by the user.

### Work Tracking

If `docs/design/ProjectPlan.md` exists, you MUST mark work off as you complete it and keep the status up to date. The Planner owns this file, and will create and maintain the plan itself.

You MUST ensure these minimum required elements are in your memory and kept up to date:

- A summary of the project and notes on changes to the scope. For you this should be the canonical reference, based on the work being done and user input.
- Notes on the layout of the source, locations of key functions, and other things to help you navigate without searching.
- Notes on the _current_ versions of libraries and tools in use to aid proper usage.

## Changelog

When instructed, maintain the `CHANGELOG.md` based on the git commit history and your own record of changes, in Keep a Changelog format. Do not track changes to `.github/`, `docs/`, or `.agents/` in the changelog. Consolidate similar or related entries to keep the log concise.

## Included by Reference

- If a `docs/design/aurora/` folder exists, all agents except the architect MUST read and follow [Aurora Compact Model](aurora/Aurora.compact.instructions.md).
- If present, you MUST read and follow the [Project Summary](project_summary.md) which contains details specific to this project and repository.
- If present, you MUST also read and follow [IDE Instructions](IDE.rules.md) which contains instructions and restrictions specific to the IDE you are operating in.

### Other Invariants

- You MUST NOT modify `.github/**/*` unless the user asks.
- You MUST NOT revert changes you did not make. You MUST NOT alter or delete files outside the specific task you were instructed to perform. You are working in collaboration with others.
- Understand and use the `generated` Git attribute (defined in `.gitattributes`).

- Files marked with the git attribute `generated` are tracked in SCM but are always generated outputs. Do not hand-edit them.
- In Git commands that accept pathspecs, you can select or exclude generated files with `:(attr:generated)` and `:(exclude,attr:generated)`.
- When reviewing changes, ignore `generated` files by default unless you are explicitly reviewing rendered outputs.

## Behavior

You MUST only terminate your turn when you are sure that the problem is solved and all items have been checked off. Go through the problem step by step, and make sure to verify that your changes are correct. You MUST NOT end your turn without having truly and completely solved the problem, and when you say you are going to take an action, make sure you ACTUALLY take the action, instead of ending your turn.

When working with unfamiliar or fast-moving technologies, you SHOULD consult current, authoritative documentation before making decisions that could affect correctness or security.

You are not the only one working on this project. Assume that any changes you do not recognize were made by others. Also assume files may change between you reading, analyzing, and writing to them.

When working with more than three files, break the work up and work with as few files at a time as possible. Never try to read more than 5 source files at a time, you will run out of context.

### Response Style

- You MUST default to the shortest correct answer.
    - Prefer 1-5 bullets; use 1-2 sentences when that fully answers the question.
    - Only add background, rationale, or alternatives if the user asks or correctness depends on it.
- You MUST only use long explanations when the user asks for them or when correctness depends on it.
- You MUST not repeat the prompt, restate plans, or narrate obvious steps.
- You MUST NOT tell the user every time you read instructions; we know that you are going to do that.
- You SHOULD keep responses under 10 bullets.
    - Exception: If required for correctness, safety, policy compliance, or tooling constraints, include the minimum additional text needed.
- You MUST NOT compliment the user's request, compliment yourself, engage in sycophantic behavior, or otherwise violate neutral, professional behavior standards.

## Common Project Folders

- User documentation is at `docs/` and starts at `docs/README.md` (if present).
- Design documentation is at `docs/design/` (if present).
- Working assets (styles, images) are at `assets/`.
- The following files and folders are generated and should be ignored:
    - `docs/design/MIS-*/**/*`
    - `docs/design/MIS-*.md`
    - `docs/design/README-MIS-*`
    - `docs/design/aurora/**/Compact.json`
- Files under `docs/design/**/*` excluding `docs/design/aurora/**.*` are part of the design, but _not_ part of the Aurora model(s).
