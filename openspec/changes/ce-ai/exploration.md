# Exploration: ce-ai — CE plugin manager CLI

## Context

- **Project**: `ce-ai` — a brand-new Rust CLI in `/Users/mastepanoski/projects/web/ai/ce-ai` (`Cargo.toml`, `src/main.rs` stub, `cargo test` TDD gate).
- **Goal**: Manage installation, sync/upgrade, and model configuration of the `compound-engineering-plugin` (CE) across multiple AI coding harnesses, modeled on `gentle-ai`.
- **Reference implementation**: CE plugin repo at `/Users/mastepanoski/projects/web/ai/compound-engineering-plugin` has **no installer CLI**; installs are done via each harness's native mechanism or a legacy Bun converter (`src/commands/install.ts`) that writes files directly for opencode/codex/pi/antigravity.
- **Reference UX**: `gentle-ai` (`/opt/homebrew/bin/gentle-ai` v2.4.0) uses `~/.gentle-ai/state.json` for installed state and `model_assignments`, plus `~/.config/opencode/profiles/*.json` for append-only model profiles. Commands: `install`, `sync`, `uninstall`, `upgrade`, `doctor`, `skill-registry refresh`, review/SDD helpers.
- **Constraint**: A functional E2E test MUST run in an isolated Docker container with a fresh HOME and opencode installed, proving install → sync/upgrade → model config.

## A. Scope of the Three Pillars

### A1. INSTALL — wrapping native commands vs. direct file writing

#### Evidence from CE repo
- `README.md` lists native install paths per harness:
  - **Claude Code**: `/plugin marketplace add EveryInc/compound-engineering-plugin` then `/plugin install compound-engineering` — interactive slash commands inside Claude Code, not scriptable from a shell.
  - **Cursor**: `/add-plugin compound-engineering` — interactive Cursor Agent chat command.
  - **Codex CLI**: `codex plugin marketplace add EveryInc/compound-engineering-plugin` + `codex plugin add compound-engineering@compound-engineering-plugin`.
  - **Kimi CLI**: `/plugins install https://github.com/EveryInc/compound-engineering-plugin`.
  - **Pi**: `pi install git:github.com/EveryInc/compound-engineering-plugin` (+ `pi install npm:pi-subagents`).
  - **AGY (Antigravity)**: clone repo, then `agy plugin install ./compound-engineering-plugin/.agy`.
  - **OpenCode**: add `"plugin": ["compound-engineering@git+https://github.com/EveryInc/compound-engineering-plugin.git"]` to `opencode.json`.
- Legacy Bun converter in `src/commands/install.ts` and `src/targets/` (opencode.ts, codex.ts, pi.ts, antigravity.ts) writes plugin manifests/files directly:
  - **OpenCode**: merges `opencode.json`, writes `plugins/compound-engineering.js`, skills under `skills/`, agents/commands.
  - **Codex**: writes `config.toml` managed MCP block, `hooks.json`, prompts/agents/skills under `~/.codex`, and `install-manifest.json`.
  - **Pi**: writes skills/prompts/extensions/agents under `~/.pi/agent`, `AGENTS.md` block, `install-manifest.json`.
  - **AGY**: writes `.agy` plugin contents directly.

#### Headless automatibility

| Harness | Headless shell-out possible? | Direct file write possible? | Recommendation |
|---------|------------------------------|----------------------------|----------------|
| OpenCode | No native install command; config-only | **Yes** — edit `opencode.json` + plugin loader | First slice |
| Codex CLI | **Yes** — `codex plugin marketplace add` + `codex plugin add` | **Yes** — replicate converter file writes | First slice (shell-out is canonical; file-write for E2E or offline) |
| Pi | **Yes** — `pi install git:...` | Partial — can write skills/agents directly, but `pi-subagents` package is still `npm:` | First slice (shell-out) |
| AGY | Partial — requires local checkout + `agy plugin install ./.agy` | **Yes** — copy `.agy` bundle to harness dir | First slice |
| Claude Code | **No** — `/plugin` is interactive | Hard — would need manifest placement unsupported by Claude | Instruct user / detect-only |
| Cursor | **No** — `/add-plugin` is interactive | Hard — Cursor plugin store is app-managed | Instruct user / detect-only |
| Kimi | Yes via CLI | Not investigated; low priority | Defer |
| Copilot/Droid/Qwen | Shell commands exist | Converter code exists | Defer |

#### Options for install strategy

| Approach | Pros | Cons | Effort |
|----------|------|------|--------|
| **A. Native shell-out only** (`codex plugin add`, `pi install`, etc.) | Uses each harness's intended path; handles format validation, caching, signature checks | Claude/Cursor not scriptable; requires harness binaries installed; slower; harder to rollback precisely | Medium |
| **B. Direct file write only** (re-implement Bun converter in Rust) | Fully headless; deterministic; fast; easy to snapshot/diff; matches CE converter ownership model | Bypasses harness package manager; must replicate manifest schemas and cleanup logic; future CE format drift is our responsibility | High |
| **C. Hybrid — direct file write for opencode + agy; shell-out for codex + pi; instruct for Claude/Cursor** | Maximizes what is automatable; falls back to native commands where they are headless and canonical; leaves interactive-only harnesses to the user | Two code paths to maintain; shell-out still needs binary present and idempotency handling | Medium |
| **D. Hybrid with file-write fallback** — prefer shell-out, but if binary missing use direct write | Works in Docker E2E without all harness binaries | Complexity of reconciling "native install" state vs. file-write state | High |

#### Recommendation for INSTALL
Adopt **Approach C (hybrid) with a first-slice of OpenCode + Codex + Pi + AGY**.

- **OpenCode**: direct write to `opencode.json` plugin array and `plugins/` loader (like `.opencode/plugins/compound-engineering.js`). No native install command exists, so direct write is the only path.
- **Codex**: shell out to `codex plugin marketplace add` + `codex plugin add` by default; offer `--offline` / `--direct-write` flag that replicates the Bun converter for E2E or air-gapped scenarios.
- **Pi**: shell out to `pi install git:...` and `pi install npm:pi-subagents`; optionally `--offline` for direct skill/agent writes.
- **AGY**: direct write of `.agy/` plugin bundle after optionally cloning the CE repo locally (mirrors `agy plugin install <local dir>`).
- **Claude/Cursor**: detect presence, print exact interactive commands, mark as `manual` in status.

### A2. SYNC/UPGRADE — what does sync mean and where does state live

#### Evidence
- `gentle-ai` stores canonical state in `~/.gentle-ai/state.json`: `installed_agents`, `managed_asset_digest`, `components`, `preset`, `model_assignments`, `last_update_check`.
- CE converter tracks per-install state via `install-manifest.json`:
  - Codex: `~/.codex/compound-engineering/install-manifest.json` (`version`, `pluginName`, `skills`, `prompts`, `agents`).
  - Pi: `~/.pi/agent/compound-engineering/install-manifest.json`.
  - OpenCode: `~/.config/opencode/compound-engineering/install-manifest.json`.
- These manifests are used to clean up removed skills/agents on re-install.
- CE has no global update check; updates are ad-hoc git pulls or marketplace refreshes.

#### Options for sync strategy

| Approach | Pros | Cons | Effort |
|----------|------|------|--------|
| **(a) Re-fetch from GitHub** — clone latest tag/main and re-run install per harness | Simple mental model; always matches upstream CE | Requires network; may pull unwanted breaking changes; does not reconcile local drift | Low |
| **(b) Reconcile local install manifest vs. installed files** | Detects manual user edits/drift; safe incremental updates; offline-capable | Needs hash or content diff; must know what "current" should be | Medium |
| **(c) Both** — fetch latest CE, compute desired manifest, diff against installed manifest and filesystem | Most robust; supports upgrade, repair, and drift detection | More implementation work; needs canonical source-of-truth | High |

#### Recommendation for SYNC/UPGRADE
Adopt **Approach (c) — both**, but implement in phases:

1. **Phase 1**: `sync` = re-run install from the current locally cached CE source (or latest GitHub release if no cache). Write/update `install-manifest.json` and CE-managed files.
2. **Phase 2**: `upgrade` = check GitHub releases/tags, fetch newer version if available, then sync.
3. **Canonical state**: introduce `~/.ce-ai/state.json` (gentle-ai-style) containing:
   - `installed_harnesses`: list with `name`, `version`, `source`, `last_synced_at`.
   - `managed_asset_digest`: SHA256 of the CE source used.
   - `model_assignments`: per-harness/per-agent model map.
4. **Drift detection**: compare `install-manifest.json` group lists against actual filesystem; `doctor` reports missing/extra files.

### A3. MODELS — configuring models for harness subagents

#### Evidence
- CE ships **27 skills, 0 standalone agents** (`README.md` line 108). Model assignment therefore targets **harness subagents** where supported.
- `gentle-ai` model assignment format:
  - `~/.gentle-ai/state.json` → `model_assignments: { "<slot>": { "provider_id", "model_id", "effort?" } }`.
  - Writes `agent.<slot>.model` (+ `variant`) into `~/.config/opencode/opencode.json`.
  - Profiles: `~/.config/opencode/profiles/*.json` with `{ "models": { "<slot>": "provider/model" } }` and `profile-versions/` snapshots.
- OpenCode config (`~/.config/opencode/opencode.json`) supports per-agent `model` and `variant` fields (observed: `gentle-orchestrator.model = "kimi-for-coding/kimi-for-coding"`, `variant = ""`).
- Codex `config.toml` supports top-level `model` and `model_reasoning_effort`; per-agent model support is **not evident** in the CE converter or local config.
- Pi agent files (`~/.pi/agent/agents/*.toml`) support a `model` field (common in Pi); need verification.
- Claude agent markdown files may include `model` in YAML frontmatter (needs verification).

#### Options for model persistence

| Approach | Pros | Cons | Effort |
|----------|------|------|--------|
| **Single `~/.ce-ai/state.json`** with `model_assignments` | Mirrors gentle-ai; one source of truth; easy to backup/version | Harness-specific writes must derive from it; not all harnesses may consume it directly | Low |
| **Per-harness config writes only** | Directly useful to each harness | No cross-harness view; harder to sync/rollback | Medium |
| **State.json + profiles + snapshots** (gentle-ai-style) | Rich history, easy rollback, composable profiles | More files to manage; overkill if only opencode supports per-agent models initially | Medium |

#### Recommendation for MODELS
Adopt **state.json + per-harness writes + optional profiles**:

1. Primary format: `~/.ce-ai/state.json` with:
   ```json
   {
     "model_assignments": {
       "sdd-explore": { "provider_id": "opencode-go", "model_id": "kimi-k2.6", "effort": "high" },
       "ce-work": { "provider_id": "kimi-for-coding", "model_id": "kimi-for-coding" }
     }
   }
   ```
2. Write per-harness config:
   - **OpenCode**: `agent.<name>.model` and `agent.<name>.variant` in `opencode.json`.
   - **Codex**: top-level `model` + `model_reasoning_effort` in `config.toml` (per-agent if/when Codex supports it).
   - **Pi**: `model` field in `~/.pi/agent/agents/*.toml` if supported.
   - **Claude/Cursor**: agent markdown frontmatter `model` field if supported.
3. Optional profiles directory: `~/.ce-ai/profiles/*.json` with append-only snapshots under `~/.ce-ai/profiles/versions/`.
4. Commands: `ce-ai models`, `ce-ai models set <slot> <provider/model> [--effort ...]`, `ce-ai models profile <name>`.

## B. E2E Docker Strategy

### B1. OpenCode headless in Docker

- **Official image?** No `ghcr.io/opencode-ai/opencode` found; upstream is `ghcr.io/sst/opencode` or `ghcr.io/anomalyco/opencode`.
- **Community images**: `n8500x/opencode` (ships `opencode-ai` npm, JDK, Node, Python; headless `opencode run --format json`); `ghcr.io/brockar/opencoded` (web/TUI); `ghcr.io/nimbleflux/opencode-docker`.
- **Alternative**: install via `npm install -g opencode-ai` in a `node:slim` or `debian:bookworm-slim` container.

### B2. What can be exercised in the container

| Harness | In container? | How to assert |
|---------|---------------|---------------|
| OpenCode | **Yes** — install npm package or use image | `opencode.json` contains plugin entry; `plugins/compound-engineering.js` exists; skills path registered |
| Codex | Partial — `codex` CLI may need auth/OTP; better to assert direct file writes | `~/.codex/config.toml` managed block present; install-manifest exists; skills dir populated |
| Pi | Partial — `pi` CLI may need auth | File writes to `~/.pi/agent/skills/ce-*`; `AGENTS.md` managed block |
| AGY | No `agy` in lightweight image | Direct copy of `.agy/` bundle; assert files |

### B3. Recommended E2E harness

**Use a Rust integration test that spawns a Docker container** (requires `docker` available at test time).

- Dockerfile: based on `node:22-bookworm-slim` (or `n8500x/opencode`), installs opencode via `npm install -g opencode-ai`, copies the built `ce-ai` binary, sets a fresh `HOME=/tmp/ce-ai-home`.
- Test flow:
  1. `docker build -t ce-ai-e2e .`
  2. `docker run --rm -e HOME=/tmp/ce-ai-home ce-ai-e2e ce-ai install --harness opencode --dry-run`
  3. `ce-ai install --harness opencode` (real)
  4. `ce-ai sync`
  5. `ce-ai models set sdd-explore opencode-go/kimi-k2.6`
  6. Assert: `cat $HOME/.config/opencode/opencode.json | jq '.plugin'` contains CE entry; `agent.sdd-explore.model` set; plugin loader file exists.
- For CI: add a GitHub Actions job (or Makefile target `e2e`) that builds the binary, builds the image, and runs the assertions.
- Fallback if Docker is unavailable locally: shell script with `mktemp -d` fake HOME and assert file contents. This does NOT replace the Docker gate.

## C. CLI Shape

Recommend a `clap`-based CLI modeled on `gentle-ai`:

```text
ce-ai [OPTIONS] <COMMAND>

Commands:
  install       Install CE into one or more harnesses
  sync          Reconcile installed CE files with the current source
  upgrade       Fetch latest CE release and sync
  models        Manage per-harness subagent model assignments
  status        Show installed harnesses, versions, and drift
  uninstall     Remove CE-managed files from harnesses
  doctor        Run health/diagnostics checks
  help          Print this message or the help of the given subcommand(s)
```

### Key flags

- Global:
  - `--config-dir <PATH>` — override `~/.ce-ai` state/config directory.
  - `--dry-run` — show what would change without writing files.
  - `--verbose`, `-v` / `--quiet`, `-q`.
- `install`:
  - `--harness <NAME>` — repeatable; if absent, auto-detect installed harnesses.
  - `--all` — install into all supported/headless harnesses.
  - `--scope global|workspace` — opencode/agy scope.
  - `--source <PATH|URL|TAG>` — local clone, GitHub URL, or release tag (default: latest GitHub release).
  - `--offline` / `--direct-write` — skip native shell commands and write files directly.
- `sync`:
  - `--harness <NAME>` — default all installed.
  - `--verify` — also run `doctor` after sync.
- `upgrade`:
  - `--to <TAG>` — pin to a release; default latest.
- `models`:
  - `set <slot> <provider/model>`.
  - `profile save <name>` / `profile load <name>`.
  - `list`.

### Config file location and format

- `~/.ce-ai/state.json` — canonical state (installed harnesses, model assignments, last update check, digest).
- `~/.ce-ai/profiles/*.json` — optional named model profiles.
- `~/.ce-ai/config.toml` (optional) — user preferences like default harnesses, release channel, auto-update.

## Risks

1. **Claude/Cursor are not scriptable** — ce-ai cannot fully automate those harnesses; UX must clearly tell the user what to run manually and verify state.
2. **Harness format drift** — OpenCode/Codex/Pi config formats change without warning; direct file writes may break. Mitigation: pin supported harness versions in CI and treat format changes as breaking CE releases.
3. **E2E fragility** — Docker image availability, npm install time, and opencode auth requirements can make E2E slow/flaky. Mitigation: use offline/direct-write mode for the E2E assertions so the test does not depend on live npm/opencode auth.
4. **Dual install paths (shell-out vs. direct-write)** — may leave state inconsistent (e.g., Codex native install + file-write fighting). Mitigation: pick one default per harness and make the alternative opt-in with clear docs.
5. **Model assignment support varies** — per-agent model config may only work reliably in OpenCode at first; other harnesses may only support global model, limiting the "models" pillar value.

## Open Questions for Proposal Round

1. Should the first slice **only** support OpenCode (simplest E2E) and add Codex/Pi/AGY in a follow-up slice, or tackle all four headless harnesses from the start?
2. Should `ce-ai` depend on the CE repo being cloned locally, or should it fetch from GitHub releases by default? Where is the source-of-truth for the 27 skills (git tag tarball vs. npm-style asset)?
3. Should Codex install default to native `codex plugin add` or direct file-write? Native is canonical but harder in Docker; direct-write is easier for E2E.
4. How should `sync` detect drift for harnesses that do not have an install manifest (e.g., a manually edited `opencode.json`)? Hash-based or content-based?
5. Do we want profiles/snapshots for model assignments in the first release, or start with a single `state.json`?
6. What is the exact Pi agent `.toml` model field key, and does Codex support per-agent `model` in custom agent TOML? (Need to verify runtime behavior.)
7. Should `ce-ai` also manage the `pi-subagents` / `pi-ask-user` companion packages, or leave that to the user/docs?
8. Should `uninstall` remove backups/legacy-backup dirs or keep them for safety?

## Recommendation Summary

- **Install**: hybrid approach — direct file write for OpenCode and AGY; shell out for Codex and Pi; manual instructions for Claude/Cursor. First-slice harnesses: OpenCode, Codex, Pi, AGY.
- **Sync/Upgrade**: canonical state in `~/.ce-ai/state.json` + per-harness `install-manifest.json`; sync reconciles desired manifest vs. installed files; upgrade fetches latest CE release then syncs.
- **Models**: gentle-ai-style `model_assignments` in `state.json`; write per-agent `model`/`variant` to OpenCode `opencode.json`; write global model/effort to Codex `config.toml`; extend to Pi/Claude/Cursor as format support is verified.
- **E2E**: Rust integration test that builds a Docker image with opencode installed and asserts real file mutations in an isolated `HOME`.
- **CLI**: clap subcommands `install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, `doctor`; global `--config-dir`, `--dry-run`, `--verbose`/`--quiet`.

**Ready for proposal**: Yes, with the open questions above resolved in the proposal phase.
