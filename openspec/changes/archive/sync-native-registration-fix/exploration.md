# Exploration: `sync-native-registration-fix`

## Investigation

### Fallthrough blast radius (verified against current main)

`sync_with()` rebuilds every active harness after reconciling the OpenCode
managed tree. Arms existed for Cursor, Claude, Codex, Copilot, Grok, and —
since v1.19.0 — Custom. Everything else fell into:

```rust
} else {
    let _ = crate::opencode::config::ensure_plugin_and_skills(
        &target_config,
        &plugin_entry(&config_dir)...,
        &skills_path(&config_dir)...,
    );
}
```

Two defects in one line pair:

1. Wrong-format writes for any harness whose `target_config` is readable
   JSON (Kimi/Agy/Fx native MCP files): OpenCode-specific keys injected.
2. `let _ =` swallows every failure class — including Pi, whose
   `default_config_path` intentionally resolves to the skills *directory*
   (Pi has no config file concept; No-MCP philosophy from objective 8),
   making `read_config` fail every time.

### Why install was already correct

Install gained dedicated arms per harness during the audit series (#199-#203):
MCP registration helpers exist per vendor (`register_{kimi,agy,fx}_mcp_server`)
and skills trees are copied to per-vendor roots. Sync simply never received
the same treatment — the series' "consistencia spec↔impl" pattern.

### Options considered

1. **Per-kind arms mirroring install** (chosen): explicit, mirrors install
   1:1, keeps sync self-contained.
2. Extract a per-harness registration strategy trait shared by install/sync:
   architecturally nicer but a cross-cutting refactor of ~10 call sites;
   rejected for this fix's scope (noted as future consolidation).
3. Skip non-armed harnesses in sync (no-op): would leave skill trees stale —
   worse than status quo for correctness.

### Verification matrix gap

Directory-copying harnesses were split: Claude/Codex/Copilot/Grok hash-checked;
Kimi/Agy/Pi/Fx labelled "config registration only". With real skill copies in
their new arms, all eight belong in the checked group. Agy's root is
`<home>/.gemini/config/skills` (note the extra `config` segment), so the loop
needs a per-kind root resolver instead of a bare `.join("skills")`.

### Bookkeeping findings

- `multi_harness_support/tasks.md` Task 2.5 claims `generic_json.rs`
  implemented Codex/Grok/Kimi/AGY/DeepSeek — historically false (it only ever
  implemented Custom), and the file itself was removed in v1.19.0 after the
  custom-r4 contract work. Task 2.6 cites
  `harness::custom::tests::custom_flags_registration`, a test that never
  existed. Both get reality-note annotations; requirements/spec text stays
  frozen.

### fx clause disposition

Objective 9's pending ("cláusula exists() latente, fx.rs:26") refers to the
filesystem-dependent `exists()` check removed by `fx-adapter-audit-
refinements`. Current code performs deterministic basename checks with unit
coverage for all three arms; production callers always hit the third arm.
Verified closed — no code change required here.
