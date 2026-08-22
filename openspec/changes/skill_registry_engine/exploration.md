# OpenSpec Exploration: Multi-Harness Skill Registry Engine

- **Feature Name**: `skill_registry_engine`
- **Issue Reference**: #96
- **Status**: Draft / Proposed

---

## 1. Technical Investigation & Codebase Audit

### Current Skill Management Architecture
- `src/opencode/plugins.rs`: Defines `skills_path(config_dir)` pointing at `config_dir.join("compound-engineering").join("skills")`.
- `src/opencode/manifest.rs`: Records individual file paths and SHA256 digests in `install-manifest.json`.
- `src/commands/install.rs` & `src/commands/sync.rs`: Copies managed skill files and updates manifest entries.

### The Multi-Harness Challenge
`ce-ai` supports 12 AI coding agent harnesses, each with distinct default config and skill directories:
1. `opencode`: `~/.config/opencode/compound-engineering/skills/`
2. `claude`: `~/.claude/skills/`
3. `cursor`: `.cursor/rules/` or `.cursorrules`
4. `copilot`: `.github/copilot-instructions.md`
5. `pi`: `~/.pi/skills/`
6. `kimi`: `~/.kimi-code/skills/`
7. `agy`: `~/.gemini/antigravity-cli/skills/`
8. `codex`, `grok`, `deepseek`, `fx`, `custom`: harness-specific or custom paths.

Storing the registry inside `opencode.json` or `~/.config/opencode/` violates harness neutrality. Storing it in `~/.ce-ai/skills-registry.json` ensures that `ce-ai` remains the single neutral source of truth across all 12 harnesses.

---

## 2. Evaluated Options

### Option A: Harness-Specific Skill Registries
- Store separate registry files inside each harness's config directory (e.g. `~/.config/opencode/skills.json`, `~/.claude/skills.json`).
- **Pros**: Isolated per harness.
- **Cons**: High duplication, fragmentation, difficult to audit centrally via `ce-ai doctor`.

### Option B: Neutral Global Registry with Per-Harness Mappings (CHOSEN)
- Maintain master index at `~/.ce-ai/skills-registry.json`.
- Each skill entry in the registry contains a map of supported harnesses and their corresponding target paths.
- **Pros**: Single source of truth, atomic writes, zero duplication, harness-neutral CLI query API.
- **Cons**: Requires scanning multiple target paths during `sync`.

---

## 3. Tradeoffs & Architectural Choices

| Dimension | Decision | Rationale |
|-----------|----------|-----------|
| **Storage Path** | `~/.ce-ai/skills-registry.json` | Harness-neutral master directory managed by `ce-ai`. |
| **Frontmatter Parsing** | Lightweight YAML parser (`yaml-rust2` / `serde_yaml`) | Extracts `name`, `description`, `triggers` from `SKILL.md`. |
| **Mutation Safety** | `crate::state::write_atomic` | Prevents corrupted state files on process interruption. |
| **CLI Command Set** | `ce-ai skills list`, `resolve`, `doctor` | Clean subcommand interface for agents and humans. |
