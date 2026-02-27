# Aurora Machine Agent Instruction

## Instruction precedence (Aurora models)

When working with Aurora model artifacts (for example files under `docs/design/aurora/`, a model home `aurora/`, or the schema/reference material that supports them), this document's Aurora-specific rules override any general repository instructions where they conflict.

Examples:

- Aurora model JSON files require `$schema` to be a relative path into the model home, even if general JSON guidance prefers a URL.
- Model-home schema/reference locations and invariants defined here are authoritative for Aurora modeling tasks.

## Model Overview

Aurora is a deterministic, typed, directed graph rooted at a single `Mission` card. Cards are nodes, and links are constrained edges. Meaning comes from graph structure and allowed link types, not from diagram shapes or wording. Views are read-only projections of the model and never modify it. The model is a pure architecture which represents logical architecture and intent, not runtime instances, operational state, or implementation tracking.

**One Goal**: Enable the Architect to focus on modeling the architecture instead of drawing diagrams and pictures.

## Canonical split: schema vs instruction

- Schemas are canonical for structure and field constraints.
- Instructions are canonical for behavior and process rules.
- Examples are illustrative and non-canonical.

## Models

A model is the central artifact in Aurora. It is a collection of cards that represent architectural elements, connected by links that describe their relationships, starting from a `Mission`.

Each model is identified by its `Mission` ID.

### Getting started

1. Create or locate the model home folder: `aurora/` (located at `docs/design/aurora/` by default).
2. Ensure `aurora/schemas/` contains `Aurora.audit.schema.json`, `Aurora.card.schema.json`, `Aurora.compact.schema.json`, `Aurora.modelconfiguration.schema.json`, and `Aurora.viewconfiguration.schema.json`. If missing, copy them from `.github/aurora/schemas/`. Do not copy the Markdown files.
3. Ensure `aurora/reference/` contains `Aurora.modelconfiguration.json`, `Aurora.viewconfiguration.json`, and `SVGTemplate.svgz`. If missing, copy them from `.github/aurora/reference/`. Do not copy the Markdown files.
4. Create the `Mission` card in the model home (`aurora/`).
5. Add other cards under `{mission id}/{card type folder}/` and link them from existing cards.
6. Append to `{mission id}/AuditLog.ndjson` for every change event. One entry may include changes to multiple cards.

At the end of making changes, validate the model(s), generate the Markdown, views, and compact model. **If this fails do not stop working.**

### How the Model Works

Any pair of cards in the model can be described using simple sentences:

**Examples**:

```text
The mission "Drive Excellence" is "Drive excellence in operations by streamlining processes, integrating automation, and formalizing documentation".

The mission establishes the driver "Operational Friction Elimination" which is "Eliminate non-value-adding manual effort by enforcing end-to-end process automation, standardized workflows, and machine-verifiable documentation across all operational domains".

"Operational Friction Elimination" drives the requirement "Define a Deterministic Modeling Framework" which is "Design a framework for documenting deterministic process models with measurable latency and failure semantics".
```

**These sentences produce a model that looks like this**:

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
  drive_excellence(("`Mission:<br />Drive Excellence`"))
  operational_friction_elimination(["`Driver:<br />Operational Friction Elimination`"])

  drive_excellence -- establishes --> operational_friction_elimination
```

### Cards

Cards represent architectural elements (nouns). A card contains properties of the element and links to other elements. A card MAY include an optional `icon` property to override the default icon mapping when rendering views. An `attributes` object is included to capture properties that are not already represented by other fields. Use links for relationships and interactions between elements. Card files should be "pretty printed" using `prettier` or a similar tool.

Card field structure is defined exclusively in `schemas/Aurora.card.schema.json`.

When present, the optional `icon` value MUST match an icon id in `reference/Aurora.viewconfiguration.json` (`available_icons`).

**Example `id`s**:

- MIS-001
- DRI-001
- DRI-002

#### External References

In order to allow for the inclusion of artifacts, such as JSON schemas, ADR documents, URLs and other external material, the `external_references` field can contain an array of strings, each of which is a _relative_ path or URL to the external material. When rendered to Markdown these will become links.

The default location for such material is `docs/design/references/`.

#### ID acronym consistency

For canonical card types, a card ID prefix MUST match the canonical card-type acronym (for example, `REQ-001` for `Requirement`, `DST-001` for `Data Store`). For non-canonical card types, define one stable three-letter acronym and use it consistently.

#### Canonical Cards

A canonical set of cards and relationships is included. The canonical set is designed to cover all normal architectural elements and ensure that the relationships conform to the invariants. The canonical set also includes common subtypes for convenience. Aurora is designed to be easily extended, so models are not limited to the canonical set.

Extension rule (minimal): use canonical cards and relationships by default. Add non-canonical types only when no canonical option is semantically correct, and preserve all graph invariants.

Canonical card types, relationships, and view definitions are defined in the model configuration registry:

- [Aurora Model Configuration](reference/Aurora.modelconfiguration.json)
- [Aurora Model Configuration Schema](schemas/Aurora.modelconfiguration.schema.json)

Canonical view appearance defaults, which are only used by the rendering tools, (including `available_icons`) are defined in the view configuration registry:

- [Aurora View Configuration](reference/Aurora.viewconfiguration.json)
- [Aurora View Configuration Schema](schemas/Aurora.viewconfiguration.schema.json)

### File and Folder Structure

Models live in a folder named `aurora/` (the model home). If an `aurora/` folder already exists at the target location, use it. Never create an `aurora` folder inside another `aurora` folder. Multiple models may share one model home. The `Mission` card is stored in the model home, and all other cards are stored in mission- and card-type-specific folders: `{mission id}/{card type folder}/`. The `{card type folder}` value MUST use card type sanitization: remove symbols and replace spaces with underscores (for example, `Data Store` -> `Data_Store`, `State Machine` -> `State_Machine`, `Threat Diamond` -> `Threat_Diamond`).

A model may include six kinds of files:

1. Schemas: These are used to validate the JSON and NDJSON files of models at load and when validating. See the list of schemas under [Getting Started](#getting-started).
    - These schemas are shared by all models, audit logs, and compact models in the same model home.
    - All model JSON files MUST have the `$schema` attribute with the relative path from that file to the appropriate schema. NDJSON does not use the `$schema` property.
2. References: These files are used by the tooling to ensure that models are handled according to the definitions in place at the time of their creation. See the list of reference files under [Getting Started](#getting-started).
3. Cards: the central component of the model, each card is stored in a separate JSON file named `{id}-{name}.json` where name has had all symbols removed and spaces replaced with underscores. Card-type folder names use the same sanitization behavior.
4. Audit Log: an append-only, line-delimited JSON history of change events over time, stored at `{mission id}/AuditLog.ndjson`. Each line MUST conform to `Aurora.audit.schema.json` and may capture multiple changed cards in one entry. An audit log entry MUST be appended for every change event.
5. Compact Model (generated): an optional compact, single file version of the model at `{mission id}/Compact.json` and conforming to the `Aurora.compact.schema.json` in the model home.
6. Markdown renderings (generated): Parallel to the `aurora/` folder Markdown and SVG files that are generated by the tooling may exist.

The model home may be stored as a ZIP file for transport if the folder structure is preserved.

**Example Folder and File Structure**:

```text
aurora
  ├─ MIS-001
  │    ├─ Driver
  │    │    ├─ DRI-001-Some_Reason.json
  │    │    └─ DRI-002-Another_Reason.json
  │    ├─ Requirement
  │        └─ REQ-001-Something_Has_To_Happen.json
  │    ├─ AuditLog.ndjson
  │    └─ Compact.json
  ├─ MIS-002
  │    ├─ Driver
  │    └─ ...
  ├─ schemas
  │    ├─ Aurora.audit.schema.json
  │    ├─ Aurora.card.schema.json
  │    ├─ Aurora.compact.schema.json
  │    ├─ Aurora.modelconfiguration.schema.json
  │    └─ Aurora.viewconfiguration.schema.json
  ├─ reference
  │    ├─ SVGTemplate.svg
  │    ├─ Aurora.modelconfiguration.json
  │    └─ Aurora.viewconfiguration.json
  ├─ MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json
  └─ MIS-002-Write_User_Documentation_for_Aurora.json
... etc
```

### Invariant Rules

These invariant rules ensure that the model is a rooted directed graph with only local recurrence.

1. **The `Mission` Card**: All models must start with and include a single `Mission` card that summarizes the high-level "why" of the project. The `Mission` card must only have outgoing links and serves as the root node of the directed graph.
2. **Direction (graph links)**: Traversal follows directed edges from `Mission` outward. Traversal algorithms MUST halt when they encounter either a leaf node (out-degree `0`) or a previously visited node.
3. **No orphans**: Other than the `Mission` card, all cards must have one or more incoming links and a path from the `Mission` card. All cards may have any number of outgoing links. All link targets must be valid cards in the model.

## Optimizations for Handling Models

- Unless making changes to the model, the compact model should cover everything an agent needs to implement.
- Treat the mission audit log (`{mission id}/AuditLog.ndjson`) as the primary “what changed” record.
- In order to extract all details for an Application, start at the APP card and work down as if rendering a view. This skips having to scan the entire model
- The audit log is append-only NDJSON. In most workflows, appending a new entry is sufficient; avoid reading the entire file unless required.
- Many Aurora workflows regenerate large, mechanical outputs (for example views, markdown renderings, and compact exports). These changes can overwhelm `git diff` and obscure intent.
- The `Aurora.viewconfiguration.json` and `Aurora.viewconfiguration.schema.json` do not need to be read by agents; they are only needed by the rendering tools.
- The cards are easy to locate and self-indexing, so there is generally no need to keep more than three cards in active memory.
- It is better to run `aurora_cli validate` than to manually load the schemas to validate. Only load schemas when necessary.
