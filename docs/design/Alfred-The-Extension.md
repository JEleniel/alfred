## Alfred (VS Code-integrated LLM mediation layer) — Requirements / Features

---

### 1. Chat Interface Replacement

* Provide a native VS Code extension chat panel (Webview or sidebar)
* Accept all user prompts through Alfred UI (no direct Copilot Chat usage in workflow)
* Support structured task entry (not free-form only), including:

  * intent (implement / refactor / explain / analyze)
  * target scope (file / function / symbol / selection)

---

### 2. Deterministic Context Compiler

* Build a context assembly system that:

  * resolves symbols from workspace index (not model-driven exploration)
  * extracts minimal required code slices (not full files by default)
  * includes only:

    * function signatures
    * relevant types/traits
    * explicitly selected dependencies
* Replace static “base prompt” with task-scoped rule injection
* Produce a final structured “model input packet”

---

### 3. Workspace Indexing System

* Maintain a local full-text index of the codebase
* Add AST-based indexing layer (Rust-aware preferred)
* Provide fast symbol-level queries:

  * definitions
  * references
  * call relationships (coarse-grained acceptable)

---

### 4. Controlled Retrieval Layer

* All code retrieval must pass through Alfred
* Model is not allowed to:

  * enumerate repository
  * request file exploration
* Retrieval outputs are:

  * pre-filtered
  * minimal
  * structured

---

### 5. Scope Enforcement (Hard Boundaries)

* Define explicit edit envelopes per task:

  * file-level or function-level scope
  * allowed symbol set
* Reject or rewrite model inputs/outputs that:

  * introduce new modules or public symbols (unless explicitly allowed)
  * modify outside designated scope
  * expand task beyond requested boundaries

---

### 6. Diff-Only Output System

* Model outputs must be restricted to:

  * patch/diff format only
* Alfred validates:

  * structural correctness
  * scope compliance
  * duplication and redundancy detection
* Invalid outputs are rejected before reaching editor

---

### 7. Structural Validation Engine

* Integrate rule-based checks (AST-aware where possible):

  * error handling rules (no swallowed results, no silent failures)
  * helper-function explosion detection
  * function size/complexity constraints
  * forbidden patterns (project-specific)
* Enforcement must be deterministic (not LLM-based)

---

### 8. Model Memory Replacement

* No reliance on Copilot or model “memory”
* Replace with deterministic workspace state:

  * symbol graph
  * file structure map
  * project invariants registry
* Reconstructed context per request, not persisted model state

---

### 9. Model Abstraction Layer

* Alfred acts as:

  * prompt compiler
  * retrieval engine
  * constraint enforcer
  * output validator
* LLM is treated strictly as:

  * stateless code transformation engine

---

### 10. Model Routing Flexibility (optional)

* Support interchangeable backend models:

  * Copilot (if accessible)
  * direct API-based models
* Backend must be abstracted behind Alfred
* No dependency on Copilot chat pipeline behavior

---

## Summary (condensed)

* Replace VS Code chat interface with Alfred-controlled panel
* Centralize all prompt construction via context compiler
* Enforce deterministic codebase indexing (text + AST)
* Restrict model to scoped, pre-resolved context only
* Prevent any model-driven repository exploration
* Require diff-only outputs with strict validation
* Implement AST-based rule enforcement engine
* Replace “memory” with deterministic workspace state
* Treat LLM as stateless transformation backend
* Decouple Alfred from any single model provider or Copilot internals
