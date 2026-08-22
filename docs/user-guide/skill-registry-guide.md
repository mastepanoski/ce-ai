<!-- Diátaxis Quadrant: How-to / Reference | Audience: Both -->
# Skill Registry Guide (`ce-ai skills`)

The **Multi-Harness Skill Registry Engine** indexes, validates, and resolves `SKILL.md` instruction files across all 12 supported AI coding agent harnesses. It provides a central, harness-neutral master catalog stored at `~/.ce-ai/skills-registry.json` that enables agents to discover available skills, verify SHA256 integrity, and inject sub-agent prompt blocks deterministically.

---

## Quick Reference Commands

| Command | Purpose |
| :--- | :--- |
| `ce-ai skills list [--harness <kind>] [--json]` | Catalog list of all indexed skills with scope and SHA256 hashes |
| `ce-ai skills resolve --harness <kind> --query <query> [--json]` | Resolve skills for a harness and generate sub-agent prompt blocks |
| `ce-ai skills doctor` | Run diagnostic health probes against the central registry index |

---

## How-to: Resolve Skills for Sub-Agents

To query and inject skills into sub-agent prompts:

1. **Resolve skills in Markdown prompt format (default)**:

   ```bash
   ce-ai skills resolve --harness opencode --query brainstorm
   ```

   *Output:*
   ```markdown
   <!-- ce-ai:skill_resolution status=paths-injected timestamp=2026-08-22T16:00:00Z -->
   ## Skills to load before work:
   - **ce-brainstorm**: Explore vague or ambitious ideas into right-sized requirements
     Path: `file:///home/user/.ce-ai/skills/ce-brainstorm/SKILL.md`
   ```

2. **Resolve skills in JSON machine-readable format**:

   ```bash
   ce-ai skills resolve --harness claude --query "review" --json
   ```

3. **Check resolution degradation status**:
   - `status=paths-injected`: Matching skills found and verified clean SHA256 hashes.
   - `status=fallback-fuzzy`: Matching skills found, but files were missing or SHA256 hash drifted.
   - `status=none`: No skills matched the query.

---

## 4-Tier Precedence Resolution Hierarchy

When `ce-ai` builds or refreshes the skill index (`install`, `sync`, `upgrade`, `init-prj`), it scans skill directories across 4 priority tiers:

| Tier | Level | Path | Purpose |
| :--- | :--- | :--- | :--- |
| **Tier 1** | Workspace Central (Highest) | `<cwd>/.ce-ai/skills/` | Repository-specific skill overrides |
| **Tier 2** | Workspace Harness | `<cwd>/.opencode/skills/` | OpenCode project-scoped skill overrides |
| **Tier 3** | Global Harness User Roots | `~/.config/<harness>/skills/` | Harness-specific global user skill overrides |
| **Tier 4** | Global Managed (Lowest) | `~/.ce-ai/skills/` | Default managed skill catalog installed by `ce-ai` |

---

## Security & Diagnostics

- **R3 Path Security**: Canonicalizes all candidate paths to ensure they stay within authorized root boundaries. Symlink escapes or relative path traversals (`../`) are rejected.
- **Diagnostic Probes**: Running `ce-ai doctor` or `ce-ai skills doctor` validates:
  1. Registry file existence and JSON schema validity.
  2. Existence of underlying `SKILL.md` files.
  3. Resolution-time SHA256 digest integrity.
