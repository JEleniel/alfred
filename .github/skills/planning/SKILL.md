---
name: planning
description: Use this skill when creating or maintaining a project plan, including task decomposition, sequencing, dependencies, and progress tracking.
---

# Planning Skill

## General guidelines

- This skill must **never** generate, modify, or suggest changes to source code or documentation other than the project plan.
- If a `docs/design/aurora/` folder exists, read and follow [Aurora Compact Model](../../aurora/Aurora.compact.instructions.md) to understand the design.
    - The Aurora Compact Model exists to save time and tokens by keeping key information in a single compact (thus the name) file.
    - Use the full Aurora instructions only when applying the Architecture skill.
- Outcome-First Planning Focus: Express intended outcomes, checkpoints, and deliverables rather than implementation details.
- Sequencing Focus: Order tasks so prerequisites, dependencies, and review gates are visible before dependent work.
- Verifiable Progress Focus: Make status and deliverables objective enough that completion can be checked without interpretation.
- Technology-Agnostic Planning Focus: Avoid prescribing technologies or implementation details unless the user explicitly requires them.

## Principles of Elegant Planning

- Clarity: Tasks, priorities, and dependencies are understandable without outside interpretation.
- Specificity: Each task has a clear purpose, scope, and completion target.
- Measurability: Progress and completion can be verified from status and deliverables.
- Relevance: Every task contributes directly to project goals or required control gates.
- Sequencing: The plan makes ordering, prerequisites, and review points explicit.
- Minimal Incidental Complexity: The plan coordinates work without prescribing unnecessary implementation detail.

### Indications of Poor Planning

- Vague Tasks.
- Hidden Dependencies.
- Milestone Bundling.
- Implementation Leakage.
- Missing Review Gates.
- Stale Status.

## Deliverables

- The Project Plan at `docs/design/ProjectPlan.md` or a user-designated path.
- A Project Plan containing structured tasks, dependencies, priorities, and progress status.
- If the plan spans design, implementation, release, or verification work, explicit tasks for the applicable review and checklist gates.
- The Project Plan should use the following task format:

```markdown
1. [x] <priority P0-P3>: Description of task
    - Status: `Not Started`, `In Progress`, `Completed`, or `Blocked`
    - Description: Detailed description of the task, its purpose, and any relevant context.
    - Deliverables:
        - Clear, specific, concise deliverables that can be verified upon completion.
    - Cards: (Optional) List of related Aurora card IDs (if applicable)
    - Notes: (Optional) Any assumptions, constraints, or additional information relevant to the task.
    - Dependencies: (Optional) List of other tasks that must be completed before this task can be started.
```

## Operating Procedure

1. Confirm the planning scope, goals, constraints, and target plan path from the user request and any relevant project artifacts. If critical information is missing, stop and get clarification before updating the plan.
2. Inspect the existing plan and related artifacts in `docs/design/` before changing task structure, priorities, or status.
3. Decompose the requested work into tasks with clear priorities, statuses, descriptions, and deliverables, using optional notes, dependencies, and Aurora card references only when they add clarity.
4. Sequence tasks so prerequisites, dependencies, and required review gates appear before the work that depends on them.
5. Keep the plan outcome-focused and technology-agnostic unless the user explicitly requires technology choices or implementation detail in the plan.
6. Update task status to reflect reality, including blocked work and incomplete dependencies, rather than aspirational progress.
7. Validate the result against the deliverables and validation checklists in this skill before considering the plan update complete.

## Validation Checklists

### Plan Structure Checklist

- Correct plan target path.
- Plan format matches the documented structure.
- Consistent numbering and hierarchy.
- Priorities present and within the defined range.
- Status present for every task.

### Task Quality Checklist

- Specific tasks with a clear purpose.
- Measurable deliverables.
- Achievable scope.
- Relevant contribution to project goals.
- Technology-agnostic wording unless the user requires otherwise.

### Dependency and Sequencing Checklist

- Explicit dependencies where they matter.
- No circular dependencies.
- Prerequisites listed before dependents.
- Explicit review and checklist gates where applicable.
- Status that reflects real progress and blockers.

## Things to Watch For

### Planning Foot-Guns

- Vague tasks with no verifiable deliverable.
- Hidden dependencies.
- Overstuffed tasks that bundle unrelated work.
- Review gates missing from delivery plans.
- Implementation details masquerading as project goals.
- Status drift between the plan and reality.

## Cross-skill tasks

- If the request includes code or documentation edits beyond the project plan, split the work into phases and use the appropriate skill for each phase.
- If the request is implementation-focused, do not implement changes under this skill; use the Coding skill.
- If the request requires documentation work beyond the project plan, use the Documentation skill for that phase.
- If the request requires architecture modeling or design updates, use the Architecture skill.
- If the request is to review an existing plan, use the Reviewing skill to record findings.

## Glossary

See the shared [Skills glossary](../GLOSSARY.md).
