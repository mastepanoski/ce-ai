> STATUS (v1.20.1): ce-ai tools command live in src/commands/tools.rs. Residual open boxes below were not re-audited item-by-item.

# OpenSpec Tasks: Tools Install Real Sidecar Registration

- **Change:** `tools-install-real-sidecar-registration`
- **Issue:** #158 (P0)

---

## 📋 Task Checklist

- [ ] **Task 1**: Refactor `install_tool` in `src/commands/tools.rs` to parse `opencode.json` and merge MCP server definition.
- [ ] **Task 2**: Save modified `opencode.json` atomically using `crate::state::write_atomic`.
- [ ] **Task 3**: Add post-install health probe check returning `CeError::Verification` on failure.
- [ ] **Task 4**: Add unit tests in `src/commands/tools.rs` and CLI integration tests in `tests/cli.rs`.
- [ ] **Task 5**: Verify formatting (`cargo fmt --check`), clippy (`cargo clippy --all-targets --all-features -- -D warnings`), and test suite (`cargo test`).
