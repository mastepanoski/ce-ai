# Tasks: Canonical Sequential-Thinking Skill Integration

- [ ] 1. Author canonical `skills/sequential-thinking/SKILL.md` in repository root (~90 LOC)
  - Author frontmatter with unified fields (`name`, `description`, `argument-hint`, `scope`, `triggers`).
  - Author structured reasoning protocol (step progression, hypothesis testing, revision logging, falsification, synthesis).
- [ ] 2. Create `src/source/builtin_skills.rs` with embedded fallback constant (~35 LOC)
  - Embed canonical skill content via `include_str!`.
  - Provide helper `ensure_builtin_skills(target_dir, dry_run)` to seed missing skills into the target managed directory.
  - Expose module in `src/source/mod.rs`.
- [ ] 3. Wire fallback seeding into `src/commands/install.rs` and `src/commands/sync.rs` (~45 LOC)
  - In `install.rs` and `sync.rs`, check if `skills/sequential-thinking/SKILL.md` was harvested from `source_path`.
  - If absent, seed the embedded skill into target harness skills directory using `write_atomic` behind `if !ctx.dry_run` and `arm!`.
- [ ] 4. Unit tests in `src/source/tests/registry.rs` (~60 LOC)
  - Test frontmatter parsing of the unified schema (`name`, `description`, `argument-hint`, `scope`, `triggers`).
  - Test `SkillRegistry::build` discovery and SHA256 digest computation for `sequential-thinking`.
  - Test `SkillRegistry::resolve` returning `status=paths-injected` and verified `file://` URI.
  - Test `SkillRegistry::resolve` degrading to `fallback-fuzzy` on file mutation or deletion.
- [ ] 5. CLI integration tests in `tests/cli.rs` (~60 LOC)
  - Test `ce-ai install` seeds `sequential-thinking` and indexes it in `skills-registry.json`.
  - Test `ce-ai skills resolve sequential-thinking` produces `status=paths-injected` with non-empty URI.
  - Test `ce-ai skills resolve --harness pi sequential-thinking` functions seamlessly without MCP registration.
  - Test `ce-ai doctor` confirms `is_skill_configured("sequential-thinking") == true` and emits zero unconfigured suggestions.
- [ ] 6. Documentation & Solution Architecture update (~40 LOC)
  - Document `sequential-thinking` in `docs/user-guide/skill-registry-guide.md`.
  - Capture architectural solution in `docs/solutions/architecture/sequential-thinking-skill-integration.md`.
- [ ] 7. SemVer version bump and CHANGELOG update (~15 LOC)
  - Bump version to `1.43.0` in `Cargo.toml`.
  - Document changes in `CHANGELOG.md` following Keep a Changelog standard.

Total estimated LOC: ~345 lines.
