# Specification: Canonical Sequential-Thinking Skill Integration

## Requirements

### R1: Canonical Asset Presence & Frontmatter Validity
- WHEN `skills/sequential-thinking/SKILL.md` is inspected, THEN it MUST exist in the repository tree and contain non-empty markdown documentation defining a structured reasoning protocol.
- WHEN `skills/sequential-thinking/SKILL.md` is parsed by `parse_skill_frontmatter`, THEN:
  - `fm.name` MUST equal `"sequential-thinking"`.
  - `fm.description` MUST be non-empty.
  - `fm.scope` MUST equal `"global"`.
  - `fm.triggers` MUST contain at least `"complex reasoning"` and `"sequential thought"`.
  - Additional keys such as `argument-hint` MUST NOT cause parsing panics, errors, or corruptions.

### R2: Fallback Seeding During Install and Sync
- WHEN `ce-ai install` or `ce-ai sync` runs against a source tree lacking `skills/sequential-thinking/SKILL.md`, THEN the command MUST seed the canonical skill from `BUILTIN_SEQUENTIAL_THINKING_SKILL` into the target harness skill directory using `write_atomic`.
- WHEN `ce-ai install` or `ce-ai sync` runs against a source tree that already contains `skills/sequential-thinking/SKILL.md`, THEN the file from the source tree MUST be copied and take precedence over the embedded constant.

### R3: Dry-Run Safety Invariant (SU-4)
- WHEN `ce-ai install --dry-run` or `ce-ai sync --dry-run` is invoked, THEN the system MUST NOT create, modify, or delete any files under `~/.ce-ai/skills/` or any harness skills directory.
- WHEN `--dry-run` is active, THEN the command MUST output planned actions (`plan: seed/copy skills/sequential-thinking/SKILL.md`) to standard output.

### R4: Registry Indexing & Cryptographic Integrity
- WHEN `SkillRegistry::build` is invoked after installation, THEN:
  - `sequential-thinking` MUST be present in `registry.skills`.
  - Its `sha256` digest MUST match the SHA256 hash of the on-disk `SKILL.md`.
  - Its `harness_paths` map MUST contain valid absolute filesystem paths for all active harnesses.
- WHEN `SkillRegistry::sync_registry` executes, THEN `~/.ce-ai/skills-registry.json` MUST be updated atomically with POSIX permissions `0644`.

### R5: Deterministic Resolution Output
- WHEN `ce-ai skills resolve sequential-thinking` is executed and the indexed file matches the on-disk SHA256, THEN:
  - The status tag MUST be `paths-injected`.
  - The output markdown MUST contain `<!-- ce-ai:skill_resolution status=paths-injected -->`.
  - The output MUST contain a `- **sequential-thinking**:` entry with a valid `file://` URI pointing to the on-disk `SKILL.md`.
- WHEN the physical file is tampered with, corrupted, or deleted after indexing, THEN the status tag MUST degrade to `fallback-fuzzy` or `none`, and the corrupted path MUST NOT be reported as verified.

### R6: Diagnostic Auto-Resolution
- WHEN `skills-registry.json` contains a verified entry for `sequential-thinking`, THEN `is_skill_configured(&ctx, "sequential-thinking")` MUST return `true`.
- WHEN `is_skill_configured(&ctx, "sequential-thinking")` returns `true`, THEN:
  - `ce-ai doctor` MUST NOT emit an unconfigured `skill-suggestion: sequential-thinking` warning.
  - `ce-ai tools status` MUST display `sequential-thinking` with installed/configured status.

### R7: Harness Parity & Zero External Runtime Dependencies
- WHEN `sequential-thinking` is resolved for the `pi` harness (`ce-ai skills resolve --harness pi sequential-thinking`), THEN it MUST return a valid `file://` URI without registering any MCP server or violating Pi's strict No-MCP design invariant.
- The installation, indexing, and resolution of `sequential-thinking` MUST NOT require `node`, `npm`, `npx`, or any external daemon process.
