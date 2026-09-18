# Tasks: Dynamic Skill Selection & Injection (Intelligent Skill Routing)

## Estimated Scope: ~680 LOC across 4 atomic work units (~150–190 LOC per work unit)

- [x] **Work Unit 1: Skill Classification Metadata & Schema Extension** (est. ~150 LOC)
  - [x] Extend `SkillEntry` and `SkillFrontmatter` in `src/source/registry.rs` with `pub categories: Vec<String>`.
  - [x] Update YAML frontmatter parser in `parse_frontmatter` to extract `categories` or `decision.categories`.
  - [x] Define `SkillRoutingConfig` in `src/decisions/skill_routing.rs` and extend `DecisionsConfig` in `src/state/state.rs`.
  - [x] Add unit tests verifying serialization, frontmatter extraction, and `Eq` trait compliance.
  - [x] TDD Verification: `cargo test source::registry::tests::categories`

- [x] **Work Unit 2: Core Skill Router & Intent Classification** (est. ~190 LOC)
  - [x] Implement `SkillRouter` in `src/decisions/skill_routing.rs`.
  - [x] Implement 7 semantic category questions (`architecture`, `security`, `testing`, `debugging`, `documentation`, `code_review`, `research`).
  - [x] Implement confidence filtering against `minimum_confidence_pct`.
  - [x] Implement `SkillRegistry::resolve_with_routing` combining candidate categories, scope precedence, and SHA256 integrity verification.
  - [x] Add unit tests in `src/decisions/tests/skill_routing_tests.rs` with `MockDecisionProvider` simulating single-category, multi-category, below-threshold, and provider error cases.
  - [x] TDD Verification: `cargo test decisions::skill_routing::tests`

- [x] **Work Unit 3: CLI Subcommand `ce-ai skills resolve` Diagnostics** (est. ~170 LOC)
  - [x] Update `Action::Resolve` in `src/commands/skills.rs` to support `--json` and `--verbose` diagnostics.
  - [x] Wire `SkillRouter` into `skills resolve` when enabled in configuration.
  - [x] Output verbose category score breakdown, candidate categories, and resolved skills.
  - [x] Add unit tests verifying CLI argument handling and output formatting.
  - [x] TDD Verification: `cargo test commands::skills::tests`

- [x] **Work Unit 4: Presets, Doctor Probe Integration & CLI Tests** (est. ~170 LOC)
  - [x] Update `ce-ai decisions setup --preset recommended` to enable skill routing (`minimum_confidence_pct = 70`).
  - [x] Update `probe_decision_engine_health` in `src/commands/doctor.rs` to display skill routing status.
  - [x] Add end-to-end CLI integration test in `tests/cli.rs` (`test_cli_skills_resolve_with_routing`).
  - [x] Run full project quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`).
  - [x] TDD Verification: `cargo test --test cli skills_resolve`
