# Design: `custom-harness-r4`

## Architecture

```
CLI flags (--plugins-dir/--skills-dir/--rules-file)
        │  highest precedence
        ▼
resolve_custom_config()  ──►  CustomHarnessConfig { plugins_dir, skills_dir, rules_file? }
        ▲                     (src/harness/custom.rs)
        │  fallback
~/.ce-ai/custom_harness.json  { "plugins_dir": "...", "skills_dir": "...", "rules_file": "..." }
        │
        ▼
install.rs ─┬─ Custom branch: copy assets + manifest + state entry
uninstall.rs ─┬─ Custom branch: resolve from state entry ▸ surgical removal
sync.rs ────┴─ Custom branch: re-copy + SHA256 verification surface
```

Single path contract: **`~/.ce-ai/custom_harness.json`**. All other defaults
(`~/.config/custom`, `~/.custom/config.json`) are removed.

## Data Schemas

### Config file `~/.ce-ai/custom_harness.json`

```json
{
  "plugins_dir": "/abs/or/~-relative/path",
  "skills_dir": "/abs/or/~-relative/path",
  "rules_file": "/optional/path/rules.md"
}
```

Parsed into the existing `CustomHarnessConfig` (serde). Unknown keys are
ignored on read and preserved implicitly by read-modify flows; missing
`plugins_dir` or `skills_dir` makes resolution fail.

### State entry extension (`state.installed_harnesses[]`)

```json
{
  "name": "custom",
  "version": "...",
  "source": { ... },
  "installed_at": "...",
  "last_synced_at": "...",
  "custom": {
    "plugins_dir": "...", "skills_dir": "...", "rules_file": "..."
  }
}
```

The resolved configuration is snapshotted at install time; uninstall/sync
prefer it over re-resolution.

## CLI Contracts

`install` gains:

| Flag | Type | Meaning |
| --- | --- | --- |
| `--plugins-dir <PATH>` | `Option<PathBuf>` | Target directory for CE plugin assets |
| `--skills-dir <PATH>` | `Option<PathBuf>` | Target directory for CE skill folders |
| `--rules-file <PATH>` | `Option<PathBuf>` | Markdown rules file receiving the managed CE block |

`uninstall` gains the same three flags (used only when the state entry lacks
the `custom` snapshot).

Resolution order (both commands): flags ▸ state-entry snapshot (uninstall/sync)
▸ `~/.ce-ai/custom_harness.json` ▸ `CeError::Usage` with guidance text.
Relative paths and `~/` prefixes are expanded against `$HOME`.

## Component Changes

### `src/harness/custom.rs`

- Remove `#[allow(dead_code)]` from `CustomAdapter::new`.
- Add `pub fn resolve(home: &Path, flags: CustomConfigFlags) -> Result<CustomHarnessConfig, CeError>`
  implementing the precedence chain and `~` expansion.
- Add asset-layout helpers:
  - `plugin_rel(managed_rel) -> Option<&str>` (`plugins/x` → `Some("x")`)
  - `skill_rel(managed_rel) -> Option<&str>` (`skills/x` → `Some("x")`)
- Keep `default_config_path(home) = home/.ce-ai/custom_harness.json` as the
  one true default; drop the rules/plugins-dir fallback inside it (a configured
  adapter resolves paths through `resolve`, not through path synthesis).

### `src/harness/mod.rs`

- `harness_dir(Custom)` → `home_dir.join(".ce-ai")` (the directory containing
  the single-contract config file).
- `config_path(Custom)` → `base_dir.join("custom_harness.json")` so the
  chained `config_path(harness_dir(home))` used across commands yields
  `~/.ce-ai/custom_harness.json`.

### `src/harness/generic_json.rs` — deleted

Zero production callers; removes the third fabricated default.

### `src/commands/install.rs`

New branch `else if *harness_kind == HarnessKind::Custom` (before the final
else):

1. `let cfg = resolve(...)?` (Usage error when unresolvable; dry-run prints
   the plan against resolved paths first).
2. Create `plugins_dir`, `skills_dir` (and `rules_file` parent) as needed.
3. Copy managed files:
   - `plugins/<rest>` → `plugins_dir/<rest>` (includes loader
     `compound-engineering.js`)
   - `skills/<rest>` → `skills_dir/<rest>`
   All writes via `write_atomic`; hashes recorded as `ManifestFile`s.
4. Rules file (when configured): idempotently replace-or-append the managed
   block `<!-- ce-ai:block begin v=2 … end -->` using
   `render_block_content(AdoptionTier::Full)`; record a `ConfigMutation`
   (backup captured beforehand when the file exists).
5. Manifest written to `plugins_dir/compound-engineering/install-manifest.json`.
6. State entry pushed with the `custom` snapshot.

No MCP registration; no `ensure_plugin_and_skills`; no agent-map write
(`ensure_orchestrator_agent` already no-ops for Custom).

### `src/commands/uninstall.rs`

New Custom arm:

1. Resolve config preferring the state entry snapshot, then flags, then the
   config file; Usage error if nothing resolves.
2. Load the manifest from `plugins_dir/compound-engineering/`; delete each
   recorded file; prune directories that ce-ai emptied (never `remove_dir_all`
   on user-owned roots); remove the manifest itself.
3. Strip the managed block from `rules_file` when present; leave all other
   content untouched.
4. Drop the state entry.

When no manifest exists but a state entry does, fall back to removing the
recorded layout paths (`plugins_dir/plugins/**`, `skills_dir/**` keys known
from the current source tree are NOT guessed — absence of manifest yields a
warning and best-effort removal of the two recorded roots' CE subpaths).

### `src/commands/sync.rs`

- Before the generic else, add the Custom arm: re-copy managed trees into the
  resolved dirs (state-entry preferred) exactly like install step 3–4.
- Preserve the `custom` field when rebuilding `state.installed_harnesses`
  (currently cleared and re-pushed).
- Verification matrix: hash-check `plugins_dir` and `skills_dir` surfaces via
  `verify_tree_against`; report `Verified`/`Failed` like native skills
  surfaces; drift fails with exit code 6.

## Error Mapping

| Condition | Error | Exit |
| --- | --- | --- |
| `--harness custom` with no resolvable configuration | `CeError::Usage` | 2 |
| Rules file unreadable / block malformed beyond repair | `CeError::Runtime` | 1 |
| Copy/IO failure mid-install | `CeError::Io` | 4 |
| Post-sync hash drift on custom surfaces | `CeError::Verification` | 6 |

## Testing Strategy

- Unit (`src/harness/custom.rs`): resolution precedence, `~` expansion, missing
  config error, rel-path mappers, block strip/replace helpers.
- CLI (`tests/cli.rs`, hermetic HOME/config-dir fixtures):
  - install without config → exit 2, zero filesystem writes;
  - install with flags → exact tree layout + manifest + state snapshot;
  - install with config file only; flags override file values;
  - idempotent reinstall replaces block, keeps user lines around it;
  - uninstall removes recorded files, preserves foreign files, strips block;
  - sync re-copies drifted assets and detects tampering (exit 6).
