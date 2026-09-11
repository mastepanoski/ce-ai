# Tasks: Engram Context Echo Behavioral Risk Documentation

- [x] 1. Author Architectural Solution Document (`docs/solutions/architecture/engram-context-echo-behavioral-risk.md`) [~120 LOC]
  - [x] 1.1 Add YAML front-matter with tags, components, and applies_when.
  - [x] 1.2 Document symptom, initial hypotheses, and refutation of Hypothesis 2 in `ce-ai`.
  - [x] 1.3 Document audit of third-party Engram codebase with exact file and line citations.
  - [x] 1.4 Analyze LLM behavioral feedback loop mechanics.
  - [x] 1.5 Detail practiced manual mitigation and advisory upstream recommendation.

- [x] 2. Update Domain Vocabulary (`CONCEPTS.md`) [~10 LOC]
  - [x] 2.1 Add `Engram Context Echo (Behavioral Loop)` definition with cross-link.

- [x] 3. Update User Guide (`docs/user-guide/fsm-and-checkpoints-explained.md`) [~10 LOC]
  - [x] 3.1 Add `[!NOTE]` alert under the tasks desync section regarding memory echo persistence.

- [x] 4. Verification and Delivery [~0 LOC]
  - [x] 4.1 Verify `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] 4.2 Verify zero modifications in `src/`, `tests/`, or `Cargo.toml`.
  - [x] 4.3 Commit, push feature branch, and open PR.
