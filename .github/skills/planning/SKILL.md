---
name: planning
description: Use this skill when creating or maintaining a project plan, including task decomposition, sequencing, dependencies, and progress tracking.
---

# Planning Skill

## General guidelines

- Evaluate plans with an adversarial mindset. Surface hidden dependencies, missing controls, and sequencing that can create rework or unsafe delivery risk.
- Verify that the plan reflects current requirements, constraints, and architecture artifacts.
- Record explicit rationale, constraints, assumptions, and trade-offs for planning decisions.
- Prioritize plans that are auditable, adaptable, and progress-verifiable without interpretation.
- This skill must **never** generate, modify, or suggest changes to source code or documentation other than the project plan.
- If a `docs/design/aurora/` folder exists, read and follow [Aurora Compact Model](../../aurora/Aurora.compact.instructions.md) to understand the design.
    - Use the full Aurora instructions only when applying the Architecture skill.

## Principles of Elegant Planning

- Keep plan structure and intent understandable without external explanation.
- Solve the planning problem with the smallest complete set of tasks and controls.
- Compose workstreams orthogonally without cross-cutting, duplicated ownership.
- Handle similar work with consistent task structure, status semantics, and review gates.
- Keep decisions, constraints, and dependencies discoverable and auditable.
- Keep plan content focused on outcomes and controls, not implementation convenience.

### Indications of Poor Planning

- God Tasks: A single task owns unrelated concerns or spans too many components.
- Leaky Plans: Tasks require undocumented context to execute correctly.
- Inconsistent Vocabulary: Multiple terms are used for the same planning concept.
- Plan-Reality Drift: Task status and deliverables do not match actual progress.
- Unbounded Task Scope: Tasks define broad intent without objective completion criteria.
- Decision Orphans: Priority, sequence, or dependency choices lack rationale.

## Deliverables

- Root planning artifacts in `docs/design/ProjectPlan.md`.
- Keep `docs/design/ProjectPlan.md` current as scope, constraints, and sequencing evolve.
- Produce a structured plan with priorities, statuses, dependencies, and verifiable deliverables.
- Ensure the plan includes implementation, analysis, architecture, code review, and documentation review tasks, plus remediation tasks for identified gaps.
- Decompose work into inseparable units that can be independently tracked and verified.
- Keep review order as analysis, architecture, code, and documentation unless the user explicitly overrides it.
- Use markdown nesting for hierarchy and sequencing; do not number plan items.
- Split tasks that contain multiple deliverables into separate subtasks.
- Use the following task format in the Project Plan:

```markdown
- [x] <priority P0-P3>: Build the DS3231 Driver
    - Description: Implement the driver for the DS3231 Real Time Clock.
    - Deliverable:
        - The driver receives resources from the HAL, including pins and busses.
        - The driver exposes a clean, simple, stateless interface.
        - Errors are locally typed, `#[from]` conversions are used everywhere possible, and all fallible functions return `Result<_, _x_Error>`.
    - References: (Optional) Links to relevant Aurora cards, documentation, designs, or project artifacts that provide context for the task.
    - Notes: (Optional) Concise additional information directly related to executing the plan. Implementation details, functional notes, and other details are documentation, not notes.
    - Depends on: (Optional) Hardware Abstraction Layer
    - Subtasks:
        - [ ] `new` - accepts an I2C bus and time zone (name) and uses them to initialize the device.
        - [ ] `get_time` - function that returns now in UTC.
        - [ ] `get_time_local` - function that returns now in the time zone.
        - [ ] `set_time_zone` - function that updates the time zone.
        - [ ] `set_time` - function that sets the time on the device.
```

## Operating Procedure

1. Confirm planning scope, stakeholders, constraints, success criteria, and target path from the user request and related artifacts.
2. Inspect existing artifacts under `docs/design/`, including the current plan and requirements.
3. Update planning artifacts in place unless scope boundaries require a separate planning artifact.
4. Capture or update assumptions, constraints, dependencies, review gates, and key sequencing decisions.
5. Decompose requested work into inseparable, independently verifiable tasks.
6. Assign priorities and dependencies that preserve flow and minimize rework.
7. Ensure the required review order appears explicitly in the plan unless user direction overrides it.
8. Keep terminology, identifiers, and references consistent across planning and design artifacts.
9. Link references when they materially support plan execution or verification.
10. Validate the result against this skill's deliverables and checklists before completion.

## Validation Checklists

### Plan Structure Checklist

- Correct plan target path.
- Plan format matches the documented structure.
- Hierarchy is consistent and unambiguous.
- Priorities are present and within the defined range.
- Plan includes implementation, analysis, reviews, documentation, and remediation tasks.
- Tasks cover every required step needed to reach goals and align with design artifacts.
- Review order is analysis, architecture, code, and documentation unless explicitly overridden.

### Task Quality Checklist

- Each task touches one subsystem or one inseparable unit of work.
- Each task includes references when they add traceability and context.
- Task order minimizes total work and avoids predictable rework.
- Each task depends only on equal or higher priority tasks.
- Every task includes objective verification criteria.
- Tasks are specific, directly actionable, and avoid vague language.
- Deliverables are specific, concise, and objectively verifiable.

### Dependency and Sequencing Checklist

- Dependencies form a clean directed structure with no circular dependencies.
- Tasks depend only on equal or higher priority parents.
- Research tasks are included for any unresolved question.
- Dependencies are explicit where they materially affect sequencing.
- Prerequisites are listed before dependent tasks.
- Status reflects actual progress and blockers.

## Things to Watch For

### Planning Foot-Guns

- Vague tasks with no verifiable deliverable.
- Hidden dependencies.
- Overstuffed tasks that bundle unrelated work.
- Review gates missing from delivery plans.
- Implementation details masquerading as project goals.
- Status drift between the plan and reality.
- Tasks that are not broken into the smallest possible unit of work that can be independently tracked and verified.
- Tasks that depend on lower priority tasks, which can lead to blockers and delays.
- Tasks that touch more than three files or components, which can indicate that the task is not sufficiently decomposed.

## Cross-skill tasks

- If the request includes code or documentation edits beyond the project plan, split the work into phases and use the appropriate skill for each phase.
- If the request is implementation-focused, do not implement changes under this skill; use the Coding skill.
- If the request requires documentation work beyond the project plan, use the Documentation skill for that phase.
- If the request requires architecture modeling or design updates, use the Architecture skill.
- If the request is to review an existing plan, use the Reviewing skill to record findings.
