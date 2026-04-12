# Design Authority Matrix

This file defines One Source of Truth ownership for Alfred design concerns. If any document conflicts with this matrix, this matrix wins until the conflict is resolved at the authoritative source.

## Canonical Ownership

| Concern | Canonical Source | Notes |
| --- | --- | --- |
| Product intent and mission scope | `docs/design/ProjectSummary.md` | Scope, goals, non-goals, and high-level constraints. |
| Functional and non-functional requirements | `docs/design/AlfredOverview.md` | Requirement-level behavior, execution semantics, and SLOs. |
| Architecture and decomposition intent | `docs/design/AlfredArchitecture.md` | Logical architecture, boundaries, and cross-cutting controls. |
| Wire protocol framing and tool envelope | `docs/design/Protocol.md` | UTF-8 framing, envelope shape, streaming semantics, path representation. |
| Public tool contracts | `docs/design/ToolContracts.md` | Tool names, operations, input/output schema, ordering, and limits. |
| Error taxonomy and deterministic details | `docs/design/ErrorTaxonomy.md` | Canonical error kinds and `details.reason` conventions. |
| Configuration semantics and precedence | `docs/design/Configuration.md` | Merge semantics, precedence, location controls, and policy gates. |
| Default values | `docs/design/Defaults.md` | Canonical defaults only. No other document may redefine defaults. |
| Storage and on-disk layout | `docs/design/StorageLayout.md` | Canonical file and folder locations across storage modes. |
| Deterministic redaction behavior | `docs/design/Redaction.md` | Detection, span merge, token fitting, and warnings behavior. |
| Structural quality and engineering policy | `docs/design/QualityPolicy.md` | Canonical maintainability and decomposition constraints. |
| Execution tracking only | `docs/design/ProjectPlan.md` | Work tracking; not authoritative for API/contract definitions. |
| Formal findings and evidence | `docs/design/analysis/` and `docs/design/Review-*.md` | Evidence sources; do not override canonical contracts directly. |

## Conflict Resolution Rules

1. Resolve contradictions at the canonical source listed in this file.
2. Cross-reference from non-canonical files to canonical sources rather than duplicating normative rules.
3. If a policy applies to multiple concerns, define it once and reference it.
4. Deprecation, aliasing, and compatibility behavior must be explicitly documented at the canonical source; implicit behavior is non-compliant.

## Integration Policy

All findings from formal analysis, code review, documentation review, security review, and fix tracking must be represented in canonical docs by one of these methods:

- Direct normative update at the canonical source.
- Explicit deferred decision entry in `docs/design/ProjectPlan.md` with a link to the canonical target.

No standalone backlog artifact may remain as a competing source of design truth.
