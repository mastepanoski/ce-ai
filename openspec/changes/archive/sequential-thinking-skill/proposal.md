# Proposal: Canonical Sequential-Thinking Skill Integration

## Problem Statement

In `ce-ai`, `sequential-thinking` is currently defined as an on-demand companion skill suggestion in [`src/source/tools_registry.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/tools_registry.rs#L116-L124) (`name: "sequential-thinking"`, `resolve_cmd: "ce-ai skills resolve sequential-thinking"`). However, running `ce-ai skills resolve sequential-thinking` today produces:

```markdown
<!-- ce-ai:skill_resolution status=none -->
## Skills to load before work:
```

This degradation occurs because no physical `sequential-thinking/SKILL.md` file exists in any indexed skill directory (`~/.ce-ai/skills/`, `~/.config/opencode/compound-engineering/skills/`, etc.). Because the skill is never indexed in `skills-registry.json`, [`is_skill_configured(&ctx, "sequential-thinking")`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/tools_registry.rs#L348-L367) returns `false`, causing `ce-ai doctor` and `ce-ai tools status` to report a perpetual unconfigured suggestion pointing to a command that yields no actionable path.

Issue #309 closed the architectural question by rejecting Option (a) (registering an external Node.js MCP server daemon) and adopting Option (b) (on-demand skill model). This proposal defines Sub-option 1: delivering structured reasoning guidance via an authentic, physically distributed `SKILL.md` file integrated into `ce-ai`'s native skill harvesting and indexing pipeline.

## In-Scope Boundaries

- **Authored Protocol (`skills/sequential-thinking/SKILL.md`)**: Author a comprehensive, production-grade `SKILL.md` defining an explicit step-by-step reasoning protocol: linear step progression, dynamic thought revision, hypothesis formulation, falsification criteria, and evaluation of contradictory evidence.
- **Dual-Compatible Frontmatter**: Format frontmatter with fields required by `ce-ai`'s [`SkillFrontmatter`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs#L48-L54) (`name`, `description`, `triggers`, `scope`) while preserving compatibility with harness slash-command conventions (`argument-hint`).
- **Distribution & Seeding Pipeline**:
  - Store the canonical file at `skills/sequential-thinking/SKILL.md` within the `ce-ai` repository.
  - Embed the content via `include_str!` as a compile-time fallback constant (`BUILTIN_SEQUENTIAL_THINKING_SKILL`).
  - Ensure `ce-ai install` and `ce-ai sync` guarantee placement into managed skill roots (`~/.ce-ai/skills/sequential-thinking/SKILL.md` or harness managed paths) even when upstream release tarballs lack the file.
- **Registry Indexing & Resolution**:
  - Guarantee [`SkillRegistry::build`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs#L127-L211) discovers, hashes, and indexes `sequential-thinking` into `~/.ce-ai/skills-registry.json`.
  - Guarantee [`SkillRegistry::resolve`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs#L214-L278) resolves `ce-ai skills resolve sequential-thinking` to `status=paths-injected` with a verified `file://` URI.
- **Diagnostic Auto-Resolution**:
  - Confirm [`is_skill_configured(&ctx, "sequential-thinking")`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/tools_registry.rs#L348-L367) automatically returns `true` once indexed, cleanly satisfying `ce-ai doctor` and `ce-ai tools status` without ad-hoc code changes.
- **Automated Verification**:
  - Add unit tests in `src/source/tests/registry.rs` and CLI integration tests in `tests/cli.rs` validating installation, discovery, resolution status, hash verification, and doctor freshness reporting.

## Out-of-Scope Boundaries

- Promoting `sequential-thinking` to a registered MCP server or requiring `@modelcontextprotocol/server-sequential-thinking`.
- Modifying [`SkillRegistry::resolve`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs#L214-L278) to synthesize in-memory markdown strings or bypass SHA256 file validation.
- Adding runtime Node.js, npm, or npx dependencies.
- Modifying the Pi harness architecture or violating its strict No-MCP design invariant.

## Risk Evaluation & Mitigation

- **Risk: Upstream Tarball Lag**: If `ce-ai install` fetches an upstream release tarball from `everyinc/compound-engineering-plugin` that does not yet contain `skills/sequential-thinking/SKILL.md`, installation might miss the file.
  - *Mitigation*: Embed `BUILTIN_SEQUENTIAL_THINKING_SKILL` in the binary and seed it into the managed skills directory during `install`/`sync` if missing from the source tree, mirroring the proven pattern used for `BUILTIN_LOADER` in [`src/opencode/plugins.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/opencode/plugins.rs#L17).
- **Risk: Frontmatter Field Collisions or Parsing Rejections**: Using YAML keys unsupported by `ce-ai` could cause parse errors.
  - *Mitigation*: [`parse_skill_frontmatter`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs#L450-L509) uses a resilient, zero-panic line-based parser that captures `name`, `description`, `scope`, and `triggers`, while cleanly ignoring unrecognized keys like `argument-hint`.
- **Risk: Dry-Run Mutation Leakage**: Seeding builtin skills during `--dry-run` could violate the SU-4 invariant.
  - *Mitigation*: Gate all seeding and atomic writes behind `if !ctx.dry_run`, logging planned copies instead during dry-run executions.

## Success Criteria

1. `skills/sequential-thinking/SKILL.md` exists and contains valid YAML frontmatter and a comprehensive structured reasoning protocol.
2. `ce-ai install` places `sequential-thinking/SKILL.md` into the target skill tree across all supported harnesses.
3. `ce-ai skills resolve sequential-thinking` returns `<!-- ce-ai:skill_resolution status=paths-injected -->` with a valid `file://` link.
4. `is_skill_configured(&ctx, "sequential-thinking")` returns `true`.
5. `ce-ai doctor` does not report `sequential-thinking` as an unconfigured skill suggestion.
6. 100% test pass rate, 0 clippy warnings, and clean formatting.
