---
name: planning
description: Instructions for creating and maintaining a project plan, including task decomposition, sequencing, and progress tracking. This skill focuses on high-level planning and should not include direct code generation or modification.
---

# Planning

## When to use this skill

Use this skill when the task involves:

- Creating or updating a project plan.
- Decomposing a high-level goal into actionable tasks.
- Sequencing tasks with dependencies and estimated effort.

This skill must **never** generate, modify, or suggest changes to source code or documentation other than the project plan.

## Planning Principles

- **Specific**: Tasks should be clearly defined with specific deliverables.
- **Measurable**: Progress should be trackable through the status and deliverables.
- **Achievable**: Tasks should be realistic and achievable within the context of the project.
- **Relevant**: Tasks should directly contribute to the overall project goals.
- **Technology Agnostic**: The plan should not prescribe specific technologies or implementation details, but rather focus on the high-level objectives and outcomes.

## Deliverables

- The Project Plan at `docs/design/ProjectPlan.md` or a user designated path.
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
