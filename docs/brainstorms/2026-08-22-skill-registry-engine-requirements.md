# Requirements Document: Multi-Harness Skill Registry Engine (`ce-ai skills`)

- **Document Version**: 1.0.0
- **Date**: 2026-08-22
- **OpenSpec Change Reference**: `openspec/changes/skill_registry_engine/`
- **Issue Reference**: #96

---

## 1. Problem & User Goal

### Problem
`ce-ai` currently manages skills and loader scripts across 12 AI coding agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`). While file paths and SHA256 hashes are recorded in `install-manifest.json` for drift detection, `ce-ai` lacks a structured metadata index.

Without a multi-harness skill registry:
- Agent sessions re-scan disk paths from scratch every time sub-agents are launched.
- Hardcoded harness paths (e.g. `~/.config/opencode/`) violate multi-harness neutrality.
- Corrupted skills or broken frontmatter fail silently at runtime instead of being caught by diagnostic probes.
- No auditable trace exists mapping task triggers to exact `SKILL.md` paths injected into sub-agent prompts.

### User Goal
Provide a native, multi-harness **Skill Registry Engine** within `ce-ai` that indexes skills across all active host harnesses and workspace repositories, exposing a fast resolution API (`ce-ai skills resolve`) and diagnostic probes (`ce-ai doctor`).

---

## 2. Scope Boundaries

### In-Scope
- **Central Storage (`~/.ce-ai/skills-registry.json`)**: Harness-neutral master index managed via `crate::state::write_atomic`.
- **Multi-Harness Path Resolution**: Indexing skills across global managed paths (`~/.ce-ai/skills/`), global user paths (`~/.config/<harness>/skills/`), and adopted workspace repositories (`.ce-ai/skills/`, `.opencode/skills/`).
- **YAML Frontmatter Parsing**: Extracting `name`, `description`, `triggers`, `scope`, and `harness_paths` from `SKILL.md` files.
- **Subcommand Suite (`ce-ai skills`)**:
  - `ce-ai skills list`: Full catalog display with SHA256 health status.
  - `ce-ai skills resolve --harness <kind> --query "<task>"`: Dual-format resolution output (Markdown prompt block + JSON).
  - `ce-ai skills doctor`: Diagnostic probe for missing files, invalid frontmatter, or corrupted digests.
- **Lifecycle & Uninstall Parity**:
  - Auto-refresh on `ce-ai install`, `sync`, `upgrade`.
  - Complete removal of `~/.ce-ai/skills-registry.json` on `ce-ai uninstall`.
  - Removal of project stubs and managed `.gitignore` entries on `ce-ai deinit-prj`.

### Out-of-Scope
- Remote network skill downloading (skills remain local or fetched via release tarballs).
- Executing skill scripts directly (skills provide markdown instructions for AI agents to consume).

---

## 3. Product & Decision Log

| Decision ID | Topic | Decision Made | Rationale |
|-------------|-------|---------------|-----------|
| **DEC-01** | Output Format | Dual Format: Markdown prompt block + JSON (`--json`) | Allows direct sub-agent prompt injection (`## Skills to load...`) while remaining machine-parseable for CLI tools. |
| **DEC-02** | Conflict Precedence | Local Override with Global Fallback | Workspace-local skills (`.ce-ai/skills/`) override global skills (`~/.ce-ai/skills/`) of the same name, preserving non-conflicting global skills. |
| **DEC-03** | Degradation Handling | Explicit Degradation Tag (`paths-injected` \| `fallback-fuzzy` \| `none`) + `stderr` Warning | Prevents fatal crashes during prompt resolution while providing clear observability of missing or corrupted skills. |
| **DEC-04** | Harness Neutrality | Global Master Index at `~/.ce-ai/skills-registry.json` | Ensures `ce-ai` remains neutral across all 12 supported AI coding agent harnesses. |

---

## 4. Multi-Harness Resolution Contract

### Command Interface
```bash
ce-ai skills resolve --harness <harness-kind> --query "<task-or-trigger>" [--json]
```

### Output Formats

#### Default Markdown Output (Ready for Sub-Agent Prompt Injection)
```markdown
<!-- ce-ai:skill_resolution status=paths-injected timestamp=2026-08-22T14:30:00Z -->
## Skills to load before work:
- **ce-brainstorm**: Explore vague or ambitious ideas into right-sized requirements
  Path: `file:///Users/user/.config/opencode/compound-engineering/skills/ce-brainstorm/SKILL.md`
```

#### JSON Output (`--json`)
```json
{
  "resolution_status": "paths-injected",
  "query": "brainstorm",
  "harness": "opencode",
  "skills": [
    {
      "name": "ce-brainstorm",
      "scope": "project",
      "description": "Explore vague or ambitious ideas into right-sized requirements",
      "path": "/Users/user/project/.ce-ai/skills/ce-brainstorm/SKILL.md",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    }
  ]
}
```

---

## 5. Success Criteria

- [ ] `skills-registry.json` is generated under `~/.ce-ai/` on `ce-ai install` and `ce-ai sync`.
- [ ] `ce-ai skills list` displays catalog for any specified host harness.
- [ ] `ce-ai skills resolve --harness <kind> --query "<query>"` outputs dual-format prompt blocks.
- [ ] Local workspace skills (`.ce-ai/skills/`) cleanly override global skills of the same name.
- [ ] Resolution degradation is explicitly tagged (`paths-injected` vs `fallback-fuzzy` vs `none`).
- [ ] `ce-ai uninstall` removes `~/.ce-ai/skills-registry.json` and reverts `.gitignore` entries.
- [ ] `ce-ai doctor` includes `skill-registry-integrity` probe flagging missing or corrupted files.
- [ ] 100% green unit, CLI integration, and security test coverage (`cargo test`).
