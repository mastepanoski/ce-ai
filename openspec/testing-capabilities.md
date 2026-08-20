## Testing Capabilities

**Strict TDD Mode**: enabled
**Detected**: 2026-08-19

### Test Runner

- Command: `cargo test`
- Framework: Rust built-in test harness

### Test Layers

| Layer       | Available | Tool        |
| ----------- | --------- | ----------- |
| Unit        | ✅         | `cargo test` |
| Integration | ✅         | `cargo test --test <name>` |
| E2E         | 🔜 REQUIRED | Docker container with opencode installed, isolated HOME; verify install CE -> sync/upgrade -> model config against real harness files |

### E2E Strategy (user requirement)

- **Environment**: Docker container with `opencode` CLI installed (example: `ghcr.io/opencode-ai/opencode` or built image), fresh `$HOME`, no host state.
- **Scope**: run the built `ce-ai` binary end-to-end: `install` CE into the harness, `sync`/`upgrade` tools, `models` config for subagents; assert resulting harness files (opencode.json, agent blocks, profile files).
- **Isolation**: never touch the host home dirs; mount nothing from the real environment.
- **Gate**: E2E must pass before archive/PR of the feature work.

### Coverage

- Available: ❌
- Command: — (no coverage tool configured; consider `cargo tarpaulin` or `cargo llvm-cov`)

### Quality Tools

| Tool         | Available | Command        |
| ------------ | --------- | -------------- |
| Linter       | ✅         | `cargo clippy --all-targets --all-features -- -D warnings` |
| Type checker | ✅         | `cargo check`  |
| Formatter    | ✅         | `cargo fmt`    |
