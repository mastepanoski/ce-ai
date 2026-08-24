# Tasks: `custom-harness-r4`

Checklist generated from design.md. TDD: write the failing test first for
every ✅-gated item.

## 1. Contract foundation

- [ ] **T1.1** Delete `src/harness/generic_json.rs` + its `mod` line; update
      `mod.rs` Custom arms (`harness_dir` → `~/.ce-ai`, `config_path` →
      `custom_harness.json`); drop `#[allow(dead_code)]` from
      `CustomAdapter::new`. Verify: `cargo clippy --all-targets
      --all-features -- -D warnings` green.
- [ ] **T1.2** Implement `CustomHarnessConfig::resolve(home, flags)` with
      flag ▸ config-file precedence, `~` expansion, and `CeError::Usage` on
      unresolvable input. ✅ Unit tests: precedence, expansion, error text.

## 2. Install path

- [ ] **T2.1** Add `--plugins-dir/--skills-dir/--rules-file` to
      `install::Args`. Verify: `cargo run -- install --help` shows them.
- [ ] **T2.2** Implement the `HarnessKind::Custom` install branch (layout
      copy, manifest at `P/compound-engineering/`, state `custom` snapshot,
      rules-file block injection, dry-run plan). ✅ CLI tests:
      flags-install layout+manifest+state; usage-error fast-fail writes
      nothing; config-file install; flags override file; idempotent
      reinstall preserves surrounding user lines.

## 3. Uninstall path

- [ ] **T3.1** Add the three flags to `uninstall::Args`; implement the
      surgical Custom arm (manifest-driven removal, empty-dir pruning, block
      strip, state cleanup, Usage fast-fail). ✅ CLI tests: recorded files
      gone, foreign files kept, block stripped, rest of file intact.

## 4. Sync path

- [ ] **T4.1** Add the Custom arm to `sync_with` (re-copy + manifest
      refresh), preserve `custom` snapshots across the state rebuild, and
      register custom surfaces in the verification matrix. ✅ CLI tests:
      drift repaired by sync; tampered file → exit 6 with FAILED surface.

## 5. Docs & release hygiene

- [ ] **T5.1** Update `docs/user-guide/harness-matrix.md` custom row and
      confirm README claim accuracy (≤ 100 lines, style guide compliant).
- [ ] **T5.2** Bump `Cargo.toml` + `Formula/ce-ai.rb` to 1.19.0; add
      CHANGELOG.md entry under Added/Changed/Fixed.

## 6. Gates (Definition of Done)

- [ ] **T6.1** `cargo fmt --check`
- [ ] **T6.2** `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] **T6.3** `cargo test`
- [ ] **T6.4** `make e2e`
