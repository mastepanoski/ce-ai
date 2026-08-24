# Implementation Plan: Antigravity (AGY) Adapter Audit Refinements

Document and clarify design details for Google Antigravity (`agy`) Native Harness Adapter:
1. **Environment Variables Documentation**: Document `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` as `ce-ai` extension environment variables in `openspec/changes/agy-native-harness-adapter/design.md`.
2. **Project Rules Documentation**: Document `.agents/rules/compound-engineering.md` and `GEMINI.md` project rule adoption behavior in `openspec/changes/agy-native-harness-adapter/design.md`.
3. **Name Collision Policy Documentation**: Document the name collision behavior in `register_agy_mcp_server` where registering a stdio command server overrides pre-existing remote `serverUrl` entries sharing the same server name.

## User Review Required
> [!NOTE]
> All documentation refinements reflect current codebase behavior and preserve 100% test passing status.

## Proposed Changes

### 1. OpenSpec Contract (`openspec/changes/agy-adapter-audit-refinements/`)
- Create `proposal.md`, `exploration.md`, `design.md`, `spec.md`, and `tasks.md`.
- Update `openspec/changes/agy-native-harness-adapter/design.md` with explicit documentation for `$ANTIGRAVITY_CONFIG_DIR`, `$GEMINI_HOME`, rule adoption, and `serverUrl` collision policy.

### 2. Verification Plan
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
