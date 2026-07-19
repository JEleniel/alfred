---
name: analysis
description: 'Use this skill when asked to analyze (not review) code structure, maintainability, coupling, complexity, call chains, module boundaries, or to produce a formal call map. Use it for analysis, not line-level review.'
---

# Code Analysis Skill

## Boundaries

- Base findings on observable code structure, dependency relationships, and verified behavior.
- Confirm the scope early and keep the depth of analysis proportional to that scope.
- Analyze how modules, layers, functions, and data flow interact. Do not treat structural problems as isolated local defects.
- Prefer objective indicators such as dependency fan-in and fan-out, public API breadth, function length outliers, nesting depth, parameter count outliers, and pass-through wrapper frequency.
- Recommend concrete structural or behavioral changes that improve correctness, clarity, maintainability, or long-term evolvability.
- Perform structural analysis, not line-level review, defect triage, or implementation work.
- Capture the call map mechanically. Record direct calls, uses, and dependencies without turning the call map into analysis.
- Keep intent clear, complexity bounded, behavior local, decomposition appropriate, and similar responsibilities in similar patterns.

## What to Analyze

- Check module and directory breadth versus depth, excessive nesting, flat structures that hide boundaries, cycles, mixed responsibilities, and dependency hubs.
- Prefer clear ownership, limited coupling, predictable hierarchy, contained state, and one source of truth.
- Check dependency fan-in and fan-out, tight cross-layer coupling, hidden transitive dependencies, cross-domain imports, and overuse of shared utilities.
- Flag fragile graphs, architecture violations, and hard refactoring boundaries.
- Check function length, cognitive complexity, parameter count, mixed responsibilities, nesting, and hidden state mutation.
- Prefer one conceptual task, predictable control flow, and minimal hidden side effects.
- Check call-chain length, pass-through layers, deeply nested delegation, context propagation, and error propagation complexity.
- Exclude deliberate recursion unless it creates unexpected complexity or coupling.
- Treat unintentional recursion in a call chain as a structural issue.
- Check mutable shared state, cross-layer mutation, inconsistent ownership, excessive parameter passing, and hidden state transitions.
- Prefer clear ownership, minimal mutation boundaries, and predictable transformations.
- Flag abstractions that expose internals, add little value, expose broader interfaces than needed, or obscure domain meaning.
- Prefer abstractions that reduce cognitive load, hide implementation details, and provide domain-relevant operations.

## Findings and Recommendations

- Flag god modules, oversized files or functions, utility hubs, excessive wrappers, indirection without value, deep hierarchies with thin value, long call chains with little logic per layer, and overly broad public APIs.
- Recommend flattening or restructuring hierarchies, extracting cohesive modules, reducing call-chain depth, consolidating redundant abstractions, encapsulating mutable state, narrowing public interfaces, simplifying control flow, and removing pass-through layers.
- Prioritize lower cognitive complexity, shorter reasoning paths, clearer boundaries, safer state management, and better long-term evolvability.

## Files and Folders to Ignore

- Ignore generated artifacts, build output, vendored code, derived documentation, and cache or tool output directories unless the user explicitly includes them in scope. Treat `docs/design/analysis/` as output to replace, not evidence to inherit. Read `.github/` only when the user explicitly includes repository customization or workflow assets in scope.

## Call Map and Call Graph

Record direct calls, uses (such as constants, types, or other non-function items), and dependencies between components (for example, structs). Keep the call map mechanical. Do not interpret or analyze while capturing it.

- Do not capture low-level internal details such as formulas and local variables.
- List each call pair only once, no matter how many times A calls B.
- Do not capture dependencies between _files_ or modules. Capture only relationships between functions, types, constants, and other code items.

Build the Call Graph as a Mermaid flowchart directly and mechanically from the Call Map. Do not optimize it. Orient it left to right, make modules subgraphs, place code items in their module subgraph, and point every caller to the one callee node. Ignore external dependencies. Use solid edges for calls, dotted edges for uses, thick edges for dependencies, keep all edges directed, and quote all display text.

## Deliverables

- Replace any existing `docs/design/analysis/Code.md` and `docs/design/analysis/CallMap.md` for the current scope. Do not merge with prior analysis unless the user explicitly asks for that.
- Produce `docs/design/analysis/Code.md` with structural weaknesses, complexity indicators, risks, simplification opportunities, recommendations, and lessons learned.
- Create a Call Map at `docs/design/analysis/CallMap.md` that identifies components, their functions, and what their functions call, use, and depend on.
- Generate the Call Graph at `docs/design/analysis/CallGraph.md` from the Call Map.

## Operating Procedure

1. Confirm scope, constraints, and success criteria. If critical scope information is missing, stop and get clarification.
2. Inspect the module hierarchy, file layout, and dependency structure needed for the requested scope.
3. Build the call map as a direct capture of calls, uses, and dependencies.
4. Analyze structure, complexity, call chains, state flow, and abstraction quality using observable evidence.
5. Identify structural weaknesses and propose the smallest safe structural improvements.
6. Replace `docs/design/analysis/Code.md` and `docs/design/analysis/CallMap.md` with the current results for the requested scope.
7. Validate the result against the deliverables and checklists in this skill.

Use `./.github/violations.sh` if present to identify oversized files and functions, as well as small functions that may be opportunities.

## Validation Checklists

- Structure, functions, call chains, state flow, and abstractions are evaluated clearly and with evidence.
- `docs/design/analysis/Code.md` and `docs/design/analysis/CallMap.md` are replaced for the current scope, not incrementally merged.
- Findings stay scoped and structural.
- The call map stays descriptive and does not become a second analysis document.

## Cross-skill Tasks

- Use the Reviewing skill for review requests, the Architecture skill for architectural issues, and the Coding skill after analysis when the task includes implementation.
