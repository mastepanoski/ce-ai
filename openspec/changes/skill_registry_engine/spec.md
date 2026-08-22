# OpenSpec Specification: Multi-Harness Skill Registry Engine

- **Feature Name**: `skill_registry_engine`
- **Issue Reference**: #96
- **Status**: Draft / Proposed

---

## Requirements

### Requirement 1: Harness-Neutral Storage
WHEN `ce-ai install`, `sync`, or `upgrade` executes,
THEN `ce-ai` SHALL generate and update `skills-registry.json` under `~/.ce-ai/` using `write_atomic`.

### Requirement 2: Multi-Harness Skill Resolution
WHEN `ce-ai skills resolve --harness <kind> --query "<query>"` is invoked,
THEN `ce-ai` SHALL return the exact absolute `SKILL.md` path corresponding to the specified harness `<kind>` without defaulting to `opencode`.

### Requirement 3: Health Probe Integration
WHEN `ce-ai doctor` is executed,
THEN `ce-ai` SHALL audit `skills-registry.json` for missing files, SHA256 drift, and malformed frontmatter YAML.

### Requirement 4: Uninstall Parity & .gitignore Maintenance
WHEN `ce-ai uninstall` or `deinit-prj` is executed,
THEN `ce-ai` SHALL clean up `skills-registry.json`, remove any generated workspace-local skill registry artifacts, and clean up `.gitignore` entries added by `ce-ai`.
