# Task Breakdown: Multi-Harness Reconciliation, DeepSeek De-scope & Release Fallback Hardening

- [ ] Update `README.md` to qualify 10 native AI agent harnesses and document native formats
- [ ] Reconcile `openspec/changes/multi_harness_support/spec.md` with shipped native adapter architecture and remove DeepSeek from JSON harness list
- [ ] Implement `DeepSeek` de-scope handler returning `CeError::Usage` (exit code 2) on CLI invocations and filter out `Deepseek` from harness detection
- [ ] Remove `Deepseek` arm from `src/harness/generic_json.rs`
- [ ] Implement GitHub API 403 / 429 / network error fallback to `main_tarball_url()` in `src/source/release.rs`
- [ ] Update audit score label, `--fail-under` docstring, and threshold error message to `configuration coverage` in `src/commands/audit.rs`
- [ ] Bump version to `1.18.0` in `Cargo.toml` and `Formula/ce-ai.rb`
- [ ] Update `CHANGELOG.md` under `[1.18.0]`
- [ ] Run `ce-doc-review` panel
- [ ] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [ ] Run `ce-code-review` panel
- [ ] Document solution in `docs/solutions/architecture/multi-harness-reconciliation.md`
- [ ] Create branch `feat/multi-harness-reconciliation`, commit, push, PR, merge, release `v1.18.0`
