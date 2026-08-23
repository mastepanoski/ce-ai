# OpenSpec Tasks: Adoption Block Staleness Alignment

- **Change:** `adoption-block-staleness-alignment`
- **Issue:** #149

---

## 📋 Task Checklist

- [ ] **Task 1**: Implement `check_adoption_block_status` helper and `AdoptionBlockStatus` enum in `src/commands/init_prj.rs`.
- [ ] **Task 2**: Refactor `src/commands/doctor.rs` to use `check_adoption_block_status`.
- [ ] **Task 3**: Refactor `src/commands/status.rs` to use `check_adoption_block_status` and render actionable upgrade hints.
- [ ] **Task 4**: Add CLI integration tests in `tests/cli.rs` asserting `ce-ai status` output for stale blocks.
- [ ] **Task 5**: Verify zero Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`), strict formatting (`cargo fmt --check`), and test suite (`cargo test`).
