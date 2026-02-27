---
name: planning
description: Instructions for creating and maintaining a project plan, including task decomposition, sequencing, and progress tracking. This skill focuses on high-level planning and should not include direct code generation or modification.
---

# Planning Skill

## When to use

Use this skill when the task involves:

- Creating or updating a project plan.
- Decomposing a high-level goal into actionable tasks.
- Sequencing tasks with dependencies and estimated effort.

This skill must **never** generate, modify, or suggest changes to source code or documentation other than the project plan.

## Planning principles

- If a `docs/design/aurora/` folder exists, read and follow [Aurora Compact Model](../../aurora/Aurora.compact.instructions.md) to understand the design.
    - The Aurora Compact Model exists to save time and tokens by keeping key information in a single compact (thus the name) file.
    - Use the full Aurora instructions only when applying the Architecture skill.
- **Specific**: Tasks should be clearly defined with specific deliverables.
- **Measurable**: Progress should be trackable through the status and deliverables.
- **Achievable**: Tasks should be realistic and achievable within the context of the project.
- **Relevant**: Tasks should directly contribute to the overall project goals.
- **Technology Agnostic**: The plan should not prescribe specific technologies or implementation details, but rather focus on the high-level objectives and outcomes.

## Deliverables

- The Project Plan at `docs/design/ProjectPlan.md` or a user-designated path.
- The Project Plan is a list of structured tasks, dependencies, and progress status in the following format:

```markdown
1. [x] Description of task
    - Priority: 0 (Critical) to 3 (Low)
    - Cards: List of related Aurora card IDs (if applicable)
    - Description: Detailed description of the task, its purpose, and any relevant context.
    - Deliverables:
        - Clear, specific, concise deliverables that can be verified upon completion.
    - Notes: Any assumptions, constraints, or additional information relevant to the task.
    - Status: `Not Started`, `In Progress`, `Completed`, or `Blocked`
    - Dependencies: (Optional) List of other tasks that must be completed before this task can be started.
```

## Cross-skill tasks

- If the request includes code or documentation edits beyond the project plan, split the work into phases and use the appropriate skill for each phase.
- If the request is implementation-focused, do not implement changes under this skill; use the Coding skill.
- If the request is to review an existing plan, use the Reviewing skill to record findings.

## Validation

- The plan file is the correct target (`docs/design/ProjectPlan.md` unless a different path is specified).
- Every task has clear deliverables and a status that can be verified.
- Dependencies are consistent (no circular dependencies; prerequisites are listed before dependents).
- Priorities are present and within the defined range.
- The plan format matches the documented structure.

## Glossary

See the shared [Skills glossary](../GLOSSARY.md).
