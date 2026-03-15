---
applyTo: 'docs/design/aurora/**/Compact.json'
---

# Aurora Compact Model Instructions (Read-Only)

This instruction defines what is needed to read and interpret Aurora compact model exports.

## Canonical split: schema vs instruction

- Structure is canonical in `./schemas/Aurora.compact.schema.json`.
- This instruction is interpretation guidance only.

## What these files are

The Compact Model is a copy of all of the Aurora cards stripped of extraneous data and compiled into a single, token-friendly file. It contains all the information needed for implementation. Use the compact model when _not_ modifying the model, e.g., when implementing the code or documentation. The Compact Model is the preferred read-only source of access to the model; it is a derived artifact and is not the source-of-truth.

## Invariants

- You MUST NOT edit any `docs/design/aurora/<MISSION_ID>/Compact.json` compact export.

## Consumption rules

- Parse compact exports using the compact schema only.
- The compact model's `$schema` is a relative path to the schema used to create and validate it. There is no need to load or validate against this schema because the tooling that generates the compact model already validated everything.
- Compact cards MAY include `boundary` because it can affect logical reasoning about grouping semantics.
- The `attributes` property, if present, preserves implementation details from the source card.
- Relationship semantics remain canonical.

## Aurora Invariant Rules and Interpretation

Any Aurora model satisfies three invariants. These invariant rules ensure that the model is a rooted directed graph with only local recurrence.

1. **The `Mission` Card**: All models must start with and include a single `Mission` card that summarizes the high-level "why" of the project. The `Mission` card must only have outgoing links and serves as the root node of the directed graph.
2. **Direction (graph links)**: Traversal follows directed edges from `Mission` outward. Traversal algorithms MUST halt when they encounter either a leaf node (out-degree `0`) or a previously visited node.
3. **No orphans**: Other than the `Mission` card, all cards must have one or more incoming links and a path from the `Mission` card. All cards may have any number of outgoing links. All link targets must be valid cards in the model.

### Interpretation

- Every non-`Mission` card is reachable from `Mission` by at least one directed path.
- While local loops may appear, they easily decompose into DAGs. Traversal algorithms halt when they encounter either a leaf node (out-degree `0`) or a previously visited node.
- Relationship verbs are descriptive labels; validity comes from the links and traversal.
- Traversal may treat any of the following card types as local roots of a DAG subgraph of the model:
    - Mission, System, Application, Vendor, Deployment, Capability, Threat Model, State Machine, Stakeholder, Actor, Process
- The audit log is not needed to understand the compact model.
- In order to extract all details for an Application, start at the APP card and traverse out. This skips having to scan the entire model.
