# OpenSpec Task List: Multi-Harness Skill Registry Engine

- [ ] **Task 1: Data Models & Storage (`src/source/registry.rs`)**
  - Implement `SkillEntry` and `SkillRegistry` structs.
  - Add YAML frontmatter parser for `SKILL.md` headers.
  - Implement atomic JSON persistence (`write_atomic`).

- [ ] **Task 2: CLI Subcommand Handler (`src/commands/skills.rs` & `src/main.rs`)**
  - Add `ce-ai skills` subcommand parser with `list`, `resolve`, and `doctor`.
  - Wire CLI arguments to `src/source/registry.rs`.

- [ ] **Task 3: Integration in Lifecycle (`install.rs`, `sync.rs`, `doctor.rs`)**
  - Trigger `build_skill_registry()` in `install.rs` and `sync.rs`.
  - Add `skill-registry-integrity` probe to `doctor.rs`.

- [ ] **Task 4: Unit & Integration Testing (`tests/cli.rs`)**
  - Add unit tests for frontmatter parsing and registry round-tripping.
  - Add CLI integration tests for `ce-ai skills list` and `ce-ai skills resolve`.
