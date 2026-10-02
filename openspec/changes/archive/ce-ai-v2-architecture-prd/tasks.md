# Tasks: CE-AI v2 Architectural Pivot & PRD

- [x] **Unit 1 (~200 LOC):** OpenSpec Specification Package
  - Draft and formalize `proposal.md`, `exploration.md`, `design.md`, and `spec.md` under `openspec/changes/ce-ai-v2-architecture-prd/`.
  - Align with `AGENTS.md` guidelines and upstream maintainer recommendations.
  - *Verification*: Validate file existence and completeness.

- [x] **Unit 2 (~220 LOC):** Comprehensive Architecture PRD Document
  - Author `docs/architecture/ce-ai-v2-architecture-prd.md` detailing the CE-AI v2 architectural pivot.
  - Address all 5 maintainer critique points: state ownership, fleet governance, doc debt, concrete bugs, and stable contracts.
  - Define the Rust CE Compatibility Layer interface and the phased migration roadmap.
  - *Verification*: Document review and verification against maintainer critique.

- [x] **Unit 3 (~80 LOC):** Domain Vocabulary Accretion in `CONCEPTS.md`
  - Monotonically accrete domain glossary with new architectural concepts:
    - *Cross-Host Operational Companion*
    - *Advisory Workflow Observation Engine*
    - *Native Harness Installer Orchestrator*
    - *Upstream Schema Compatibility Layer*
    - *Configurable Docs Root Resolution*
  - Verify zero clobbering or deletion of existing terms.
  - *Verification*: `cargo run -- doc lint --strict` and `python3 scripts/validate-concepts.py`.

- [x] **Unit 4: Code Simplification & Review (`ce-simplify-code`)**
  - Verify docs-only nature of this architectural change.
  - Confirm markdown styling aligns with `docs/references/docs-styling.md`.

- [x] **Unit 5: Structured Code Review & Review Receipt**
  - Review technical plan compliance (`docs/plans/2026-10-01-ce-ai-v2-architectural-migration-plan.md`).
  - Generate workflow review receipt.

- [x] **Unit 6: Knowledge Compounding & Learning Capture (`ce-compound`)**
  - Capture architectural solution in `docs/solutions/architecture/ce-ai-v2-architectural-pivot.md`.
  - Validate frontmatter against schema.

- [x] **Unit 7: Verification, SemVer bump, and Shipping (`ce-commit-push-pr`)**
  - Bump PATCH/MINOR version in `Cargo.toml` and update `CHANGELOG.md`.
  - Run full test gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
  - Open PR with empirical verification evidence and clean conventional commit history.
