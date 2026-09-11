# Proposal: Detect Claude Code Native Marketplace Plugin Divergence in Doctor

## Problem Statement
`ce-ai` manages its own isolated content tree for `compound-engineering` across supported harnesses (`~/.claude/compound-engineering/`, `~/.config/opencode/compound-engineering/`, etc.) tracked via `state.json` (`installed_harnesses` and `release_provenance`).

However, Claude Code also maintains a native plugin marketplace under `~/.claude/plugins/` (with index `~/.claude/plugins/installed_plugins.json` and cache `~/.claude/plugins/cache/`). When a user enables `compound-engineering@compound-engineering-plugin` via Claude Code's native plugin manager, Claude Code executes skills and personas directly from its native cache, completely bypassing the tree managed by `ce-ai`.

When a user runs `ce-ai upgrade`, `ce-ai` updates its managed tree to the latest release (e.g. `3.24.0`), but Claude Code's native plugin may remain pinned to an older version (e.g. `3.8.4` or `3.17.1`). This produces silent, baffling version divergence: `ce-ai status` reports 3.24.0, but the active session in Claude Code runs 3.8.4.

Currently, `ce-ai doctor` does not inspect `~/.claude/plugins/installed_plugins.json`. Users have no automated way to discover this divergence without manually inspecting internal JSON manifests.

## In-Scope
1. **Best-Effort Native Marketplace Inspection**: Read `<claude_dir>/plugins/installed_plugins.json` (where `<claude_dir>` respects `CLAUDE_CONFIG_DIR` or defaults to `~/.claude`).
2. **Scope-Aware Applicability Matching**:
   - `user` scope: always applicable across all working directories.
   - `project` and `local` scopes: applicable when the current working directory (`cwd`) or repository root is equal to or a subpath of `projectPath`.
3. **Plugin Identity & Version Normalization**:
   - Match plugins targeting `compound-engineering` (exact or `compound-engineering@<marketplace>`).
   - Normalize versions by stripping prefixes (`compound-engineering-`, `compound-engineering@`, `v`) before comparing.
4. **Structured Divergence Detection**: Provide a typed data model `ClaudeMarketplaceDivergence` in `src/harness/claude.rs` for clear domain modeling.
5. **Non-Blocking Doctor Diagnostic**: In `ce-ai doctor`, report each applicable divergence as an advisory `doctor-info:` notice with the exact remediation command (`claude plugin marketplace update <marketplace> && claude plugin update <plugin>`), without altering `findings` or failing the exit code.
6. **Graceful Degradation**: If `installed_plugins.json` is missing, unreadable, or malformed, or if `claude` is not an installed harness in `state.json`, degrade silently without reporting false positives or failing `doctor`.
7. **Empirical Verification**: Unit test matrix in `src/harness/tests/claude.rs` and hermetic CLI integration test in `tests/cli.rs`.

## Out-of-Scope
1. **Adding `--json` CLI Flag to `doctor`**: The CLI contract for `doctor::Args` supports only `--strict`. Introducing a new `--json` flag to `doctor` would cause scope creep across the entire command. Typed domain structs satisfy programmatic consumers internally.
2. **Mutating Native Marketplace Files**: `ce-ai` is strictly read-only on `<claude_dir>/plugins/`. Under no circumstances will `ce-ai` edit native marketplace manifests or delete native caches.
3. **Executing Native Commands Automatically**: `ce-ai` will not automatically invoke `claude plugin update`, preserving user control and external process boundaries.
4. **Simulating Precedence Resolution**: Do not attempt to guess whether Claude Code prioritizes `local` over `project` over `user`. All applicable divergent scopes are reported transparently.

## Risk Evaluation & Mitigation
- **Risk (False Positives on Unrelated Plugins)**: Other native plugins (e.g. `vercel`, `swift-lsp`) might have conflicting versions.
  - *Mitigation*: Strictly filter to `compound-engineering` / `compound-engineering@*`.
- **Risk (JSON Schema Breaking Changes)**: Anthropic could alter `installed_plugins.json` schema in future releases.
  - *Mitigation*: Use tolerant serde deserialization (`#[serde(default)]`, Option fields) and degrade to empty without crashing or emitting error diagnostics.
- **Risk (CI Blocking)**: Emitting divergence as an error would fail CI runs in environments with disparate plugin caches.
  - *Mitigation*: Emitted strictly as `doctor-info:`, keeping exit code 0.
