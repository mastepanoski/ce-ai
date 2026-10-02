# Exploration: Decoupling Semantic Authority & OpenSpec in CE-AI v2

## 1. Context & Investigation

In CE-AI v1, the operational model was tightly coupled to a mandatory 7-stage development cycle:
```text
Stage 1: Ideation ➔ Stage 2: OpenSpec ➔ Stage 3: Plan ➔ Stage 4: Work ➔ Stage 5: Verify ➔ Stage 6: Compound ➔ Stage 7: Ship
```

Every code change—regardless of whether it was a 2-line typo fix, an urgent bug patch, or a large architectural overhaul—was forced through Stage 2. If `openspec/changes/<feature>/` did not exist with `proposal.md`, `spec.md`, and `tasks.md`, `ce-ai gate check` blocked the write.

The upstream maintainer of Compound Engineering identified two critical architectural flaws with this approach:
1. **Unnecessary Documentation Debt:**
   > *"The part of ce-ai that pushes the other way is the write gate. Requiring proposal/spec/tasks docs before any write grows the pile that then has to be kept trustworthy."*
2. **False Linearity:**
   > *"The stages also aren't linear in CE. Trivial work skips planning, ce-debug has its own path, and ce-compound only writes a doc when there's something worth capturing."*

## 2. Evaluated Architectural Options

### Option A: Complete Removal of OpenSpec from CE-AI
- **Description:** Strip all references to OpenSpec, delete `openspec/` tooling, and rely exclusively on upstream CE skills.
- **Trade-offs:**
  - *Pros:* Maximally minimal core. Completely eliminates any possibility of write gating.
  - *Cons:* Throws out the baby with the bathwater. For major architectural features, multi-agent migrations, or complex API refactors, Spec-Driven Development (formal `WHEN ... THEN ...` clauses, bounded task lists) is demonstrably valuable to prevent agent hallucinations and provide unambiguous delivery contracts.
- **Verdict:** **Ruled Out.**

### Option B: Retain OpenSpec as Mandatory Core, Expand Exemptions
- **Description:** Keep OpenSpec as Stage 2 in the core FSM, but add more exemption flags (e.g. `--skip-spec`, `trivial:*`, `quickfix:*`).
- **Trade-offs:**
  - *Pros:* Minimal changes to v1 codebase.
  - *Cons:* Keeps CE-AI as an authoritative gatekeeper that imposes its own semantics on CE. Adding exemption rules creates a maze of heuristic edge cases, perpetuating the exact coupling friction the maintainer criticized.
- **Verdict:** **Ruled Out.**

### Option C: Decouple Semantic Authority & Reposition OpenSpec as Optional Integration (Selected)
- **Description:**
  1. Formally adopt the principle: **"CE-AI must not require artifacts that Compound Engineering itself does not require."**
  2. Strip OpenSpec of any workflow authority. Remove the `OpenSpec Required` hard invariant and write gate.
  3. Decouple OpenSpec from CE-AI's core domain model:
     ```text
     CE-AI Core
      ├── CE compatibility/contracts (docs_root, schema.yaml, mode:return-to-caller)
      ├── Host adapters (Claude, OpenCode, Codex, Pi, Cursor, Copilot, Antigravity)
      ├── Workflow observation (read-only capability detection)
      └── Environment coordination (fleet version governance)
            │
            ├── OpenSpec integration (optional, opt-in)
            └── Future tool integrations
     ```
  4. Replace the rigid numbered stages (`Stage 1 -> Stage 2 -> ...`) with **Observable Workflow State / Capabilities**:
     Rather than asking *"What stage are we in?"*, CE-AI asks:
     > *"What do we objectively know about the current state of the workflow?"*
- **Trade-offs:**
  - *Pros:* Fully respects upstream CE autonomy; completely eliminates false documentation debt; honors non-linear workflows (`ce-debug`, direct work, conditional compounding); keeps OpenSpec available for complex tasks when the user/agent chooses it.
  - *Cons:* Requires redefining the internal workflow observation engine and documentation models.
- **Verdict:** **Selected.**
