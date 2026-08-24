> STATUS (v1.20.1): Worktree probes live in src/commands/doctor.rs. Residual open boxes below were not re-audited item-by-item.

# OpenSpec Tasks: Worktree Safety Protection

- [x] Create OpenSpec specification (`proposal.md`, `spec.md`, `tasks.md`)
- [ ] Update `AGENTS.md` Hard-Gate Invariant Index with Rule 8 (Preserve Active Worktrees)
- [ ] Extend `src/commands/doctor.rs` with `git worktree list --porcelain` probe
- [ ] Add CLI integration test in `tests/cli.rs` verifying `doctor` reports sibling worktrees
- [ ] Verify `cargo test` passes 100%
- [ ] Submit PR and achieve 100% green CI matrix
