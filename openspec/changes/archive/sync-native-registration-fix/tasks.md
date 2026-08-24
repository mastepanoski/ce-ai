# Tasks: `sync-native-registration-fix`

## 1. Sync registration correctness

- [x] **T1.1** Add explicit Kimi / Agy / Fx / Pi arms mirroring install;
      replace the generic fallthrough with `CeError::Runtime`. ✅ CLI tests:
      native configs byte-identical after sync; no OpenCode keys injected.
- [x] **T1.2** Extract `copy_managed_skills` helper; adopt in Claude, Codex,
      Copilot, Grok arms; propagate copy IO errors. ✅ Existing suite green.

## 2. Verification matrix

- [x] **T2.1** Extend hash-check group to Kimi/Agy/Pi/Fx with per-kind
      skills root (Agy `config/skills`). ✅ CLI tests assert `✓ <kind>` and
      drift repair.

## 3. Bookkeeping (docs-only)

- [x] **T3.1** Reality-note annotations on `multi_harness_support/tasks.md`
      Task 2.5 (generic_json history + DeepSeek de-scope) and Task 2.6
      (real verification pointers).

## 4. Release hygiene & gates

- [x] **T4.1** Bump 1.19.1 (Cargo.toml, Formula) + CHANGELOG entry.
- [x] **T4.2** `cargo fmt --check`, clippy `-D warnings`, `cargo test`,
      `make e2e`.
