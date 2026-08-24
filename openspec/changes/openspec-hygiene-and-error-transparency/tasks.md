# Tasks: `openspec-hygiene-and-error-transparency`

- [ ] **T1.1** Tick shipped tasks in custom-harness-r4 (12),
      sync-native-registration-fix (6), sync-registration-strategy (4).
- [ ] **T1.2** Create `archive/README.md` (convention + triage table);
      `git mv` the 32 zero-open folders + upgrade-provenance-sync-verification
      with STATUS header (v1.18.1 evidence).
- [ ] **T2.1** Add `report_best_effort_remove/_write` to `src/state/mod.rs`
      with NotFound-silence unit tests.
- [ ] **T2.2** Convert deinit_prj (18), init_prj (3), tmp cleanups (3),
      registry syncs (2), ctrl-c handler (1); annotate uninstall pruning;
      tidy the parse discard.
- [ ] **T3.1** Bump 1.20.1 + CHANGELOG.
- [ ] **T3.2** Gates: fmt / clippy `-D warnings` / cargo test / make e2e.
