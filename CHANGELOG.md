# Changelog

All notable changes to `ce-ai` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.37.3] - 2026-09-04

### Fixed
- **Decoupled `ce-ai sync` and `upgrade` from OpenCode Manifest Precondition (`src/commands/sync.rs`, `src/commands/tests/sync.rs`, `tests/cli.rs`):**
  - Resolved `ce-ai sync` and `ce-ai upgrade` failing with exit code 1 (`no install-manifest.json — run install first`) when OpenCode is not installed (e.g. standalone `claude`, `copilot`, `cursor`, or `custom` harness environments).
  - Implemented `resolve_sync_source_and_version` helper to dynamically source plugin package and version information across multiple providers: probing OpenCode manifest if present, standalone harness manifests on disk (e.g. `~/.<harness>/compound-engineering/install-manifest.json` and custom plugins directories), `state.installed_harnesses` metadata, or release provenance.
  - Returns a clear error message `no harnesses installed — run ce-ai install first` when `state.installed_harnesses` is empty and no harnesses are host-detected.
  - Gated OpenCode managed directory synchronization, manifest persistence, model assignment updates, and matrix reporting behind `opencode_active`, ensuring non-OpenCode environments display their active harnesses cleanly in the Sync Verification Matrix without an erroneous `opencode` row.
  - Preserved custom harness manifest mutations directly from `cfg.plugins_dir` when updating custom harness installs during sync.
  - Added CLI integration tests and unit tests verifying sync and upgrade flows across standalone native harnesses, custom harness installs, and empty state validation.

## [1.37.2] - 2026-09-04

### Fixed
- **Workspace-Scoped OpenCode Manifest Resolution in Doctor, Status, and Sync (`src/commands/mod.rs`, `src/commands/install.rs`, `src/commands/doctor.rs`, `src/commands/status.rs`, `src/commands/sync.rs`):**
  - Stored `scope` and `target_dir` metadata in `state.installed_harnesses` when running `ce-ai install --harness opencode --scope workspace`, enabling downstream commands to resolve the workspace-level install location.
  - Implemented `Context::resolve_opencode_dir(&self, state: &State) -> PathBuf` to contextually resolve OpenCode manifest and config roots: checks matching `installed_harnesses` workspace records, falls back to probe `<workspace_root>/compound-engineering/install-manifest.json` on disk, and defaults to `ctx.opencode_config_dir` (`~/.config/opencode`).
  - Updated `ce-ai doctor` to resolve OpenCode paths using `resolve_opencode_dir`, eliminating false-positive `state-inconsistent` and missing plugin findings in worktrees and workspace-scoped setups.
  - Updated `ce-ai status` and `ce-ai sync` to evaluate manifests and drift against `resolve_opencode_dir`, preserving workspace-level tracking and metadata across sync operations.
  - Added CLI integration and unit tests covering workspace-scoped manifest verification, drift resolution, and state consistency.

## [1.37.1] - 2026-09-04

### Fixed
- **Workflow Feature Name Clearing on Reset to Stage 1 (`src/state/state.rs`):**
  - Updated `validate_and_set_workflow` so that transitioning to Stage 1 (`WorkflowStage::Ideation`) from any advanced stage clears the previous `feature_name` by default if `--feature` is omitted.
  - Advancing or staying on the current stage without `--feature` continues to seamlessly inherit the active feature.
  - Passing an explicit empty or whitespace-only feature string (`--feature ""`) is now treated as an intentional clear (`None`), preventing invalid empty path resolutions in `probe_openspec_context_in`.
- **OpenSpec Context Probing Defense-in-Depth (`src/commands/workflow.rs`):**
  - Added filter in `probe_openspec_context_in` ensuring empty or whitespace-only feature strings trigger fallback directory discovery under `openspec/changes/`.
  - Corrected `--stage` / `--phase` CLI flag documentation in `Action::Checkpoint` to show valid token examples (`4`, `work`, `tdd`).
- **User Guide Checkpoint Command Fixes (`docs/user-guide/`):**
  - Corrected copy-pasteable checkpoint examples in `quick-start-workflow-guide.md` and `fsm-and-checkpoints-explained.md` from invalid display strings (`"Stage 4: Work/TDD"`) to valid stage tokens (`--stage 4`).

## [1.37.0] - 2026-09-04

### Added
- **Guaranteed Turn-0 Drift Delivery for Google Antigravity CLI via Native `PreInvocation` Hook (`src/harness/agy.rs`, `src/commands/init_prj.rs`):**
  - Implemented `has_pre_invocation_hook`, `ensure_pre_invocation_hook`, and `remove_pre_invocation_hook` (with backward-compatible `*_session_start_hook` aliases) managing `<project>/.agents/hooks.json`.
  - Configures `PreInvocation` command hook invoking `ce-ai workflow resume --pre-invocation`, which executes before model invocations in Google Antigravity CLI (`agy`).
  - Implemented session deduplication in `ce-ai workflow resume --pre-invocation` using `conversationId` (and `sessionId` fallback) to deliver live `RepoState` via `injectSteps` containing `ephemeralMessage` on Turn-0, and returning empty `{}` in sub-millisecond time on subsequent turns.
  - Surgically parses and writes `.agents/hooks.json` via `write_atomic`, preserving existing user hook groups (e.g. `custom-security`, `PreToolUse`) and non-destructive JSON merges.
  - Cleans up `.agents/hooks.json` and rules on project de-adoption (`ce-ai deinit-prj`), safely removing empty parent directories while preserving user configurations.
- **Antigravity CLI Doctor Diagnostic Finding (`src/commands/doctor.rs`):**
  - Added audit probe in `ce-ai doctor` flagging missing Antigravity `PreInvocation` hooks in adopted projects containing `.agents/`.
- **Documentation & Solution Architecture:**
  - Updated `docs/user-guide/zero-step-drift-recovery-explained.md` explaining the per-turn vs session-start distinction and session deduplication.
  - Created solution architecture record in `docs/solutions/architecture/2026-09-02-antigravity-pre-invocation-drift-delivery.md`.

## [1.36.0] - 2026-09-02

### Added
- **Guaranteed Turn-0 Drift Delivery for Cursor via Native `sessionStart` Hook (`src/harness/cursor.rs`, `src/commands/init_prj.rs`):**
  - Implemented `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` managing `<project>/.cursor/hooks.json`.
  - Configures `sessionStart` command hook invoking `ce-ai workflow resume --json`, which executes on composer session creation across both Cursor desktop IDE and Cursor CLI (v0.45+).
  - Surgically parses and writes `.cursor/hooks.json` via `write_atomic`, preserving all pre-existing user hooks (e.g. `preToolUse`, custom scripts) and top-level user configuration fields.
  - Cleans up `.cursor/hooks.json` and `.cursor/rules/compound-engineering.mdc` on project de-adoption (`ce-ai deinit-prj`), safely removing empty parent directories while preserving user-defined rules.
- **Snake-Case `additional_context` in `workflow resume --json` (`src/commands/workflow.rs`):**
  - Added `additional_context` field to `workflow resume --json` output to natively satisfy Cursor's expected JSON stdout schema.
- **Cursor Doctor Diagnostic Finding (`src/commands/doctor.rs`):**
  - Added audit probe in `ce-ai doctor` flagging missing Cursor `sessionStart` hooks in adopted projects containing `.cursor/`.
- **Documentation Updates:**
  - Updated `docs/user-guide/zero-step-drift-recovery-explained.md` promoting Cursor to native automated lifecycle hook delivery.

## [1.35.0] - 2026-09-02

### Added
- **Guaranteed Turn-0 Drift Delivery for Pi Coding Agent via Native Extension (`src/harness/pi.rs`, `src/commands/init_prj.rs`):**
  - Implemented `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` managing `.pi/extensions/compound-engineering.ts`.
  - Configures an in-process TypeScript lifecycle extension subscribing to `session_start` and `before_agent_start`, executing `ce-ai workflow resume` on Turn-0 and injecting live `RepoState` directly into `systemPrompt`.
  - Employs Turn-0 execution caching with `session_start` cache reset (for `/resume`, `/new`, `/fork` transitions) and fail-open error handling with timeouts to prevent disrupting agent loops.
  - Surgically deploys and removes extensions via `write_atomic`, preserving any user-defined extensions and pruning empty directories on project de-adoption.
- **Pi Coding Agent Doctor Diagnostic Finding (`src/commands/doctor.rs`):**
  - Added audit probe in `ce-ai doctor` flagging missing Pi `before_agent_start` extensions in adopted projects containing `.pi/`.
- **Documentation Updates:**
  - Updated `docs/user-guide/zero-step-drift-recovery-explained.md` promoting Pi coding agent to native automated extension delivery.

## [1.34.0] - 2026-09-02

### Added
- **Guaranteed Turn-0 Drift Delivery for OpenAI Codex CLI via Native `SessionStart` Hook (`src/harness/codex.rs`, `src/commands/init_prj.rs`):**
  - Implemented `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` managing `.codex/config.toml`.
  - Configures `[[hooks.SessionStart]]` command hook running `ce-ai workflow resume` matching `"startup|resume|compact"`, ensuring live `RepoState` injection at session start and mid-turn session compaction.
  - Surgically merges and removes hooks via `toml::Table` manipulation with `write_atomic`, preserving all pre-existing user settings (MCP servers, env vars, other hooks) and cleaning up empty configuration files upon de-adoption.
- **`hookSpecificOutput` in `workflow resume --json` (`src/commands/workflow.rs`):**
  - Added `hookSpecificOutput` JSON structure with `hookEventName: "SessionStart"` and `additionalContext` for full compatibility with OpenAI Codex CLI's structured output schema.
- **OpenAI Codex Doctor Diagnostic Finding (`src/commands/doctor.rs`):**
  - Added audit probe in `ce-ai doctor` flagging missing Codex CLI `SessionStart` hooks in adopted projects containing `.codex/`.
- **Documentation Updates:**
  - Updated `docs/user-guide/zero-step-drift-recovery-explained.md` promoting OpenAI Codex CLI to native automated hook delivery with compaction resilience.

## [1.33.0] - 2026-09-02

### Added
- **Guaranteed Turn-0 Drift Delivery for GitHub Copilot CLI via Native `sessionStart` Hook (`src/harness/copilot.rs`, `src/commands/init_prj.rs`):**
  - Implemented `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` managing `.github/hooks/hooks.json`.
  - Configures `sessionStart` command hook running `ce-ai workflow resume --json` cross-platform with both `bash` and `powershell` definitions.
  - Surgically merges and removes hooks via `write_atomic`, preserving all pre-existing user hooks (`preToolUse`, other `sessionStart` hooks) and cleaning up empty hook files upon de-adoption.
- **`additionalContext` Output in `workflow resume --json` (`src/commands/workflow.rs`):**
  - Enhanced `ce-ai workflow resume --json` payload with a top-level `additionalContext` string containing formatted status lines, satisfying GitHub Copilot CLI's context injection contract while preserving complete structured metadata for autonomous agents.
- **GitHub Copilot Doctor Diagnostic Finding (`src/commands/doctor.rs`):**
  - Added audit probe in `ce-ai doctor` flagging missing Copilot CLI `sessionStart` hooks in adopted projects containing `.github/`.
- **Documentation Updates:**
  - Updated `docs/user-guide/zero-step-drift-recovery-explained.md` promoting GitHub Copilot CLI to native automated hook delivery.

## [1.32.0] - 2026-09-02

### Added
- **Guaranteed Turn-0 Drift Delivery for OpenCode via Native Plugin Lifecycle Hooks (`.opencode/plugins/compound-engineering.js`, `src/opencode/plugins.rs`):**
  - Implemented canonical OpenCode plugin subscribing to the `session.created` event to automatically run `ce-ai workflow resume` and inject live `RepoState` via `client.session.prompt` with `{ noReply: true }`.
  - Added `experimental.session.compacting` hook ensuring canonical `RepoState` persists across context compaction.
  - Preserved full dynamic skill discovery and slash-command registration.
  - Embedded canonical plugin loader directly into the `ce-ai` binary (`BUILTIN_LOADER`), ensuring offline installation safety and eliminating external tarball release dependencies.
  - Implemented idempotent lifecycle helpers: `has_session_start_plugin`, `ensure_session_start_plugin`, and `remove_session_start_plugin`, guaranteeing atomic writes and non-destructive preservation of user plugins and settings in `opencode.json`.
- **OpenCode Doctor Health Diagnostics (`src/commands/doctor.rs`):**
  - Added diagnostic check alerting users when OpenCode is an installed harness but its `SessionStart` plugin is missing or outdated in `opencode.json`.
- **Documentation Updates:**
  - Updated `zero-step-drift-recovery-explained.md` and `harnesses-loops-and-context-masterclass.md` to document OpenCode's native plugin lifecycle event integration alongside Claude Code.

## [1.31.0] - 2026-09-02

### Added
- **Guaranteed Turn-0 Session-Start Drift Delivery (`src/harness/claude.rs`, `src/commands/init_prj.rs`):**
  - Native Claude Code `SessionStart` lifecycle hook support: `ce-ai init-prj` non-destructively injects a `SessionStart` command hook in `.claude/settings.json` running `ce-ai workflow resume`, injecting live `RepoState` into system context at session startup.
  - Surgical hook cleanup: `ce-ai deinit-prj` cleanly removes the hook without affecting custom user settings or other lifecycle hooks.
  - Universal Turn-0 directive: Added mandatory `ce-ai workflow resume` execution directive to the `AGENTS.md` managed block for instruction-driven harnesses (Cursor, Copilot, Codex, Kimi, Grok, Pi).
  - Adoption Block v4: Bumped `BLOCK_VERSION: u32 = 4`, enabling automated stale version detection and upgrade guidance in `ce-ai doctor` and `ce-ai status`.
- **Runtime Checkpoint & Doctor Health Gates (`src/commands/workflow.rs`, `src/commands/doctor.rs`):**
  - `ce-ai workflow checkpoint` runs `probe_repo_state()` and surfaces prominent non-blocking warnings if managed files drifted during the session.
  - `ce-ai doctor` audits adopted projects with `.claude/` directories to verify the `SessionStart` hook is configured.
- **Documentation Truthfulness & Realignment:**
  - Updated `zero-step-drift-recovery-explained.md` and `harnesses-loops-and-context-masterclass.md` to accurately delineate automated hook delivery vs prompt directives.
  - Replaced outdated placeholder caveat in `workflow-panel-native-vs-agent-skills.md` with the production `v1.30.0`+ live probing engine capabilities.

## [1.30.0] - 2026-09-02

### Added
- **Zero-Step Environment Drift Recovery via Live `RepoState` Sync (`src/commands/workflow.rs`, `src/commands/status.rs`):**
  - Added live, sub-15ms `RepoState` probe to `ce-ai workflow resume` and `ce-ai status`, inspired by SKILL.state findings (arXiv:2608.26263v2) demonstrating that long-horizon agents suffer 5–8 turns of lag under silent workspace changes.
  - Automatically captures and renders active git branch, HEAD commit SHA, uncommitted/untracked working tree modifications, managed plugin SHA256 manifest drift, and project adoption block status.
  - Emits structured `repo_state` object in `ce-ai workflow resume --json` for autonomous agent consumption on Turn 0.
  - Surfaces non-blocking drift warnings and guidance (`! Warning: Drift detected in managed files. Run 'ce-ai sync' to reconcile.`).
- **Adoption Block SSOT Integration:**
  - Integrated `check_adoption_block_status()` directly into `probe_adoption_status()` without duplicating cryptographic hash logic.
- **TDD Test Coverage:**
  - Added dedicated unit tests for `OpenSpecContextInfo` probing and task checkbox parsing.
  - Added dedicated unit tests for `TreeDrift` diff tracking and status calculation.
  - Added CLI integration tests verifying zero-step drift reflection during turn resumption.

## [1.29.2] - 2026-09-01

### Added
- **Zero-Friction Release Resolution with Web Redirect & Atom Feed Fallback (`src/source/release.rs`):**
  - Added public web redirect resolver (`https://github.com/{repo}/releases/latest`) and Atom feed parser (`https://github.com/{repo}/releases.atom`) as automatic fallbacks for `resolve_latest_release()`.
  - Eliminates rate limit errors (`HTTP 403 Forbidden`) for unauthenticated users without requiring `CE_AI_GITHUB_TOKEN`, personal access tokens, or `gh` CLI configuration.
  - Transparent hierarchy: tries authenticated REST API when credentials exist, but falls back seamlessly to web routes with zero friction.
- **Doctor Guidance Clarity (`src/commands/doctor.rs`):**
  - Updated token probe output to reflect that release resolution works out of the box with resilient web fallback when unauthenticated.

## [1.29.1] - 2026-09-01

### Fixed
- **Sync Verification Matrix for Native Harnesses (`src/commands/sync.rs`):**
  - Resolved false verification drift error (exit code 6) where `ce-ai sync` and `ce-ai upgrade` checked native harness directories for release skills that were intentionally not copied following the R4 token-neutrality adoption model.
  - Properly classifies registered, unadopted native harnesses as `registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)`.
- **Multi-Tiered GitHub Token Discovery (`src/source/release.rs`):**
  - Added support for `GITHUB_TOKEN` and `GH_TOKEN` environment variables alongside `CE_AI_GITHUB_TOKEN`.
  - Added automatic fallback to GitHub CLI (`gh auth token`) to prevent unauthenticated `HTTP 403 Forbidden` errors during `ce-ai upgrade` and `install`.
- **Doctor Token Probe Consistency (`src/commands/doctor.rs`):**
  - Unified token resolution across `doctor`, `install`, and `upgrade` via `resolve_github_token()`.

## [1.29.0] - 2026-08-29

### Changed
- **Test Architecture & Code Separation (Issue #265, PRs #266-#272):**
  - **Zero Inline Tests:** Extracted all `#[cfg(test)] mod tests { ... }` inline blocks across 36 functional source files into dedicated test files located under domain-specific `tests/` directories using Rust's explicit `#[path = "..."]` attribute.
  - **Domain Modular Layout:**
    - `src/state/tests/`: Isolated tests for `state.rs`, `diff.rs`, `profiles.rs`, `backups.rs`, and `mod_tests.rs`.
    - `src/opencode/tests/`: Dedicated tests for `config.rs`, `manifest.rs`, and `plugins.rs`.
    - `src/source/tests/`: Dedicated tests for `cache.rs`, `tools_registry.rs`, `registry.rs`, `release.rs`, and `archive.rs`.
    - `src/harness/tests/`: Dedicated tests for all 10 harness adapters (`agents.rs`, `pi.rs`, `claude.rs`, `copilot.rs`, `grok.rs`, `codex.rs`, `custom.rs`, `agy.rs`, `cursor.rs`, `fx.rs`, `kimi.rs`) and `mod_tests.rs`.
    - `src/tui/tests/`: Isolated headless and contract tests in `mod_tests.rs`.
    - `src/commands/tests/`: Dedicated tests for all subcommands (`upgrade.rs`, `tools.rs`, `audit.rs`, `models.rs`, `guard.rs`, `sync.rs`, `doctor.rs`, `init_prj.rs`, `workflow.rs`, `install.rs`).
  - **Zero Assertion / Behavior Drift:** Pure mechanical extraction preserving 100% of test fixtures, assertions, synchronization mutexes, and zero production runtime modification.
  - **Knowledge Capture:** Documented complete architectural patterns and tradeoffs in `docs/solutions/architecture/extract-inline-unit-tests-2026-08-28.md`.

## [1.28.0] - 2026-08-28

### Added
- **Pedagogical Guardrail Mode (`ce-ai guard`, Issue #114):**
  - **State Schema:** Added `GuardLevel` (`junior`, `strict`) and `GuardrailState` with backward-compatible deserialization in `state.json`.
  - **CLI Lifecycle Subcommands:** `ce-ai guard enable [--level junior|strict] [--harness <name>]`, `ce-ai guard disable`, and `ce-ai guard status [--json]`.
  - **Visibility & Governance:** Wired into `doctor` health checks and `status` summary for ISO/IEC 42001 and NIST AI RMF 1.0 Human-In-The-Loop compliance.
  - **Strategy Integration:** Registered in `Commands::Guard` within `src/commands/registry.rs`.
  - **Knowledge Capture:** Authored architectural solution document in `docs/solutions/architecture/pedagogical-guardrail-mode-lifecycle-2026-08-28.md`.

## [1.27.0] - 2026-08-28

### Changed
- **Transversal Maintainability Architecture (U1-U6):**
  - **Command Registry Pattern (`src/commands/registry.rs`):** Replaced hardcoded CLI matching in `src/main.rs` with `CeCommand` dispatch strategy.
  - **Harness Factory (`src/harness/mod.rs`):** Centralized multi-harness tool MCP registration and harness adapter dispatch, eliminating redundant per-harness branches.
  - **TUI Modular Decomposition (`src/tui/`):** Decomposed 1,791-line monolithic `src/tui.rs` into 6 single-responsibility modules (`app`, `handlers`, `render`, `runner`, `spawn`, `tabs`), each `<500` lines.
  - **Ports & Adapters Architecture (`src/state/ports.rs`):** Decoupled `StateStore` and `ConfigStore` traits from direct disk I/O, introducing thread-safe `InMemoryStateStore` and `InMemoryConfigStore` adapters for hermetic unit testing.
  - **Error Hardening & Zero-Panic Guarantee:** Eliminated all `.unwrap()` and `.expect()` calls in non-test source code; enforced crate-level `#![cfg_attr(not(test), deny(clippy::unwrap_used))]`.
  - **Knowledge Capture:** Documented full architectural decisions, patterns, and validation metrics in `docs/solutions/architecture/refactor-transversal-maintainability-2026-08-27.md`.

## [1.26.1] - 2026-08-27

### Fixed
- **TUI English (artifact contract):** `Sync`/`Upgrade`/`Status` panels to English + `ui_is_english_only` headless test (no Spanish).
- **opencode.json trailing comma:** `review-validator` `    },` → `    }` at line 135 — `sync` now `verified 393/393`, `doctor: ok`, journal cleared.
- **Onboarding token hint:** `README` Quick Path + `docs/user-guide/sync-and-upgrade-mechanisms.md` Step 1 + `doctor` `github-token` info + TUI `Upgrade` tip for first-time `403`.

## [1.26.0] - 2026-08-26

### Added
- **TUI paridad 9→15 (R1):** 5 tabs espejo `Skills/Tools/Usage/Audit/InitPrj` con `render_*` + `run_*_cmd` thin vía `capture_cli` — cada vector validado en `every_tui_spawned_vector` 15/15.
- **TUI estabilidad (R3-R6+I8):** `RawModeGuard` Drop restaura `disable_raw_mode`/`LeaveAlternateScreen` incluso en panic, input precedence `picker > modal > tabs`, modal `List+Scrollbar` con `j/k/PgUp/PgDn`, gate `j/k` context-aware, `stdout.is_terminal()` guard + banner `state.json corrupt` en Status.
- **Headless E2E zen:** `headless_ui_renders_all_tabs` + `headless_screenshots_no_overflow` (`TestBackend 80×24`, dumps a `tui-screenshots/` gitignored, slop 84 para emoji wide), `e2e_runner.sh` TUI headless checks con `opencode/zen-free` fallback y `cargo test tui` soft, `Dockerfile.e2e` mantiene `opencode-ai`.

### Changed
- **Upgrade honesto (R2):** `Upgrade` sin selector harness (global, `run_upgrade_cmd` ignora `selected_harness_target`) — cierra mentira UI.
- **Modal scroll + TTY:** `render_modal` ahora scrolleable con indicador `↑ scrolled`.

### Fixed
- **E2E broken pipe:** `e2e_runner.sh` evita `| grep` directo (redirige a `/tmp/*.txt` antes de `grep`) y no reinstala en E2E 8 para no sobrescribir backup.

## [1.25.0] - 2026-08-26

### Added
- **Canonical skills adoption**: `ce-ai skills adopt` puts pre-existing `ce-*` skill copies under ce-ai management — transactional rewrite with per-file backup, partial-set completion to the full canonical inventory, and retirement of superseded managed copies — while user-authored `ce-*` skills and symlinked entries are never touched. Adoption is recorded in a `skill_surfaces` ledger; the transactional engine and ledger-scoped uninstall shipped together (no destructive window).
- **Sync currency for adopted surfaces**: `sync` rewrites adopted-surface drift back to canonical (reported as `restored-drift`), respects managed-copy retirement (no re-harvest), and the verification matrix renders the adoption lifecycle: `verified N/N` per adopted surface, `pending-adoption`, `external-duplicate`, and `orphaned`.
- **Registry + visibility**: the SkillRegistry indexes adopted surfaces for all harnesses (resolution works for copy-less harnesses via canonical paths); `status` and `doctor` surface adoption states, including orphaned surfaces requiring re-adoption. User guide gains the canonical/adoption model (R11).

### Changed
- **Harvest the real release layout**: `install`/`sync` harvest the release's top-level `skills/` tree (top-level wins over the legacy `.opencode/skills` prefix, overlap warns). Harvested skills are never copied into harness-owned directories — adoption is the only delivery path, keeping one indexed skill set per harness (token economy: duplicated skill descriptions are a per-session cost).

### Measurement
- Token-overhead tracking (origin success criterion): on the reference machine, the pre-change state indexed 35 stale `ce-*` skill descriptions (single user-dir set; the managed dir held no tracked skills). Post-adoption: the 33-skill canonical set is indexed once, refreshed to the release version, with 12 user-authored `ce-*` skills preserved untouched and the managed-dir duplicate prevented by retirement. Net: descriptions went 35 stale → 33 current with zero duplication, and the design structurally prevents the managed+user double-indexing that motivates this feature.

## [1.24.0] - 2026-08-25

### Changed
- **Retention-default SSOT rule (adoption block v3)**: the Single Source of Truth rule now states explicitly that ideation artifacts (`docs/brainstorms/`, `docs/ideation/`) are retained by default as the permanent raw-history record OpenSpec deliberately does not duplicate — "disposable inputs" means *not maintained in sync*, never a deletion mandate. Applied to root `AGENTS.md` and both managed-block surfaces (`full` tier SSOT paragraph, `orchestrator` tier line); the managed block version bumps to `v=3`. Existing `full`/`orchestrator` adoptions keep working but will report **stale block version** hints in `ce-ai status` / `ce-ai doctor` until `ce-ai init-prj <project> --tier <tier>` is re-run — re-adoption replaces the block in place, preserving all surrounding content and requiring no migration step. Existing `minimal`-tier adoptions classify as healthy without hints (their block body is byte-identical across these versions; SHA match short-circuits the version check).

## [1.23.1] - 2026-08-24

### Changed
- **Sync Verification Matrix UX clarity**: surfaces where ce-ai manages no files now read `○ <harness>: registered — <reason>` instead of the self-contradictory `synced — verification not performed`, and the reconciliation summary counts them as `N registered (nothing to verify)` instead of `N unverified`, so an expected state no longer reads like an error. When any surface is unverified, a guidance note explains what `registered` means, states that CE installed via other channels (plugin marketplaces, manual copies) is outside ce-ai's verification scope, and points to `ce-ai install --harness <name>` to put a harness under management. Matrix rendering is extracted into pure, unit-tested functions pinning the exact wording. User guide Step 6 updated with the new output, the three verification states, and a per-harness management table.

## [1.23.0] - 2026-08-24

### Changed
- **Deterministic source resolution**: `install`/`upgrade` no longer fall back to the mutable `main` branch tarball. Network, HTTP, and payload failures now exit with code 5 (`Network`), and a release list with zero `compound-engineering-v*` tags exits with code 2 (`Usage`) — both with actionable guidance to pin via `--to <tag>` or install a local tree via `--source <path>`. Scripts relying on the silent offline fallback must migrate to pinned tags.
- **Byte-stable skill resolution output**: `skills resolve` markdown no longer embeds a wall-clock timestamp in its header; identical registry state produces byte-identical output. The machine-readable `status=` tag is unchanged.

### Added
- Beginner-level explanation doc [`docs/user-guide/determinism-explained.md`](docs/user-guide/determinism-explained.md): what determinism means, the asset-layer guarantees of ce-ai, why LLM-driven workflow execution cannot be made deterministic by any technical means today, environment-relative behaviors, and compensating controls.

## [1.22.2] - 2026-08-24

### Changed
- Precision pass on the PR size policy (verified against upstream sources): the 400-line boundary is enforced by Gentle AI's **Review Workload Guard** inside `internal/assets/hermes/sdd-orchestrator.md` — a delivery-policy split signal (single PR vs chained PRs; strategies `ask-on-risk`/`auto-chain`/`single-pr`/`exception-ok`; chains via `stacked-to-main`/`feature-branch-chain`) — never a cap on what an agent may write. CONTRIBUTING.md now cites the orchestrator asset by path alongside the runtime constants.

## [1.22.1] - 2026-08-24

### Added
- Contributor policy for PR size boundaries and bounded corrections (Issue #108): 400-line review boundary with a counting contract (`--numstat`, lockfile/generated/binary exclusions), correction budget of `min(200, ceil(original/2))` per cycle with a single bounded correction, ~200-line work-unit budgets in OpenSpec `tasks.md` (rescopes narrow only), and the size-is-not-risk principle — adopted from Gentle AI v2.4.0 constants, documented in the new CONTRIBUTING.md policy section.
- PR template now requires a Changed-Lines Forecast; AGENTS.md Stage-2 checklist references work-unit budgets; docs-styling links the sibling policy.

## [1.22.0] - 2026-08-24

### Added
- Transactional multi-file operations (Issue #166): install and sync route every tracked filesystem mutation through a durable operation journal (`<config-dir>/install-journal.json`) that records prior content **before** each write; a crashed or failing run is detected by `ce-ai doctor` (`install-journal:` finding) and deterministically rolled back in reverse by the next install/sync before proceeding fresh. Fault injection via `CE_AI_FAIL_AFTER_WRITES=<N>` makes the N+1 tracked write fail; integration tests cover early and mid-sequence cuts proving user content survives without loss.
- Ordering guarantee enforced and documented: `state.json` remains the final persisted mutation of every command, with the journal cleared only after it succeeds.

## [1.21.6] - 2026-08-24

### Changed
- Homebrew distribution consolidated into the self-updating [`mastepanoski/homebrew-ce-ai`](https://github.com/mastepanoski/homebrew-ce-ai) tap: the redundant in-repo `Formula/ce-ai.rb` and the automated formula-update PR were removed. The tap's `self-update` workflow (30-minute cadence) is the single source of truth, eliminating the dual-state drift observed between consecutive releases.
- The `release-integrity` job now publishes only `SHA256SUMS.txt`; `scripts/release-integrity.sh` was simplified accordingly (digest verification of the six release assets remains fail-closed).

## [1.21.5] - 2026-08-24

### Fixed
- TUI "Upgrade Release" no longer fails with `unexpected argument '--harness'`: the panel still spawned the flags removed in v1.18.1 (Issue #161). It now invokes plain `ce-ai upgrade`, which reconciles every active harness by design, and drops the obsolete "Target Harness Scope" line.

### Changed
- Anti-drift net: all nine TUI-spawned command vectors were extracted into pure builders and are unit-validated against their live clap subcommand surfaces (`augment_args` + globals modeling). Any future CLI contract change now breaks TUI tests instead of failing at runtime for users.

## [1.21.4] - 2026-08-24

### Fixed
- Verification gates now verify (Issue #165): `make e2e` is fail-closed — Docker absence, an unreachable daemon, or a Windows host is a hard `FAILED-TO-RUN` failure with remediation guidance instead of a silent `SKIPPED` pass; and a new `make security` target installs cargo-audit if missing (one-time, `--locked`) then fails on any advisory, mirroring the CI Supply Chain job locally.

### Changed
- The CI Supply Chain Security Audit job tees its full output into the GitHub step summary (`set -o pipefail` preserved) so a green run always means the audit actually executed.

## [1.21.3] - 2026-08-24

### Added
- Release integrity automation (ISO/IEC 27002 integrity control): a `release-integrity` job now runs after every build, publishes `SHA256SUMS.txt` alongside the release binaries so users can verify downloads independently, and opens an automated PR repointing the Homebrew tap formula at the new tag with computed checksums — eliminating the manual-edit drift that let it fall eleven versions behind.
- `scripts/release-integrity.sh`: deterministic, locally runnable generator (`TAG_NAME=vX.Y.Z ./scripts/release-integrity.sh`) that fails closed on missing assets, malformed digests, or invalid formula syntax. Verified to reproduce the v1.21.2 formula byte-for-byte.

## [1.21.2] - 2026-08-24

### Changed
- CI and release pipelines moved off the deprecated Node.js 20 runtime: `actions/checkout` v4→v7, `actions/cache` v4→v6, `actions/github-script` v7→v9, `docker/setup-buildx-action` v3→v4, `actions/upload-artifact` v4→v7, and `softprops/action-gh-release` v2→v3 — every target verified to run natively on Node 24.
- Homebrew formula repaired: per-platform URLs pinned to stale release assets (v1.10.0 binaries / v1.18.1 source) now track the current tag with SHA256 integrity checksums for every asset.

## [1.21.1] - 2026-08-24

### Added
- Community health files per the [opensource.guide](https://opensource.guide/) standards checklist: `.github/PULL_REQUEST_TEMPLATE.md` (DoD verification gates, Conventional Commits, OpenSpec requirement, SemVer + CHANGELOG bumps, AI-assisted contribution disclosure), referenced from `CONTRIBUTING.md`.
- `.github/ISSUE_TEMPLATE/config.yml`: blank issues disabled; contact links route security reports to Private Vulnerability Reporting and general questions to Discussions.

### Changed
- Security disclosure is now private-only: the public `security_report.yml` issue form was removed because it exposed proof-of-concept exploit details before a patch exists. `SECURITY.md` documents Private Vulnerability Reporting as the primary channel with an email fallback (`me@maurostepanoski.ar`).
- Version metadata repaired after v1.21.0 shipped without its `Cargo.toml` bump: the package now reports 1.21.1 and the Homebrew formula carries a single `version` stanza again.

## [1.21.0] - 2026-08-24

### Added
- `ce-ai doctor` branch-protection health probe (final requirement of `context_exhaustion_resilience`): resolves the GitHub `owner/repo` slug from `origin`, queries `branches/main/protection` via `gh`, and raises a finding when `main` is verifiably unprotected or missing required status checks — the exact gap that forced admin-bypass merges during recent cycles. Non-GitHub remotes and an unavailable `gh` degrade to info notices; absent PR-review requirements surface as a single-developer advisory instead of a failure (matching `scripts/protect-branch.sh`).
- OpenSpec ledger completed: `context_exhaustion_resilience` fully implemented, tasks ticked with documented deviations, and archived — the active change folder count is now zero.

## [1.20.4] - 2026-08-24

### Fixed
- Completed the exit-code contract (invariant #7 / Issue #163): new `CeError::State` (exit **3**) raised from corrupt/unreadable/unpersistable `state.json`, `CeError::Network` (exit **5**) from propagating GitHub tarball transport failures, and filesystem I/O remapped to its own exit **4** instead of folding into runtime(1). Verification(6) existed since v1.18.1.
- Performance bounds are executable again (Issue #84): new `make bench` target runs `benches/benchmarks.rs` (`<50ms` state-resolution and manifest-roundtrip assertions) under `cargo test --benches --release`; the stale hardcoded `0.9.0` version literal is now derived from `CARGO_PKG_VERSION`.

## [1.20.3] - 2026-08-24

### Fixed
- Cursor no longer receives a skills-tree copy during install or sync (`~/.cursor/skills` pollution introduced by the v1.19.2 table): the shared registration spec now marks cursor as MCP-only, matching its documented `mcp.json` + rules surface.

### Changed
- Internal: the registration strategy table moved to `src/harness/registration.rs` and is now shared by both `install` and `sync` (single source of truth; ~230 duplicated lines removed from install). OpenCode remains the sole consumer of the plugin/skills JSON merge; the dead legacy `.opencode/skills` fallback path in install arms is gone.
- Dead-code honesty pass: every `#[allow(dead_code)]` removed (including the three module-wide suppressions in `opencode`, `source`, and `state`). Truly unused items were deleted — `AuditStatus::Fail` + `fail_count` (no detector emits failures), `CodexMcpServer`, `CursorAdapter`, `OpencodeAdapter`, cursor's `strip_managed_block`, `apply_model_assignment`, `list_snapshots`, three speculative `State` methods, TUI `LOGO`, and the trait's never-read `canonical_instruction_file`/`derived_stub_files` defaults with their per-vendor overrides.
- Wired the previously dormant `.ce-ai.json` workspace-override loading: `Context` now carries `workspace_root`, and model-assignment readers (`models`, `status`, `sync`, `doctor`) resolve state through `load_with_workspace_overrides` as documented in CONCEPTS.md.

## [1.20.2] - 2026-08-24

### Changed
- Documentation truth sweep: Stage 5 (Verification) is no longer documented as `cargo test`/`make e2e` — the CE workflow is language-agnostic, so the three user guides (`quick-start-workflow-guide`, `fsm-and-checkpoints-explained`, `compound-engineering-workflow-explained`) and AGENTS.md now define it as running the project's own quality gates, with Rust commands demoted to stack-specific examples.
- README harness-matrix pointer corrected from "all 12 harnesses" to "all supported harnesses (10 native + custom)"; DeepSeek remains de-scoped and is not a supported target.
- Refreshed `docs/solutions/multi-harness-support-implementation.md`: replaced the v0.3.0-era architecture description (fictional `generic_json.rs` coverage, DeepSeek listed as supported) with the current native-adapter + registration-spec reality.

## [1.20.1] - 2026-08-24

### Changed
- OpenSpec ledger hygiene: established `openspec/changes/archive/` with a written convention and triage table; 51 completed folders archived (35 fully-checked, 16 STATUS-annotated with feature-level ship evidence), the three changes shipped by PRs #205–#207 now have their tasks ticked, and `context_exhaustion_resilience` remains active as the only folder without observable ship evidence.

### Fixed
- Best-effort cleanup failures are no longer silently swallowed (invariant #5): `deinit-prj` stub removals/rewrites (18 sites), `init-prj` legacy migrations (3), transient tarball temp dirs (install/upgrade), skill-registry sync (install/sync) and the `sync --watch` Ctrl-C handler registration now emit stderr warnings naming path and cause, while `NotFound` stays silent; custom-mode root pruning keeps its intentional silence with a justification comment.

## [1.20.0] - 2026-08-24

### Added
- Real 7-stage workflow FSM engine (Issue #156): `ce-ai workflow` now stores a strongly-typed `WorkflowStage` state (`state.workflow` in `state.json`) with legal-transition enforcement (advance one stage, stay, rewind, or reset to Stage 1; illegal jumps fail with exit code 2), `--stage/--task/--feature` checkpoint flags, `--json` output for `status/checkpoint/resume`, and OpenSpec context recovery on `resume`. Legacy `last_update_check` checkpoint strings are still parsed transparently.
- Real long-running `sync --watch` loop (Issue #159): the watcher no longer exits after a single pass — it polls for drift, repairs it automatically, reports each repair with a timestamp, handles Ctrl-C gracefully via a `Once`-registered signal handler, and supports `--interval-ms` (default 2000) and `--max-passes` (for scripted/tested runs).
- Dry-run zero-mutation contract for remote operations (Issue #160): `--dry-run install/upgrade` now extract remote tarballs into transient temporary directories instead of persisting them into the cache, and `--dry-run workflow checkpoint` no longer writes `state.json`.
- Triple-directory snapshot test helper (`assert_dry_run_zero_mutation`) covering `config_dir`, `home_dir`, and workspace to pin the dry-run purity contract in CI.

### Fixed
- `sync --watch` drift repair is now reported on every repairing pass, including the initial pass, so watchers surface repairs immediately instead of silently fixing before entering the loop.

## [1.19.2] - 2026-08-24

### Changed
- Internal: `ce-ai sync` re-registration collapsed into an exhaustive strategy table (`registration_spec`). Adding a `HarnessKind` variant is now a compile error until it is classified in the table or given a dedicated arm — the fictional-write bug class introduced by forgotten arms becomes structurally impossible on this surface. No behavior change: the 94-test black-box CLI suite passes untouched.

## [1.19.1] - 2026-08-24

### Fixed
- `ce-ai sync` no longer routes Kimi, Antigravity (`agy`), fx, and Pi through the generic OpenCode-format branch: each now re-registers exactly like `install`. Previously a sync could inject OpenCode-only `plugin`/`skills.paths` keys into their native config files (`~/.kimi-code/mcp.json`, `~/.gemini/config/mcp_config.json`, `~/.fx/mcp.json`) — corrupting native MCP definitions.
- Managed-skills copies during sync propagate IO errors instead of being swallowed; the Claude arm's silent `let _ =` copy is gone (invariant #5).
- An unsupported harness name found in `state.json` now fails sync with a named `Runtime` error instead of silently receiving fabricated OpenCode-format mutations.
- The post-sync verification matrix hash-checks the skills surfaces of all eight directory-copying harnesses (`claude`, `codex`, `copilot`, `grok`, `kimi`, `agy`, `pi`, `fx`; agy under `config/skills`) instead of labelling four of them "not verified".

### Changed
- Bookkeeping: reality-note annotations on `openspec/changes/multi_harness_support/tasks.md` Tasks 2.5/2.6 correcting stale pointers and `generic_json.rs` history (docs-only).

## [1.19.0] - 2026-08-24

### Added
- Real `--harness custom` fallback mode (R4 of `multi_harness_support`): `install --harness custom --plugins-dir <dir> --skills-dir <dir> [--rules-file <file>]` copies managed CE plugin assets and skill folders into user-configured directories, records a SHA256 install manifest under `<plugins-dir>/compound-engineering/`, and snapshots the resolved configuration inside the `state.json` entry.
- Persisted custom-mode configuration at `~/.ce-ai/custom_harness.json` (`plugins_dir`, `skills_dir`, optional `rules_file`) with flag-over-file precedence, `~` expansion, and a fast-fail usage error (exit code 2) when unresolvable.
- Managed CE block injection into the configured rules markdown file (idempotent; user content preserved verbatim) with surgical uninstall: removes exactly the manifest-recorded files, prunes emptied CE-owned directories, and strips the block while keeping every other byte.

### Changed
- `sync` re-copies and SHA256-verifies custom-mode plugin/skill surfaces in its verification matrix and preserves directory snapshots across state rebuilds.
- Single path contract for custom mode: `~/.ce-ai/custom_harness.json`; the fictional `~/.config/custom/custom.json` default and the dead `~/.custom/config.json` adapter were removed.

### Fixed
- `install`/`uninstall --harness custom` no longer fall through the OpenCode-format branch writing fabricated `{plugin, skills.paths}` JSON into `~/.config/custom/custom.json`.

## [1.18.1] - 2026-08-24

### Changed
- `ce-ai upgrade` no longer accepts the never-implemented `--harness`/`-t` and `--force`/`-f` flags; clap now rejects them as unknown arguments with a usage error (exit code 2) instead of silently ignoring them (Issue #161).
- `ce-ai sync` (and upgrade-triggered sync) now reports only checks that actually ran per harness: the OpenCode managed surface is re-hashed after apply, harness skill copies are hash-checked when performed, and registration-only adapters are explicitly labelled as not verified; the fabricated `100% Verified` matrix is gone.

### Fixed
- `upgrade --to <tag>` is bound to recorded release provenance `{tag, url, archive_sha256, extraction_path}` persisted atomically in `state.json`: a tag mismatch fails with a precise usage error and a tampered/corrupted cached archive fails closed with a verification error naming expected vs actual SHA256 — a cached release can never be relabelled as a different requested version (Issue #161).
- Added `CeError::Verification` mapped to exit code 6, completing the standardized exit-code contract (`0/1/2/3/4/5/6`); post-sync verification drift now propagates to the process exit status.

## [1.18.0] - 2026-08-24

### Changed
- Reconciled `README.md` and `openspec/changes/multi_harness_support/spec.md` to state 10 native supported AI agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`) and detail native directory locations (Issues #155, #183).
- De-scoped `deepseek` cleanly: CLI invocations specifying `deepseek` return `CeError::Usage` (exit code 2) explaining that `dsh` uses `~/.dsh` YAML patch layers during developer preview, and excluded `deepseek` from host harness auto-detection (Issue #180).
- Renamed audit output header, `--fail-under` docstring, and threshold error message in `src/commands/audit.rs` to `configuration coverage` (Issue #164).

### Fixed
- Hardened `resolve_latest_release` in `src/source/release.rs` to catch network send errors and HTTP 403 / 429 rate limit responses from the GitHub API, logging an informative notice to stderr and falling back to the `main` branch source tarball (`main_tarball_url()`, SF-2) without hard-failing (Issue #202).

## [1.17.2] - 2026-08-24

### Fixed
- Removed filesystem-dependent `home.join("mcp.json").exists()` check in `FxAdapter::default_config_path` in `src/harness/fx.rs`, ensuring deterministic path resolution based strictly on basenames.
- Propagated IO errors on `std::fs::remove_file` in `unregister_fx_mcp_server` while ignoring `ErrorKind::NotFound`.
- Documented `$FX_HOME` environment variable override as a `ce-ai` extension convention in OpenSpec `design.md`.
- Documented extra map `type` collision cleanup in OpenSpec `design.md`.
- Expanded unit tests in `src/harness/fx.rs` covering deterministic path resolution when `$HOME/mcp.json` pre-exists, extra map `type` purging on re-registration, and IO error propagation.

## [1.17.1] - 2026-08-24

### Fixed
- Added Serde alias `url` for `server_url` in `AgyMcpServer` and ensured `register_agy_mcp_server` cleans stale remote keys (`url`, `serverUrl`, `headers`, `transport`) from `extra` map on server name collision.
- Used cross-platform path joining (`PathBuf::from(".agents").join("rules").join("compound-engineering.md")`) in `AgyAdapter::derived_stub_files`.
- Documented `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_HOME` as `ce-ai` extension conventions in OpenSpec `design.md`.
- Expanded unit tests in `src/harness/agy.rs` covering environment variable precedence, serverUrl reset, and OpenCode key exclusion.

## [1.17.0] - 2026-08-23

### Added
- Native harness adapter implementation for Vercel Labs' `fx` coding agent (`FxAdapter`, Issue #181).
- Native MCP server registration in `~/.fx/mcp.json` (and `$FX_HOME` override) with root key `mcp`, `"type": "local"`, array-form command syntax (`["codegraph", "mcp"]`), and environment map.
- Skills installation under `~/.fx/skills/`.
- Project rule adoption targeting `AGENTS.md` (root) and `.fx/AGENTS.md` (derived stub).

## [1.16.0] - 2026-08-23

### Added
- Native harness adapter implementation for Mario Zechner's `pi` coding agent (`PiAdapter`, Issue #182).
- Native asset management under `~/.pi/agent/skills/` (and environment override `$PI_CODING_AGENT_DIR`), eliminating fictional MCP JSON or OpenCode plugin config file generation.
- Project rule adoption targeting `.pi/AGENTS.md` when `.pi/` directory pre-exists.
- Informative skip handling for `pi` targets in `ce-ai tools install` (reporting `pi`'s native no-MCP philosophy by design).

## [1.15.2] - 2026-08-23

### Fixed
- Documented `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` as `ce-ai` extension conventions for custom directory relocation in Google Antigravity OpenSpec design.
- Documented project rules architecture (`GEMINI.md` canonical instruction file and `.agents/rules/compound-engineering.md` derived stub file).
- Formally specified server registration name collision policy in `register_agy_mcp_server` where stdio registration explicitly resets `server_url` to `None`.

## [1.15.1] - 2026-08-23

### Fixed
- Updated Kimi Code CLI project rule adoption target to `.kimi-code/AGENTS.md` (official Kimi Code CLI instruction file path), cleaning up legacy `.kimi-code/rules/compound-engineering.md` and empty `rules/` directories on deinit.
- Extracted neutral managed rule block update and strip helpers (`update_managed_rule_md`, `strip_managed_rule_block`, `CE_MANAGED_BEGIN`) in `src/harness/mod.rs` to decouple adapters.
- Removed stale doc comments mentioning Kimi and Antigravity in `src/harness/generic_json.rs`.

## [1.15.0] - 2026-08-23

### Added
- Native harness adapter for Google Antigravity (`agy`, Issue #179) writing to `~/.gemini/config/mcp_config.json` (`mcpServers` JSON object with `serverUrl` preservation) and `~/.gemini/config/skills/`.
- Project rule adoption for Google Antigravity under `.agents/rules/compound-engineering.md` and `GEMINI.md`.
- Environment variable override `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` support.

## [1.14.0] - 2026-08-23

### Added
- Native harness adapter for Kimi Code CLI (`kimi`, Issue #178) writing to `~/.kimi-code/mcp.json` (`mcpServers` JSON object) and `$KIMI_CODE_HOME/skills/`.
- Project rule adoption for Kimi under `.kimi-code/rules/compound-engineering.md` and `AGENTS.md`.
- Environment variable override `$KIMI_CODE_HOME` support.

## [1.13.2] - 2026-08-23

### Fixed
- Fixed flaky test environment race condition on `GROK_HOME` by introducing cross-module process-wide mutex guard `HARNESS_ENV_LOCK` across `grok.rs` and `mod.rs` unit tests.
- Removed dead legacy `HarnessKind::Grok` generic JSON mapping (`.grok/config.json`) from `src/harness/generic_json.rs`.
- Expanded unit tests in `src/harness/grok.rs` for edge cases and invalid TOML error handling, and in `src/harness/generic_json.rs` for all supported variants.

## [1.13.1] - 2026-08-23

### Fixed
- Fixed JSON `env` object replacement in `register_copilot_mcp_server` to cleanly remove the `env` key when `env` map is empty.
- Emitted explicit stderr warnings on skills directory removal failures during `ce-ai uninstall` across native harnesses.
- Documented `COPILOT_CONFIG_DIR` environment override convention in OpenSpec design contract.

## [1.13.0] - 2026-08-23

### Added
- Native xAI Grok Build CLI (`grok`) harness adapter (`~/.grok/config.toml` TOML schema, `~/.grok/skills/`, `.grok/rules/compound-engineering.md`, and `$GROK_HOME` support) (Issue #176).

## [1.12.1] - 2026-08-23

### Fixed
- Adopted official `CODEX_HOME` environment variable override across Codex harness resolution (`src/harness/mod.rs`, `src/harness/codex.rs`).
- Fixed TOML `env` table replacement in `register_codex_mcp_server` to cleanly overwrite or remove `env` entries.
- Removed dead legacy generic JSON mapping for Codex (`.codex/config.json`) in `src/harness/generic_json.rs`.
- Clarified OpenSpec R3 contract for `.codex/AGENTS.md` project adoption vs root `AGENTS.md`.

## [1.12.0] - 2026-08-23

### Added
- Native GitHub Copilot CLI harness adapter (`src/harness/copilot.rs`) targeting `~/.copilot/mcp-config.json` (JSON format) with `mcpServers` schema (Issue #177).
- Native skills placement under `~/.copilot/skills/` and project directives adoption under `.github/copilot-instructions.md`.
- `COPILOT_CONFIG_DIR` environment variable support.
- Full lifecycle support for Copilot across `install`, `tools install`, `init-prj`, `deinit-prj`, `sync`, `doctor`, `status`, and `uninstall`.

## [1.11.0] - 2026-08-23

### Added
- Native OpenAI Codex CLI harness adapter (`src/harness/codex.rs`) targeting `~/.codex/config.toml` (TOML format) with `[mcp_servers.<name>]` schema (Issue #175).
- Native skills placement under `~/.codex/skills/` and project directives adoption under `.codex/AGENTS.md`.
- `CODEX_CONFIG_DIR` environment variable support.
- Full lifecycle support for Codex across `install`, `tools install`, `init-prj`, `deinit-prj`, `sync`, `doctor`, `status`, and `uninstall`.

## [1.10.0] - 2026-08-23

### Added
- **Native Claude Code Harness Adapter (Issue #174 / Umbrella #155)**:
  - Native `~/.claude.json` / `~/.claude/settings.json` reader and writer using Claude's official `mcpServers` stdio schema (`type: "stdio"`, `command`, `args`, `env`), honoring `CLAUDE_CONFIG_DIR` when set.
  - Zero OpenCode key leakage: `plugin` and `skills.paths` keys are never written to Claude Code configuration files.
  - Native skills placement: Managed skills installed directly under `~/.claude/skills/<name>/SKILL.md`.
  - Project rules support: `ce-ai init-prj` writes project rules to `CLAUDE.md` / `.claude/CLAUDE.md` with demarcated managed comment blocks and de-adoption (`deinit-prj`) cleanup.
  - Full lifecycle support for Claude Code across `install`, `sync`, `tools install`, `init-prj`, `deinit-prj`, and `uninstall`.

---

## [1.9.0] - 2026-08-23

### Added
- **Native Cursor Harness Adapter (Issue #173 / Umbrella #155)**:
  - Native `~/.cursor/mcp.json` reader and writer using Cursor's official `mcpServers` stdio schema (`type: "stdio"`, `command`, `args`, `env`), preserving all unmanaged user MCP entries and per-server custom attributes.
  - Zero OpenCode key leakage: `plugin` and `skills.paths` keys are never written to `~/.cursor/mcp.json`.
  - Project rules support: `ce-ai init-prj` writes project rules to `.cursor/rules/compound-engineering.mdc` with valid frontmatter (`description`, `globs`, `alwaysApply`) and demarcated managed comment blocks.
  - Full lifecycle support for Cursor across `install`, `sync`, `tools install`, `init-prj`, and `uninstall` with byte-for-byte user content preservation.

---

## [1.8.4] - 2026-08-23

### Fixed
- **Transactional Error Propagation & State Commit Integrity (Issue #162)**: Replaced `let _ =` error swallowing in `uninstall`, `deinit-prj`, and `init-prj` with `?` error propagation for required filesystem operations (file deletion, backup restoration, atomic writes, `.gitignore` block updates). Delayed `state.save()` to run strictly after required filesystem work succeeds, preventing state corruption and false-positive completion messages when filesystem operations fail.

---

## [1.8.3] - 2026-08-23

### Fixed
- **Code Review Hardening for Sidecar Probe & JSON Guarding (Issue #158)**: Refactored `install_tool` to probe actual binary availability on `PATH` (`is_in_path`) avoiding tautological false-positive success reporting when executables are missing. Added `config.is_object()` validation in `register_mcp_server` to prevent runtime panics on non-object JSON roots.

---

## [1.8.1] - 2026-08-23

### Fixed
- **Atomic Tools Install Sidecar Registration (Issue #158)**: `ce-ai tools install <tool>` now performs atomic JSON config injection (`mcpServers.<tool>`) into `opencode.json` using `write_atomic`, preserving all pre-existing user MCP servers. Executes a mandatory post-install health probe, returning a non-zero exit code (`CeError::Runtime`) on failure and emitting NO false success messages.

---

## [1.8.0] - 2026-08-23

### Added
- **Multi-Harness Token Efficiency & Context Quality Audit Subcommand (Issue #117)**: Introduced `ce-ai audit`, a capability-based audit engine evaluating token efficiency (CLI output compression with `rtk`, MCP server sprawl thresholds, prompt duplication analysis) and context quality (persistent memory with `engram`, library docs provider with `context7`, code intelligence index with `.codegraph/`, and learnings library with `docs/solutions/`).
- **Audit CLI Ergonomics**: Renders a categorized console output with an overall percentage score (e.g. `score: 78%`). Purely advisory by default (Exit 0); supports `--json` for machine-readable CI reports and `--fail-under <pct>` to enforce threshold gates.

---

## [1.7.1] - 2026-08-23

### Fixed
- **Adoption Block Staleness Alignment (Issue #149)**: `ce-ai status` now surfaces actionable upgrade hints (`STALE BLOCK v=<ver> — re-run ce-ai init-prj --tier <tier> to upgrade`) matching `ce-ai doctor` when a managed block in `AGENTS.md` is on an older version (`v < BLOCK_VERSION`). Extracted a single source of truth helper `check_adoption_block_status` in `src/commands/init_prj.rs`.

---

## [1.7.0] - 2026-08-22

### Added
- **Companion-Tool Readiness & Version Freshness Engine (Issue #112)**: `ce-ai doctor` and `ce-ai tools status` now act as a proactive environment readiness engine, validating installed version freshness against an embedded tools registry (`ToolsRegistryCache`) backed by a local 24-hour TTL cache (`~/.ce-ai/cache/companion-registry.json`).
- **Resilient Exit Code Rules & `--strict` Flag**: Added `--strict` flag to `ce-ai doctor`. Outdated tools emit informational hints (`doctor-info:`) without failing `doctor` (Exit 0) by default; passing `--strict` fails `doctor` with non-zero exit code if any companion tool is missing or outdated.
- **Skill Suggestions & Self-Update Hints**: Added skill presence probes (e.g. `sequential-thinking` in Skill Registry) and orchestrator self-update notifications in `tools status` and `doctor`.
- **Graceful Offline Degradation**: Offline or timed-out network checks fall back smoothly to local TTL cache or embedded defaults, emitting `(offline)` tags without failing process exit codes.

---

## [1.6.0] - 2026-08-22

### Added
- **TUI Workflow Panel — Native Action Execution (Issue #76)**: `[Enter]` on the Workflow tab now renders the **real** `ce-ai workflow status` output in the result modal, replacing the previous canned success message. Workflow commands (`status`, `checkpoint`, `resume`) were refactored to return output lines so CLI and TUI share a single source of truth.
- **Failure-Class Modal**: Native command failures (`CeError`) render as a distinct failure block with an actionable remedy hint instead of failing silently.
- **Native-vs-Skill Guide Markers**: Stage rows now carry text markers — `[run]` for actions executable inside the dashboard, `skill:` for agent-session stages with their mapped skills — perceivable without color support. The Verify stage copy is tech-neutral ("your project's test/e2e commands").
- **Teacher-Style Docs**: New explanation page [Workflow Panel: Native vs Agent Skills](docs/user-guide/workflow-panel-native-vs-agent-skills.md) covering why agent stages are guide-only.
- **OpenSpec Specification**: Documented under `openspec/changes/tui_workflow_stage_exec/`.

### Changed
- **Resume keybinding deliberately excluded**: `workflow resume` is currently print-only and adds nothing over `[1-7]` stage checkpoints; binding it would show false success. Real checkpoint-based recovery is recorded as a candidate follow-up.

---

## [1.5.0] - 2026-08-22

### Added
- **Adoption Block v2 — Single Source of Truth Guidance**: `init-prj` full-tier blocks now include the SSOT rule (ideation artifacts in `docs/brainstorms/` / `docs/ideation/` are disposable inputs to distill into OpenSpec, never parallel specifications), orchestrator blocks carry a one-line distillation directive, and the block header/state version moved to a shared `BLOCK_VERSION` constant (`v=2`). Re-run `ce-ai init-prj <project> --tier <t>` to upgrade adopted projects; `doctor`/`status` report SHA drift for stale v1 blocks until re-adopted (#adoption-block-ssot-v2).

---

## [1.4.1] - 2026-08-22

### Fixed
- **Harness Path Scoping**: Scoped Tier 3 global user harness scanning (`harness-<kind>/skills`) strictly to target harness to prevent cross-harness path bleed (`#135`).
- **Precedence Scope Updates**: Preserved and updated `entry.scope = scope;` during 4-tier precedence overrides (`#135`).
- **UTF-8 Slicing Safety**: Replaced raw byte indexing (`&skill.description[..37]`) with character-aware iterator truncation in `ce-ai skills list` (`#135`).
- **Inline Array Trigger Parsing**: Stripped `[` and `]` brackets when parsing inline YAML array triggers (`triggers: [a, b]`) (`#135`).
- **R3 Security Boundary Hardening**: Narrowed `collect_authorized_roots` to specific harness skill subdirectories and propagated `fs::set_permissions` errors (`#135`).
- **Code Review Refactorings**: Extracted `SkillRegistry::sync_registry` and `SkillRegistry::remove` helpers and introduced `SkillFrontmatter` struct (`#135`).

---

## [1.4.0] - 2026-08-22

### Added
- **Multi-Harness Skill Registry Engine (`ce-ai skills`)**: Harness-neutral JSON index (`~/.ce-ai/skills-registry.json`) indexing and resolving skills across 12 AI coding agent harnesses (Issue #96).
- **Dual-Format Prompt Resolution**: `ce-ai skills resolve --harness <kind> --query <query> [--json]` generating sub-agent prompt blocks (`## Skills to load...`) with resolution-time SHA256 integrity checks and explicit degradation tags (`paths-injected` | `fallback-fuzzy` | `none`).
- **Security Canonicalization Boundary**: Strict path canonicalization rejecting relative path traversals (`../`) or symlinks escaping authorized skill roots (`R3`).
- **YAML Frontmatter Extraction**: Header parser for `SKILL.md` files supporting YAML list bullet syntax (`- trigger`).
- **Lifecycle Integration & `--dry-run` Invariants**: Automatic index building and refresh during `install`, `sync`, `upgrade`, and `init-prj` (gated behind `if !ctx.dry_run`).
- **Sentinel `.gitignore` & Uninstall Parity**: `deinit-prj` and `uninstall` cleanly remove registry files, temporary `.tmp*` artifacts, project stubs, and sentinel-bounded `.gitignore` entries (`# BEGIN CE-AI MANAGED BLOCK` / `# END CE-AI MANAGED BLOCK`).
- **Diagnostic Health Probes**: Wired `skill-registry-integrity` probe into `ce-ai doctor` and created `ce-ai skills doctor` alias.

---

## [1.6.3] - 2026-08-22

### Added
- **Doctor Stale-Block Upgrade Hint**: `ce-ai doctor` now distinguishes stale managed-block versions from content tampering — blocks declaring an older `v=` report a targeted finding (`re-run ce-ai init-prj --tier <tier> to upgrade`) instead of generic SHA drift.
- **Adoption v2 Test Hardening**: new integration coverage for LF-only v1→v2 replacement, malformed-block fail-closed behavior, and header-sha256 ↔ body ↔ state.json triangle consistency.

---

## [1.6.2] - 2026-08-22

### Fixed
- **Installer Download Resilience**: `install.ps1` / `install.sh` retry release downloads (3 attempts, linear backoff) and fall back to the 5 most recent releases when `latest` lacks the platform asset — eliminates HTTP 404 installer/CI failures during concurrent release publication windows. Formula version realigned to match Cargo (was drifting at 1.5.1).

---

## [Unreleased]

### Fixed
- **Models Tab Shows Live Harness Config**: The TUI Models tab now reads assignments from the selected harness's config file (switch scope with ◄/►) instead of stale `state.json` entries — deleting a model from `opencode.json` is immediately reflected (#111).
- **Per-Harness Model Picker**: Pressing `m` discovers models for the harness in scope; harnesses without a catalog CLI fail explicitly instead of showing opencode's list.
- **`models set` Harness Targeting**: New `--harness` flag writes the assignment to the chosen harness's agent-capable config (`ce-ai models set --harness claude <slot> <provider/model>`).
- **Stale State Purge on Sync**: `sync` now removes state assignments whose slot was deleted from the harness config — config wins in both directions.

---

## [1.3.0] - 2026-08-22

### Added
- **`ce-ai` Orchestrator Agent Definition**: `install` seeds the structural agent entry (description, `mode: primary`, permissions) into harness configs that support agent maps — **without** `model` or `variant`; those belong to the user (#111).
- **Harness-Driven Model Discovery**: The TUI model picker lists what the active harness actually offers by querying `opencode models` at runtime; discovery failures surface explicit errors instead of a stale static catalog.
- **Editable TUI Models Tab**: Slot navigation (`n`/`p`) and live model picker (`m`) applying assignments through the same atomic path as `ce-ai models set`; shows only real assignments plus clear "(not assigned)" placeholders.
- **TUI Output Capture**: Dashboard actions now run as captured subprocesses — `println!`-based commands can no longer paint over the alternate screen and break the layout.

### Changed
- `models set` no longer writes `variant` into `opencode.json`; that key is user-owned.

### Fixed
- **Config-Wins Drift Import**: `ce-ai sync` imports effective `opencode.json` assignments into state without pushing stale state back over user-edited config; `doctor` reports `model-assignment-drift` for CE-known slots (#111).

---

## [1.2.2] - 2026-08-22

### Fixed
- **Model Assignment Drift Reconciliation (Issue #111)**: Fixed silent state desynchronization where `agent.<slot>` assignments existed in `opencode.json` but were unrecorded in `state.json`. `ce-ai sync` now bidirectionally reconciles model assignments without data loss.
- **Default Model Assignments**: `ce-ai install` automatically populates documented default model assignments for `ce-ai` (orchestrator slot) and stage slots (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-code-review`, `ce-doc-review`) on fresh installs.
- **`ce-ai doctor` Model Drift Probe**: Extended `ce-ai doctor` with `model-assignment-drift` probe flagging desynchronized slots.
- **Interactive TUI Models Tab**: Models tab in Ratatui dashboard now features interactive slot navigation and cursor selection.

---

## [1.2.1] - 2026-08-22

### Added
- **Worktree Safety Protection**: Added Rule #8 to `AGENTS.md` Hard-Gate Invariant Index prohibiting automated or unconfirmed deletion of sibling worktrees in `<repo>-worktrees/`.
- **Sibling Worktree Doctor Probe**: Extended `ce-ai doctor` with automated discovery (`git worktree list --porcelain`) reporting active sibling worktree paths as advisory `doctor-info:` diagnostic output.
- **OpenSpec Specification**: Documented specification under `openspec/changes/worktree_safety_protection/`.

---

## [1.2.0] - 2026-08-22

### Added
- **Context-Exhaustion Resilience**: Implemented 3-tier defense-in-depth governance pattern (Issue #97) replacing probabilistic prose prompt instructions with fail-closed deterministic platform boundaries.
- **Automated Branch Protection Script**: `scripts/protect-branch.sh` configures GitHub REST API branch protection on `main` (enforcing PRs, 100% green CI matrix status checks, and blocking force pushes).
- **`ce-ai doctor` Health Probes**: Diagnostic suite expanded with `git-hooks` probe (verifying `core.hooksPath` and `.githooks/pre-commit` executable status) and `branch-protection` probe (with `gh auth status` offline fallback).
- **Compact Hard-Gate Invariant Index**: High-density header (~22 lines) top-loaded into `AGENTS.md` for maximum LLM attention weight.
- **Educational Solution Architecture**: Published [`docs/solutions/architecture/context-exhaustion-resilience-and-deterministic-invariants.md`](docs/solutions/architecture/context-exhaustion-resilience-and-deterministic-invariants.md).

---

## [1.1.0] - 2026-08-22

### Added
- **Project Adoption Engine (`ce-ai init-prj` / `ce-ai deinit-prj`)**: Implemented non-destructive, reversible project adoption engine injecting HTML marker-delimited managed blocks (`<!-- ce-ai:block begin ... -->`) into `AGENTS.md` without modifying pre-existing user documentation.
- **Derived Harness Stubs**: Automatically generates derived `CLAUDE.md` reference stubs (`@AGENTS.md`) for sub-harnesses that support file inclusion primitives.
- **Adoption Registry & Backward Compatibility**: Extended `State` in `state.json` with `pub projects: Vec<ProjectAdoptionEntry>` using `#[serde(default, skip_serializing_if = "Vec::is_empty")]` for zero schema breakage.
- **Observability & Diagnostics**: Integrated project adoption health probes and SHA256 block drift detection into `ce-ai status` and `ce-ai doctor`.
- **TUI Shortcut**: Added `[I] Init Prj` shortcut in the interactive TUI dashboard.
- **Documentation**: Published educational [Project Adoption Engine User Guide](docs/user-guide/project-adoption-guide.md) and technical solution doc [`project-adoption-engine-init-and-deinit-prj.md`](docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md).

---

## [1.0.8] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Added `.NET` `HttpWebRequest` stream copy fallback with `AllowAutoRedirect` and replaced non-ASCII multi-byte emojis to prevent encoding syntax corruption in Windows PowerShell 5.1.

---

## [1.0.7] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Added ZIP magic byte header validation (`PK` / `80, 75`) to detect and reject HTML 302 redirect responses before `Expand-Archive`.

---

## [1.0.6] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Switched failure handling to `throw` to enforce instant termination and added diagnostic WebClient exception printing.

---

## [1.0.5] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Fixed `curl.exe` User-Agent flag formatting (`-A "ce-ai-installer/1.0"`) to prevent PowerShell space-splitting argument errors.

---

## [1.0.4] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Added 3-tier download fallback (`curl.exe` -> `Invoke-WebRequest` -> `System.Net.WebClient`) and safe `Test-ValidZipFile` validator function.

---

## [1.0.3] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Streamlined file existence checks (`Test-Path`) without relying on custom boolean variable evaluations.

---

## [1.0.2] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Fixed boolean download validation flag and eliminated `$LASTEXITCODE` null evaluation syntax error in PowerShell.

---

## [1.0.1] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Added GitHub REST API resolution to retrieve direct release asset URLs, `$LASTEXITCODE` validation for native `curl.exe`, and TLS 1.2 fallback download verification.
- **CI Pipeline (`.github/workflows/ci.yml`)**: Added dedicated `windows-installation-gate` job running on `windows-latest` runners.

---

## [1.0.0] - 2026-08-21

### 💎 Production Stable Release
- **CLI Contract & Schema Freeze**: Frozen CLI subcommands (`install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, `doctor`, `workflow`, `tools`, `backups`, `tui`) and state schemas (`state.json`, `.ce-ai.json`, `opencode.json`) with strict backwards-compatibility guarantees.
- **TUI Modal Text Wrapping Fix (Issue #72)**: Ratatui Paragraph line wrapping (`Wrap { trim: false }`) in `MenuTab::Sync` and `MenuTab::Doctor` result modals preventing mid-word text breakage.
- **TUI Direct Stage Invocation (Issue #76)**: Direct stage transition key shortcuts (`[1-7]`) inside `MenuTab::Workflow` panel for rapid Flywheel phase switching.
- **Bug Report Template (Issue #75)**: GitHub Issue Form `.github/ISSUE_TEMPLATE/bug_report.yml`.
- **Homebrew Documentation**: Official installation commands for Homebrew tap and formula in `README.md`.

---

## [0.9.0] - 2026-08-21

### Added
- **ISO 27001 / ISO 27002 Security Audit Test Suite (`tests/security.rs`)**: Path traversal payload rejection, atomic state write tempfile cleanup, and corrupted JSON state recovery.
- **High-Performance Benchmarks (`benches/benchmarks.rs`)**: Verified sub-50ms execution bounds for state resolution, workspace overrides merging, and SHA256 integrity hash calculation.
- **Security Policy Update**: Updated `SECURITY.md` supported versions matrix.

---

## [0.8.0] - 2026-08-21

### Added
- **Multi-Platform Native Release Pipeline (Issue #28)**: GitHub Actions CI matrix cross-compiling release binaries for Linux (`x86_64`, `ARM64`), macOS (`Intel`, `Apple Silicon`), and Windows (`x86_64`, `ARM64`).
- **Universal One-Line Installer Script (Issue #3)**: Cross-platform POSIX (`scripts/install.sh`) and PowerShell (`scripts/install.ps1`) installer scripts for zero-dependency binary downloads.
- **Homebrew Formula Specification (Issue #2)**: Package manager formula template (`Formula/ce-ai.rb`).

---

## [0.7.0] - 2026-08-21

### Added
- **Workspace Configuration Overrides (`.ce-ai.json`)**: Added repository-local `.ce-ai.json` overrides with key-level precedence resolution over global `~/.config/ce-ai/state.json`.
- **Complete Multi-Harness Uninstall Parity (Issue #64)**: Extended `ce-ai uninstall` with `--harness <name|all>`, `--all`, and `--yes` / `-y` flags for complete removal of managed loaders and skills across all installed harnesses cleanly.
- **Teacher-Style Documentation**: Updated masterclass user guide with local cockpit settings vs master flight plan analogies.

---

## [0.6.0] - 2026-08-21

### Added
- **TUI Workflow Dashboard**: Interactive `🎮 Workflow (FSM)` panel in `ce-ai tui` visualizing 7-stage Flywheel status, active subtasks, and progress checkpoints.
- **Extended Companion Health Diagnostics**: Empirical health probes in `ce-ai doctor` for Engram SQLite DB (`~/.engram/engram.db`), CodeGraph index (`.codegraph/`), and RTK binary PATH.
- **Real-Time Sync Watcher**: Added `ce-ai sync --watch` flag for continuous drift monitoring and automatic SHA256 re-syncing.
- **Teacher-Style Documentation**: Updated masterclass user guides explaining cockpit instrument panels and autopilot guardrails.

---

## [0.5.0] - 2026-08-21

### Added
- **Workspace Scope Installation (Issue #7)**: Added `--scope workspace|global` flag in `ce-ai install` allowing repository-scoped skill installations (`./.opencode/`, `./.claude/`, `./.cursorrules`) resolved via `git rev-parse --show-toplevel`.
- **Companion Tools Manager (Issue #9)**: Added `ce-ai tools status` and `ce-ai tools install` for managing developer sidecars and memory servers (`Engram`, `CodeGraph`, `Context7`, `RTK`).
- **Workflow FSM & Recovery Engine (Issue #10)**: Added `ce-ai workflow status`, `ce-ai workflow checkpoint`, and `ce-ai workflow resume` tracking 7-stage development cycle progress and context recovery.
- **Automated GitHub Release Workflow**: Added `.github/workflows/release.yml` for multi-platform cross-compilation on `main` branch pushes.
- **Sync Verification Matrix**: Itemized SHA256 integrity reporting across all active host harnesses in `ce-ai sync`.

---

## [0.4.0] - 2026-08-21

### Added
- Interactive harness selection in TUI dashboard (`< [ target ] >`) supporting navigation across all 12 harness targets (`all`, `opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`).
- Dynamic host harness directory detection supporting `~/.claude`, `~/.pi`, `~/.kimi-code`, `~/.gemini/antigravity-cli`, and `~/.codex`.
- Interactive release version information display in TUI `Upgrade Release` tab.

### Fixed
- Fixed `state.json` recording bug in `src/commands/install.rs` where target harness names were previously hardcoded as `"opencode"`.
- Resolved Dependabot vulnerability alert #1 by upgrading `ratatui` to `0.30` (`lru` upgraded to `v0.18.2`).
- Resolved CodeQL workflow security alerts #1-#3 by adding top-level `permissions` block to `.github/workflows/ci.yml`.

### Governance
- Added mandatory Pull Request workflow directive to `AGENTS.md` prohibiting direct pushes to `main`.

### Added
- Multi-harness support (`HarnessKind` enum and `HarnessAdapter` trait) across 12 AI coding harness targets (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`).
- Native adapters in `src/harness/` for OpenCode, Claude Code, Pi, Cursor, Copilot, Generic JSON, and Custom fallback modes.
- Multi-harness model assignment sync (`ce-ai models set`) and `--all` host harness auto-probing.
- Expanded containerized Docker E2E test gate (`make e2e`).

### Added
- Pre-commit security gate (`.githooks/pre-commit` & `make hooks`) for secret scanning, test suites, and formatting checks.
- Automated PR rejection workflow (`auto-reject-failed-pr`) in GitHub Actions CI.
- Formal OpenSpec 7-stage development cycle and mandatory Stage 2 enforcement in `AGENTS.md`.
- Issue templates for security reports, feature requests, and harness support.
- OpenSpec roadmap items tracking GitHub Issues #1 through #10.

---

## [0.1.0] - 2026-08-20

### Added
- Core `ce-ai` CLI with `install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, and `doctor` subcommands.
- OpenCode harness integration with managed directory isolation and atomic write guarantees (`write_atomic`).
- Model profile snapshotting (`ce-ai models profile save/load`).
- SHA256 file manifest integrity tracking (`manifest.json`).
- Containerized Docker E2E gate suite (`make e2e`).
- Cross-platform CI GitHub Actions matrix (Linux, macOS, Windows).
