# Exploration: `custom-harness-r4`

## Investigation (verified on main @ d8f4763)

### Current custom-mode surface

| Site | Behavior today | Verdict |
| --- | --- | --- |
| `src/harness/custom.rs` | `CustomHarnessConfig { plugins_dir, skills_dir, rules_file }` + `CustomAdapter::new` under `#[allow(dead_code)]`; default path `~/.ce-ai/custom_harness.json` | Dead code; never constructed outside its own test |
| `src/harness/mod.rs:273` | `harness_dir(Custom) = ~/.config/custom` | Fictional directory |
| `src/harness/mod.rs:295` | `config_path(Custom) = <base>/custom.json` | Fictional file |
| `src/harness/generic_json.rs` | `GenericJsonAdapter` → `~/.custom/config.json`, entire impl `#[allow(dead_code)]` | Third divergent default; zero callers |
| `src/commands/install.rs` | No `Custom` arm; generic else branch calls `ensure_plugin_and_skills` → writes OpenCode-shaped JSON to `~/.config/custom/custom.json` | Invariant-#5 violation |
| `src/commands/uninstall.rs` | No `Custom` arm; final `else if target_config.is_file()` deletes the fictional JSON | Harmless only because install is fictional |
| `src/commands/sync.rs:309` | Generic else branch re-applies `ensure_plugin_and_skills` for any harness without its own arm | Same violation class for Custom |
| `install.rs` DeepSeek guard | Rejects `--harness deepseek` with `CeError::Usage` | Precedent pattern for honest refusal |
| `agents.rs::ensure_orchestrator_agent` | Returns `Ok(false)` for non-Opencode kinds | Safe for Custom without extra guards |

### Options evaluated

**Option A — Honest descope** (DeepSeek precedent): reject `--harness custom`
with `Usage`, delete the adapter and README mention. Cheapest, but discards a
frozen spec requirement (`multi_harness_support/spec.md` R4) that proposal and
design docs actively advertise (`--plugins-dir/--skills-dir`), and the user
explicitly chose implementation.

**Option B — Implement R4** (chosen): wire configuration resolution + real
asset installation through the existing `CustomHarnessConfig` shape. Cost is
moderate because install/sync already share a managed-tree model
(`MANAGED_PREFIXES` → stripped rel paths → SHA256 manifest) that maps cleanly
onto "copy plugins into dir P, skills into dir S".

**Hybrid considered**: keep flags-only, no config file. Rejected: uninstall and
sync run in separate processes where flags are absent; they need the persisted
resolution. State entry alone would work but `~/.ce-ai/custom_harness.json`
gives users a stable hand-editable contract and matches the adapter's existing
default.

## Key design discoveries

1. **Managed tree mapping is direct.** After `.opencode/` prefix stripping,
   manifest keys are `plugins/**` and `skills/**`. Mapping `plugins/<rest>` →
   `<plugins_dir>/<rest>` and `skills/<rest>` → `<skills_dir>/<rest>` requires
   no new abstraction.
2. **Manifest placement can be self-contained.** `InstallManifest::path_for`
   puts it under `<config_dir>/compound-engineering/`; using `plugins_dir` as
   the config dir gives `<plugins_dir>/compound-engineering/install-manifest.json`.
3. **Rules file has reusable machinery.** `init_prj.rs` exposes managed-block
   markers (`<!-- ce-ai:block begin v=N -->` / end) plus
   `render_block_content(tier)` and SHA checks. The custom rules file reuses
   these so uninstall strips exactly what was injected.
4. **State entry needs the resolved paths.** Uninstall/sync must not depend on
   flags being repeated or on the config file still matching; the state entry
   records the resolved `custom` object at install time and acts as source of
   truth thereafter.
5. **Uninstall semantics differ from native harnesses.** Native skill dirs are
   harness-owned, so `remove_dir_all` is acceptable there. Custom dirs are
   user-owned by definition; removal must be surgical (manifest-recorded files
   only, then prune now-empty CE-owned directories).
6. **Pre-existing sync fallthrough affects native harnesses too.** `sync.rs`
   routes Pi/Kimi/Agy/Fx into the same generic `ensure_plugin_and_skills`
   else-branch. Fixing those four is a separate change with its own per-harness
   registration ports; this change fixes only the Custom arm and records the
   finding here as follow-up debt.
7. **`GenericJsonAdapter` is fully dead.** Only references are its own module
   declaration and internal test. Deleting the file removes the third default.

## Consequences

- Single path contract: `~/.ce-ai/custom_harness.json` (adapter default,
  `mod.rs` Custom arms agree).
- `models set --harness custom` stays rejected via
  `supports_agent_definitions` (no agent-map concept for unknown harnesses).
- `status` renders custom entries unchanged (generic name/version/source
  rendering already tolerates the extra `custom` field).
- README's "plus custom fallback mode" becomes true; harness-matrix doc gains
  a concrete custom row.
