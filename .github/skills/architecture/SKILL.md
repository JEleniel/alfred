---
name: architecture
description: Guidelines for modeling architecture using the Aurora model.
---

# Aurora Skill

## Model Overview

**Version**: 2.0.0

Aurora is a deterministic architectural model designed so that any interpretation, such as view diagrams, can be generated from the model; and for direct machine consumption by LLMs, agents, reasoners, and automated tools. The model invariants guarantee unambiguous interpretation and reasoning about the model.

Semantics are derived from the invariant rules: relationship verbs are descriptive only, and meaning comes from interpretation (views, impact analysis, traceability).

**One Goal**: Enable the Architect to focus on modeling the architecture instead of drawing diagrams and pictures.

## Models

The model is the central piece of the architecture and is a collection of cards representing the architectural elements that have links describing their relationships, starting from a `Mission`.

Any pair of cards in the model can be described using simple sentences:

**Examples**:

```text
The mission "Drive Excellence" is "Drive excellence in operations by streamlining processes, integrating automation, and formalizing documentation".

The mission establishes the driver "Operational Friction Elimination" which is "Eliminate non-value-adding manual effort by enforcing end-to-end process automation, standardized workflows, and machine-verifiable documentation across all operational domains".

"Operational Friction Elimination" drives the requirement "Define a Deterministic Modeling Framework" which is "Design a framework for documenting deterministic process models with measurable latency and failure semantics".
```

**These result in a model that looks like this**:

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
  drive_excellence(("`Mission:<br />Drive Excellence`"))
  operational_friction_elimination(["`Driver:<br />Operational Friction Elimination`"])
  define_a_deterministic_modeling_framework(["`Requirement:<br />Define a Deterministic Modeling Framework`"])

  drive_excellence -- establishes --> operational_friction_elimination
  operational_friction_elimination -- drives --> define_a_deterministic_modeling_framework
```

### Cards

Cards represent the elements of the design, described as nouns, and contain the attributes of the element and the links to other elements. Cards have an `attributes` property that allows additional, arbitrary information to be included. Card files should be "pretty printed" using `prettier` or similar.

Each card is comprised of:

| Field          | Required | Meaning                                                                                                                                                                                          |
| -------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `$schema`      | Yes      | Relative link to the Aurora schema file in the model home.                                                                                                                                       |
| `id`           | Yes      | Semi-permanent unique identifier with a `card_type` prefix and sequential (to the model) integer. If the `card_type` changes a new `id` must be issued.                                          |
| `card_type`    | Yes      | Architectural element represented by the card in title case.                                                                                                                                     |
| `card_subtype` | No       | Optional refinement of the `card_type` in title case.                                                                                                                                            |
| `name`         | Yes      | Concise human-readable name for the card in title case.                                                                                                                                          |
| `description`  | Yes      | Details regarding the element the card represents.                                                                                                                                               |
| `status`       | No       | Status of an implementable element such as a `Feature`. Recommended lifecycle: "Proposed", "Design", "Implementation", "Released", "Deprecated", "Deleted"; extend consistently per `card_type`. |
| `links`        | Yes      | Pointers to other cards establishing relationships (`target` is the destination card `id`; `relationship` is a human-readable verb).                                                             |
| `version`      | Yes      | Semantic version of the card, starting from 1.0.0; increment major for changes in meaning, minor for changes like rephrasing, and patch for corrections and minor changes                        |
| `boundary`     | No       | Optional grouping label for rendering and organization.                                                                                                                                          |
| `notes`        | No       | Additional notes and information attached to the card.                                                                                                                                           |
| `attributes`   | No       | Arbitrary optional key-value pairs providing additional data; the value can be any valid JSON value, including objects.                                                                          |

**Example `id`s**:

- MIS-001
- DRI-001
- DRI-002

#### Canonical Cards and Relationships

A canonical set of cards and relationships is included. The canonical set is designed to cover all normal architectural elements and ensure that the relationships conform to the invariants. Aurora is designed to be easily extended, so models are not limited to the canonical set.

- [Aurora Model Configuration](../../aurora/reference/Aurora.modelconfiguration.json)

##### Registry format (canonical definitions)

The canonical definitions registry is a JSON object with:

- `definitions`: a JSON array of entries. Each entry defines one card type.

The file MAY include `$schema` for tooling.

- `card_type`: the human-facing type name (title case, for example `Mission`, `Data Store`).
- `acronym`: the short id prefix used in compact ids and in relationship targets (for example `MIS`, `DST`).
- `relationships`: optional array of allowed outgoing relationship targets for this card type.
    - Each item is a single-key object mapping a target acronym to a human-readable verb (for example `{ "DST": "persists to" }`).

This registry is normative for (a) which card types exist and (b) which outgoing link targets/verbs are valid for each type.

##### Normative vs rendering fields

The canonical registry contains both **normative** semantics (used to validate and reason about models) and **rendering/style** fields (used to draw consistent diagrams).

- Normative for model meaning and validation:
    - `card_type`, `acronym`, and `relationships`.
- Rendering/style hints (non-normative for model validity):
    - `shape`, `icon`, `fill`, `color`, `common_subtypes`.

Rendering/style fields are still part of the canonical registry for consistency and theming, but a model MUST NOT be considered invalid because of stylistic choices.

### Audit Log Entries

The audit log contains one top level property, `history`, which is an array of audit entries:

```json
{
    "$schema": "../Aurora.audit.schema.json",
    "history": [
        {
            "timestamp": "<ISO 8601 UTC timestamp>",
            "editor": "<name or id of editor>",
            "target": "<card id changed>",
            "change_type": "<create|change|delete>"
        }
    ]
}
```

### File and Folder Structure

Models live in a folder named `aurora/`, the model home. Multiple models may share a model home. The `Mission` card is stored in the model home, and all other cards are stored in a model and card type specific folder, `{mission id}/{card type}/` inside the model home.

A model is composed of up to four kinds of files:

1. Schemas: `Aurora.card.schema.json`, `Aurora.audit.schema.json`, and `Aurora.compact.schema.json` files in the model.
    - These schemas are shared by all models, audit logs, and compact models in the same model home.
    - If these files are not present, or when creating a new model home, copy them from `.github/aurora/` to the model home before creating any other files.

    - All JSON files MUST have the `$schema` attribute with the relative path from that file to the appropriate schema in the model home.

2. Cards: the central component of the model, each card is stored in a separate JSON file named `{id}-{name}.json` where name has had all symbols removed and spaces replaced with underscores. All cards must conform to the `Aurora.card.schema.json` in the model home.
3. Audit Log: a history of who made changes to which cards over time, stored at `{model id}/AuditLog.json` and conforming to the `Aurora.audit.schema.json` in the model home. An audit log entry MUST be made for every card change to any part of the model.
4. Compact Model: an optional compact, single file version of the model at `{mission id}/Compact.json` and conforming to the `Aurora.compact.schema.json` in the model home.

- The model home may be stored as a ZIP file for transport if the folder structure is preserved.

**Example Folder and File Structure**:

```text
aurora
  ├─ Aurora.audit.schema.json
  ├─ Aurora.card.schema.json
  ├─ Aurora.compact.schema.json
  ├─ MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json
  ├─ MIS-002-Write_User_Documentation_for_Aurora.json
  ├─ MIS-001
  │    ├─ AuditLog.json
  │    ├─ Compact.json
  │    ├─ Driver
  │    │    ├─ DRI-001-Some_Reason.json
  │    │    └─ DRI-002-Another_Reason.json
  │    └─ Requirement
  │        └─ REQ-001-Something_Has_To_Happen.json
  ├─ MIS-002
  │    ├─ Driver
... etc
```

### Invariant Rules

These invariant rules ensure that the model is a traversable, directed graph with local cycles only (cycles that do not reach a view root).

1. **The `Mission` Card**: All models must start with and include a single `Mission` card that summarizes the high-level "why" of the project. The `Mission` card must only have outgoing links, and serves as the root of the directed graph.

2. **Direction (graph links)**: When traversing starting from Mission, all links must lead away from the `Mission` card. Traversing any path starting from `Mission` must end either in a leaf card or a previously seen card, creating a local loop.

3. **No orphans**: Other than the `Mission` card, all cards must have one or more incoming links, and a path from the `Mission` card. All cards may have any number of outgoing links. All link targets must be valid cards in the model.

## Views

Views are generated by selecting a set of card types (and optionally subtypes) as roots for local graphs, selecting a set of other card types (and optionally subtypes) to include when traversing from the roots, and rendering a diagram showing those cards and the links between them. Views **do not change the model**; they change what part of and how the model is viewed. One model, many views.

A set of common views can be found in the Aurora model configuration registry (`reference/Aurora.modelconfiguration.json`).

### Registry format (view definitions)

The view definitions registry is a JSON array of view objects:

- `name`: view name.
- `description`: view intent.
- `root_card_types`: card types that define candidate roots for this view.
- `included_card_types`: card types that are eligible to be rendered when reachable from a root.

### View roots and traversal

- A view **root** is a card that cannot participate in a loop; roots delineate specific subgraphs (for example, a `State Machine` view).
- **Root safety rule**: `root_card_types` MUST only include card types whose selected root cards are not part of any cycle in the model graph.
    - If selecting roots by type yields a candidate root card that participates in a cycle, it MUST be excluded from the root set for that view.
- Traversal always starts from each root and (by the invariants) continues away until either reaching a leaf or closing a local loop.
- Links are not filtered for general traversal.
- When rendering a view, root cards are always rendered.
- During traversal, a non-root card is eligible to be rendered if its `card_type` is in either `included_card_types` or `root_card_types`.
    - Nothing outside the rendered cards (and the links between them) is rendered.

## Default Tooling

### `aurora_cli`

- Validates models
- Generates human readable Markdown copies
- Generates Markdown files containing views
- Generates the compact model files

## Rendering

This section defines rendering expectations for cards and views.

- Shapes are geometry-only and MUST NOT encode colors or fills.
    - Shapes can be anything that can be modeled with SVG.
    - Shapes SHOULD provide adequate clearance for centered text and SHOULD keep a 1.6:1 proportion.
        - Default node dimensions are 160w x 100h.
- `icon` can be any Unicode symbol.
- Colors and fills come from style fields (for example `fill` and `color`) and are applied by the renderer.
- `boundary` is typically rendered as a dashed box with the boundary name.
- `notes` are typically rendered as a callout attached to the node.
