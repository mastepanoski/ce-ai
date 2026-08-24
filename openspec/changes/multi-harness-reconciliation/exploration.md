# Exploration: Multi-Harness Reconciliation & Release Hardening

## Technical Investigation

### 1. README & Spec Alignment (Issues #155 & #183)
- The codebase ships 10 native adapters (`OpenCode`, `Cursor`, `Claude`, `Codex`, `Copilot`, `Grok`, `Kimi`, `Agy`, `Pi`, `Fx`).
- `README.md` and `openspec/changes/multi_harness_support/spec.md` should accurately document these 10 native harnesses and their exact native formats.

### 2. DeepSeek De-scope Decision (Issue #180)
- Official DeepSeek Harness (`dsh`) uses `~/.dsh` and `cordis.patch.yml` YAML patch rows.
- Writing fictional JSON files (`~/.config/deepseek/deepseek.json`) violates Invariant #5 (no dummy fallbacks/fictional paths).
- Action: When `--harness deepseek` is requested, return `CeError::Usage` (exit code 2) explaining the developer-preview status of `dsh` and guiding users to supported native adapters.

### 3. GitHub API 403 Rate Limit Fallback (Issue #202)
- In `src/source/release.rs`, when GitHub API returns HTTP 403 / 429:
  - Print stderr notice: `notice: GitHub API rate limit reached (403 Forbidden). Falling back to main branch tarball.`
  - Return `Ok(None)` to allow callers in `install.rs` and `upgrade.rs` to fall back to `main_tarball_url()`.

### 4. Audit Score Labeling (Issue #164)
- In `src/commands/audit.rs`: Update `score: {}%` output to `configuration coverage: {}%`.
