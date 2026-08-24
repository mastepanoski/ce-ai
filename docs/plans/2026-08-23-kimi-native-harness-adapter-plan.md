# Implementation Plan: Kimi Code CLI Native Harness Adapter (Issue #178)

- **Goal**: Build native harness adapter for Kimi Code CLI (`kimi`) targeting `~/.kimi-code/mcp.json`, `$KIMI_CODE_HOME/skills/`, and `AGENTS.md` project rules adoption.

## Proposed Work Steps

### Step 1: OpenSpec Contract & Doc Review
- Author OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).
- Run `ce-doc-review` panel and address findings.

### Step 2: Implementation
- `src/harness/kimi.rs`: Implement `KimiAdapter`, `KimiMcpConfig`, `KimiMcpServer`, `register_kimi_mcp_server`, `unregister_kimi_mcp_server`.
- `src/harness/mod.rs`: Wire `HarnessKind::Kimi` native path resolution (`$KIMI_CODE_HOME`, `~/.kimi-code`, `mcp.json`).
- `src/commands/install.rs`, `tools.rs`, `init_prj.rs`, `deinit_prj.rs`, `sync.rs`, `uninstall.rs`, `state/backups.rs`: Wire Kimi native handling.

### Step 3: Verification & Quality Gates
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

### Step 4: Code Review & Shipping
- Run `ce-code-review` panel.
- Document solution in `docs/solutions/architecture/kimi-native-harness-adapter.md`.
- Save Engram observation.
- Bump SemVer to `v1.14.0` in `Cargo.toml` and `Formula/ce-ai.rb`, update `CHANGELOG.md`.
- Commit, push, open PR, wait for green CI matrix, merge, tag `v1.14.0`, release, close Issue #178.
