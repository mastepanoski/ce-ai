# Task Breakdown: Global `--dry-run` Purity & Zero-Mutation Contract

- [x] Add `if !ctx.dry_run` guard in `checkpoint_lines` in `src/commands/workflow.rs`
- [x] Add transient temporary extraction handling for remote sources under `--dry-run` in `src/commands/install.rs` and `src/commands/upgrade.rs`
- [x] Implement `assert_dry_run_zero_mutation` helper in `tests/cli.rs`
- [x] Add snapshot tests verifying `--dry-run workflow checkpoint`, `--dry-run install`, and `--dry-run upgrade` leave zero disk mutations
- [x] Pass `cargo fmt`, `cargo clippy`, and `cargo test`
