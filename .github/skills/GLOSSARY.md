# Skills glossary

This glossary defines shared terms used by the skill files under `.github/skills/`.

## Aurora modeling references

| Term                              | Meaning                                                                                                                                                                                                                                                                     |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Aurora instructions               | The Aurora instruction set at `aurora/Aurora.instructions.md`. This is required when using the Architecture skill.                                                                                                                                                          |
| Aurora Compact Model instructions | The compact reference at `aurora/Aurora.compact.instructions.md`. It exists to save time and tokens by keeping key information in a single compact (thus the name) file. Prefer this for non-architecture skills unless a task explicitly requires the Aurora instructions. |
| Compact model                     | An Aurora model representation intended to carry the essential design information in a compact form suitable for fast reading and lower context usage.                                                                                                                      |

## Review terminology

| Term           | Meaning                                                                                                                                                                                                    |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Feedback       | Informal commentary in a response. It is not a formal review artifact unless the task explicitly requires recording it.                                                                                    |
| Review         | A formal set of findings and recommendations that is recorded in the review output files specified by the Reviewing skill. Reviews recommend improvements and fixes; they do not make or record decisions. |
| Finding        | A concrete observation about correctness, security, reliability, maintainability, documentation quality, or gaps, typically paired with a rationale and a smallest-safe recommendation.                    |
| Recommendation | A proposed improvement or fix. A recommendation is not a decision and does not change the codebase unless a separate implementation task is authorized.                                                    |

## Cross-skill tasks

| Term             | Meaning                                                                                                                                                                                                                                                                                                        |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cross-skill task | A request that spans multiple skills (for example: planning + implementation, review + fixes, architecture + build-out). The safe default is to split the work into phases and apply the strictest constraints for the current phase.                                                                          |
| Phase            | A bounded portion of a cross-skill task that produces a single kind of deliverable (plan, model, review artifact, code change, documentation change). Typical flow (example): Architect -> Plan -> Design Review -> Implement -> Code & Security Review -> Document -> Documentation Review -> Release Review. |

## Normative keywords

| Keyword             | Meaning                                                    |
| ------------------- | ---------------------------------------------------------- |
| MUST / MUST NOT     | Absolute requirements.                                     |
| SHOULD / SHOULD NOT | Strong guidance; deviations require a clear justification. |
| MAY                 | Optional behavior.                                         |
