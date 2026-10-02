# Design: Decoupling Semantic Authority & OpenSpec in CE-AI v2

## 1. Architectural Principles (Inviolable Design Laws)

CE-AI v2 establishes two foundational architectural principles that govern all design decisions:

### Principle 1: No Semantic Authority
> **CE-AI MUST NOT introduce mandatory workflow stages, artifacts, or transition requirements beyond those defined by the Compound Engineering contracts it integrates with.**
>
> **Optional integrations such as OpenSpec MAY introduce additional artifacts or constraints only when explicitly activated by the user or calling workflow.**
>
> **CE-AI MAY observe, validate, visualize, or coordinate these contracts, but MUST NOT silently redefine Compound Engineering workflow semantics.**

### Principle 2: Repository Reality Over Mirrored State
> **Durable repository artifacts and documented CE contracts are authoritative. CE-AI-derived workflow state is advisory and disposable: it MUST be reconstructable from authoritative sources and MUST NOT become an independent source of workflow truth.**

---

## 2. Decoupled Core Domain Architecture

CE-AI v2 separates the system into a minimal, non-authoritative **Core Domain** and pluggable **Optional Integrations**:

```text
                        Compound Engineering
                                  │
                      (owns workflow semantics)
                                  ▼
 ┌─────────────────────────────────────────────────────────────────┐
 │                           CE-AI Core                            │
 │                                                                 │
 │   ┌───────────────────────┐         ┌───────────────────────┐   │
 │   │   CE Compatibility    │         │     Host Adapters     │   │
 │   │ (docs_root, schemas)  │         │ (Claude, Codex, etc.) │   │
 │   └───────────────────────┘         └───────────────────────┘   │
 │                                                                 │
 │   ┌───────────────────────┐         ┌───────────────────────┐   │
 │   │ Workflow Observation  │         │  Fleet Coordination   │   │
 │   │ (advisory capabilities│         │  (version governance) │   │
 │   └───────────────────────┘         └───────────────────────┘   │
 └────────────────────────────────┬────────────────────────────────┘
                                  │
              ┌───────────────────┴───────────────────┐
              ▼                                       ▼
    ┌───────────────────┐                   ┌───────────────────┐
    │     OpenSpec      │                   │   Future Tools    │
    │    Integration    │                   │   & Sidecars      │
    │ (optional/opt-in) │                   │ (optional/opt-in) │
    └───────────────────┘                   └───────────────────┘
```

### Roles and Boundaries:
1. **Compound Engineering**: Owns the workflow semantics, artifact formats (`plans/`, `solutions/`), and execution contracts (`mode:return-to-caller`).
2. **AI Hosts (Claude, Codex, OpenCode, Pi, etc.)**: Own native execution, session contexts, and native plugin packaging.
3. **CE-AI Core**:
   - *Compatibility*: Knows how to parse CE configs (`docs_root`, `schema.yaml`, mode tokens).
   - *Host Adapters*: Knows how to invoke native host installers and configure non-destructive hooks.
   - *Workflow Observation*: Objectively reports repository workflow facts without enforcing transitions.
   - *Fleet Coordination*: Assures release version parity across developer machines.
4. **OpenSpec Integration (Optional)**:
   - Sits strictly *outside* the core CE-AI domain model.
   - Provides formal spec creation (`proposal.md`, `spec.md`, `tasks.md`) **only** when explicitly invoked by the developer or agent (`ce-ai spec new <name>`).
   - Imposes zero write gates or blocking checks on native Compound Engineering work.

---

## 3. Observable Workflow State (Capabilities vs. Numbered Stages)

Rather than maintaining a linear stage counter (`Stage 1 -> Stage 2 -> Stage 3 ...`), CE-AI v2 observes repository reality directly and builds an **Observable Capabilities Matrix**:

```rust
// Proposed Rust Model for v2 src/observation/state.rs

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObservableWorkflowState {
    pub active_work: bool,
    pub active_branch: Option<String>,
    pub plan: Option<PlanObservation>,
    pub verification: VerificationObservation,
    pub handoff: Option<HandoffObservation>,
    pub knowledge_capture: KnowledgeObservation,
    pub openspec: Option<OpenSpecObservation>, // None if OpenSpec is inactive
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanObservation {
    pub path: std::path::PathBuf,
    pub completed_items: usize,
    pub total_items: usize,
    pub is_requirements_only: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificationObservation {
    pub has_evidence: bool,
    pub uncommitted_changes: usize,
    pub review_receipt_stamped: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeObservation {
    pub required: bool,
    pub doc_detected: bool,
    pub doc_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenSpecObservation {
    pub feature: String,
    pub path: std::path::PathBuf,
    pub is_sealed: bool,
}
```

### Key Shift:
- Old Question: *"What stage are we in?"* (Forcing artificial stage transitions and reconciliation).
- New Question: *"What do we objectively know about the current state of the workflow?"*
- Non-linear paths are naturally supported:
  - Trivial fixes: `plan: None`, `active_work: true`, `verification: pending`.
  - Bugfixes (`ce-debug`): `plan: None`, `active_work: true`, `knowledge_capture: required`.
  - Full Spec-Driven Work: `openspec: Some(...)`, `plan: Some(...)`, `active_work: true`.
