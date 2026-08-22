# OpenSpec Proposal: Multi-Harness Skill Registry Engine (`ce-ai skills`)

- **Feature Name**: `skill_registry_engine`
- **Issue Reference**: #96 (*Analyze adopting a skill registry similar to Gentle AI's*)
- **Status**: Draft / Proposed
- **Author**: Compound Engineering AI (`ce-ai`) Team
- **Date**: 2026-08-22

---

## 1. Executive Summary & Problem Statement

`ce-ai` currently distributes skills and loader scripts across 12 supported AI coding agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`). File placement and integrity hashes are tracked in `install-manifest.json` for drift detection.

However, `ce-ai` currently lacks a structured metadata catalog of installed skills. Without a multi-harness skill registry:
- **Redundant Discovery**: Agent sessions must re-scan disk paths from scratch every time sub-agents are launched.
- **Single-Harness Path Bias**: Storing skill indexes under harness-specific paths (e.g. `~/.config/opencode/`) breaks multi-harness neutrality.
- **Silent Degradation**: Broken skill paths, malformed YAML frontmatter, or corrupted files fail silently at runtime instead of being caught by diagnostic probes.
- **Lack of ISO 42001 Auditing**: No deterministic record exists mapping task triggers to exact `SKILL.md` paths injected into sub-agent prompts.

---

## 2. Proposed Solution

Implement a native, multi-harness **Skill Registry Engine** within `ce-ai`:
1. **Central Master Storage (`~/.ce-ai/skills-registry.json`)**: Located under the global `ce-ai` configuration directory (`~/.ce-ai/`), independent of any single host harness.
2. **Multi-Harness Path Mapping**: Scans and maps skills across all 12 active host harnesses, global user paths (`~/.config/<harness>/skills`), and adopted workspace repositories (`.opencode/skills/`, `.ce-ai/skills/`, `.githooks/`).
3. **CLI Interface (`ce-ai skills`)**:
   - `ce-ai skills list`: Displays full catalog of indexed skills across all active harnesses with SHA256 health status.
   - `ce-ai skills resolve --harness <kind> --query "<task>"`: Returns absolute `SKILL.md` paths for prompt injection.
   - `ce-ai skills doctor`: Diagnostic probe for missing files, invalid frontmatter, or corrupted digests.
4. **Automated Lifecycle Integration**:
   - `ce-ai install` / `ce-ai upgrade`: Generates and updates `skills-registry.json` using atomic writes (`write_atomic`).
   - `ce-ai sync`: Re-scans global and workspace skill paths, updating SHA256 digests.
   - `ce-ai doctor`: Checks `skill-registry-integrity`.

---

## 3. Scope Boundaries

### In-Scope
- Multi-harness skill discovery and YAML frontmatter parsing (`name`, `description`, `triggers`, `scope`, `harness`).
- Central registry storage at `~/.ce-ai/skills-registry.json` using `write_atomic`.
- New `ce-ai skills` subcommand suite (`list`, `resolve`, `doctor`).
- Multi-harness path resolution for all 12 supported harnesses.
- Integration tests in `tests/cli.rs` and unit tests in `src/source/registry.rs`.

### Out-of-Scope
- Remote network skill repositories (skills remain local or fetched via standard release tarballs).
- Executing skill scripts directly (skills provide instruction markdown for agents to consume).

---

## 4. Risk Evaluation & Mitigation

| Risk | Impact | Mitigation Strategy |
|------|--------|---------------------|
| Malformed `SKILL.md` YAML frontmatter | Medium | Graceful fallback parsing with warning in `ce-ai doctor`; never panic. |
| Hardcoded harness paths | High | Harness-neutral storage at `~/.ce-ai/skills-registry.json` with per-harness path mapping structs. |
| Concurrent state writes | High | Enforce atomic file operations via `crate::state::write_atomic`. |

---

## 5. Success Criteria

- [ ] `skills-registry.json` is generated under `~/.ce-ai/` during `ce-ai install` and `ce-ai sync`.
- [ ] `ce-ai skills list` displays catalog for any specified or active harness.
- [ ] `ce-ai skills resolve --harness <kind> --query "<keyword>"` resolves exact `SKILL.md` paths.
- [ ] `ce-ai doctor` includes `skill-registry-integrity` diagnostic probe.
- [ ] 100% green unit, CLI integration, and security tests (`cargo test`).
