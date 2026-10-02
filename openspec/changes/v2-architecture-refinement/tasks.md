# Tasks: Decoupling Semantic Authority & OpenSpec in CE-AI v2

- [x] **Unit 1 (~120 LOC):** Update Architecture PRD with Inviolable Principles & Decoupled Domain
  - Add "No Semantic Authority" and "Repository Reality Over Mirrored State" to `docs/architecture/ce-ai-v2-architecture-prd.md`.
  - Update Core Boundary Matrix and Architecture diagram showing OpenSpec as an optional integration outside CE-AI Core.
  - Replace rigid 7-stage linear pipeline description with Observable Capabilities Matrix.

- [x] **Unit 2 (~110 LOC):** Update Migration Plan with Refined KTDs & Phase Roadmap
  - Update `docs/plans/2026-10-01-ce-ai-v2-architectural-migration-plan.md` to reflect OpenSpec decoupling.
  - Add KTDs for "No Semantic Authority" and "Capabilities Observation over Numbered Stage Cursor".
  - Refine Phase 3 to articulate the transition from linear FSM stages to capability observation.

- [x] **Unit 3 (~50 LOC):** Monotonic Concept Accretion in `CONCEPTS.md`
  - Accrete new domain concepts: *No Semantic Authority*, *Repository Reality Over Mirrored State*, *Observable Workflow State*, and *Optional Integration Autonomy*.
  - Verify with `python3 scripts/validate-concepts.py CONCEPTS.md`.

- [x] **Unit 4: Code Simplification & Review (`ce-simplify-code`)**
  - Ensure documentation adheres strictly to Diátaxis and cognitive load rules (`docs-styling.md`).
  - Verify zero clippy or format warnings across the workspace.

- [x] **Unit 5: Structured Code Review & Review Receipt**
  - Record workflow review receipt: `cargo run -- workflow review-receipt`.

- [x] **Unit 6: Knowledge Compounding & Vocabulary Capture (`ce-compound`)**
  - Document solution learning in `docs/solutions/architecture/decoupling-semantic-authority-and-openspec.md`.
  - Verify `cargo run -- doc lint --strict`.

- [x] **Unit 7: Verification, SemVer bump, and PR Creation (`ce-commit-push-pr`)**
  - Bump PATCH version to `1.75.2` in `Cargo.toml` and record changes in `CHANGELOG.md`.
  - Run full test suite: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
  - Push branch and open PR with upfront empirical evidence.
