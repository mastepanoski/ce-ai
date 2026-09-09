# Proposal: Detect Kimi Code Native Plugin Manager Divergence in Doctor

## Problem Statement
`ce-ai` manages its own isolated content tree for `compound-engineering` per harness. For Kimi Code CLI that tree lives at `~/.kimi-code/compound-engineering/` and is tracked via `state.json` (`installed_harnesses` entry for `kimi`, e.g. version `compound-engineering-v3.24.0`).

However, Kimi Code CLI also maintains a **native plugin manager** under `~/.kimi-code/plugins/` with its own index `~/.kimi-code/plugins/installed.json` and managed roots `~/.kimi-code/plugins/managed/<id>/`. When a user installs/enables `compound-engineering` via Kimi's native plugin flow (e.g. pinned to branch `main`), Kimi executes skills directly from its native managed root — completely bypassing the tree managed by `ce-ai`.

This produces silent version divergence: `ce-ai status`/`doctor` reports the managed version (e.g. `3.24.0`), while the active Kimi session runs the native plugin version (e.g. `3.14.3` from the native root's `package.json`). An audit of a live host confirmed exactly this state, and `src/harness/kimi.rs` had no divergence probe — the Claude Code equivalent (`check_claude_marketplace_divergence`, #327) existed, but Kimi had none.

A second, related gap: the ce-ai-managed tree at `~/.kimi-code/compound-engineering/` is only useful to Kimi when referenced from `~/.kimi-code/config.toml` (`extra_skill_dirs`). When that reference is absent, the managed tree is an orphan on disk — consuming space but never loaded — and users have no signal.

## In-Scope
1. **Best-Effort Native Plugin Inspection**: Read `<kimi_dir>/plugins/installed.json` (where `<kimi_dir>` respects `KIMI_CODE_HOME` or defaults to `~/.kimi-code`).
2. **Enabled-Plugin Filtering**: Only *enabled* native `compound-engineering` entries are evaluated for divergence (a disabled entry cannot shadow the managed tree).
3. **Native Version Resolution**: Resolve the native plugin's version from `<root>/package.json` or `<root>/plugin.json` (`version` field), tolerating missing/unreadable files.
4. **Harness & Scope Guard**: Probe applies only when `kimi` is registered in `state.installed_harnesses` with a scope applicable to `cwd` (workspace scope must match/target `cwd`).
5. **Version Normalization Reuse**: Reuse `crate::harness::claude::normalize_plugin_version` (strip `compound-engineering-`, `compound-engineering@`, `v` prefixes).
6. **Orphan Managed Tree Detection**: When `<kimi_dir>/compound-engineering/install-manifest.json` exists (a genuine ce-ai managed tree) but `~/.kimi-code/config.toml` `extra_skill_dirs` does not reference that tree, report a non-blocking warning.
7. **Structured Domain Model**: Typed `KimiMarketplaceDivergence` and `KimiOrphanManagedTree` structs in `src/harness/kimi.rs`.
8. **Non-Blocking Doctor Diagnostics**: Advisory `doctor-info:` (divergence) / `doctor-warn:` (orphan tree) output, never added to fatal `findings`, exit code unchanged.
9. **Graceful Degradation**: Missing, unreadable, or malformed `installed.json`, `config.toml`, or plugin metadata degrade silently to "no finding".
10. **Empirical Verification**: Unit test matrix with tempdir fixtures in `src/harness/tests/kimi.rs` plus a non-blocking doctor integration test in `src/commands/tests/doctor.rs`.

## Out-of-Scope
1. **Mutating Native Plugin Files**: `ce-ai` remains strictly read-only on `~/.kimi-code/plugins/`.
2. **Auto-Updating the Native Plugin**: No automatic invocation of Kimi's plugin update flow; remediation is advisory only.
3. **Editing `config.toml`**: The orphan-tree probe never writes `~/.kimi-code/config.toml`; pointing `extra_skill_dirs` at the managed tree is a user decision (ce-ai's skill delivery for kimi remains MCP + adoption based).
4. **Other Harnesses**: Cursor/Codex/Copilot/Grok/Agy/Fx native plugin equivalents are separate audits; this change is Kimi-only to mirror the Claude (#327) precedent.

## Risk Evaluation & Mitigation
- **Risk (False Positives on Unrelated Plugins)**: Other native plugins (`kimi-datasource`, `superpowers`) share `installed.json`.
  - *Mitigation*: Strict filter to `compound-engineering` / `compound-engineering@*` ids, and only `enabled: true` entries.
- **Risk (Native Schema Drift)**: Kimi may change `installed.json` or plugin metadata layout.
  - *Mitigation*: Tolerant serde (`#[serde(default)]`, `Option` fields); any parse failure degrades to empty results, never a crash or false finding.
- **Risk (CI Blocking)**: Divergence/orphan findings must not fail `ce-ai doctor` in CI.
  - *Mitigation*: Emitted strictly as `doctor-info:` / `doctor-warn:`, never pushed into `findings`; exit code unchanged.
