> STATUS (v1.20.1): ce-ai init-prj/deinit-prj commands live. Residual open boxes below were not re-audited item-by-item.

# OpenSpec Task Breakdown & Implementation Checklist: Project Adoption Engine

## Tasks & Checklist

### Phase 1: Core Data Schemas & Trait Extensions
- [ ] **Task 1.1: Add Project Adoption State Schemas**
  - File: `src/state/state.rs`
  - Implement `AdoptionTier` enum (`Full`, `Minimal`, `Orchestrator`) and `ProjectAdoptionEntry` struct.
  - Add `pub projects: Vec<ProjectAdoptionEntry>` field to `State`.
  - Add unit tests verifying serialization and deserialization in `state.json`.

- [ ] **Task 1.2: Extend `HarnessAdapter` Trait**
  - File: `src/harness/mod.rs` & `src/harness/*.rs`
  - Add `canonical_instruction_file()` and `derived_stub_files()` methods to `HarnessAdapter`.
  - Implement methods across all 12 harness adapters (`OpenCode`, `Claude`, `Pi`, `Cursor`, `Copilot`, `Codex`, `Grok`, `Kimi`, `AGY`, `DeepSeek`, `FX`, `Custom`).

### Phase 2: Command Implementations (`init-prj` & `deinit-prj`)
- [ ] **Task 2.1: Add CLI Command Parsing**
  - File: `src/main.rs`
  - Add `InitPrj` and `DeinitPrj` subcommands to `Commands` enum with `--tier` and `--force` flags.
  - Add command dispatch routing in `main.rs`.

- [ ] **Task 2.2: Implement `ce-ai init-prj` Subcommand**
  - File: `src/commands/init_prj.rs` & `src/commands/mod.rs`
  - Implement template rendering (`full`, `minimal`, `orchestrator`), marker injection, SHA calculation, and `write_atomic` file mutations.
  - Update adoption registry in `state.json`.

- [ ] **Task 2.3: Implement `ce-ai deinit-prj` Subcommand**
  - File: `src/commands/deinit_prj.rs`
  - Implement marker extraction, clean file restoration, deletion of temporary `AGENTS.md` if `created_file` is true, and registry cleanup.

### Phase 3: Diagnostics & Auditing (`status` & `doctor`)
- [ ] **Task 3.1: Integrate Adoption Auditing into `ce-ai status`**
  - File: `src/commands/status.rs`
  - Display adopted project list, tier, and SHA integrity status.

- [ ] **Task 3.2: Integrate Adoption Auditing into `ce-ai doctor`**
  - File: `src/commands/doctor.rs`
  - Add diagnostic probes for missing instruction files and manual marker drift.

### Phase 4: TUI & Skill Integration
- [ ] **Task 4.1: TUI Action Shortcuts**
  - File: `src/tui.rs`
  - Add `[I] Init Prj` and `[D] Deinit Prj` key action handlers.

- [ ] **Task 4.2: OpenCode Slash Command & Skill Packaging**
  - Files: `src/opencode/plugins.rs`, `skills/ce-ai-init-prj/SKILL.md`
  - Package `/ce-ai-init-prj` slash command and skill.

### Phase 5: Verification & Testing
- [ ] **Task 5.1: Unit & Integration Test Suite**
  - Files: `tests/cli.rs`, `tests/security.rs`
  - Test `init-prj` ➔ `deinit-prj` golden byte-for-byte roundtrip.
  - Test idempotency and `--force` conflict resolution.
- [ ] **Task 5.2: Verification Pipeline**
  - Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `make e2e`.

### Phase 6: Documentation
- [ ] **Task 6.1: User Documentation & README**
  - Files: `README.md`, `ROADMAP.md`, `CONCEPTS.md`, `CHANGELOG.md`.
  - Document Project Adoption Engine and `init-prj` / `deinit-prj` usage.
