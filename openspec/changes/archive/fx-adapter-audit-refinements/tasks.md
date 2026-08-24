# Task Breakdown: Vercel Labs fx Adapter Audit Refinements

- [x] Remove `.exists()` filesystem check from `FxAdapter::default_config_path` in `src/harness/fx.rs`
- [x] Propagate IO errors on `remove_file` in `unregister_fx_mcp_server` (ignoring `NotFound`) in `src/harness/fx.rs`
- [x] Add unit tests in `src/harness/fx.rs` verifying deterministic path resolution when `$HOME/mcp.json` pre-exists
- [x] Add unit tests in `src/harness/fx.rs` verifying `unregister_fx_mcp_server` IO error propagation and file removal
- [x] Document `FX_HOME` extension convention in `openspec/changes/fx-adapter-audit-refinements/design.md` and `spec.md`
- [x] Document extra map `type` collision cleanup in `openspec/changes/fx-adapter-audit-refinements/design.md` and `spec.md`
- [x] Run `ce-doc-review` panel
- [x] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution in `docs/solutions/architecture/fx-adapter-audit-refinements.md`
- [x] Create branch `feat/fx-adapter-audit-refinements`, commit, push, PR, merge, release patch `v1.17.2`
