# Exploration: Kimi Code Native Plugin Manager Divergence

## Technical Investigation

### Kimi Code CLI Native Plugin Layout (verified on live host)
Kimi Code CLI manages native plugins under its home directory (`~/.kimi-code` or `$KIMI_CODE_HOME`):
1. `plugins/installed.json`: Flat registry of installed plugins (array-shaped, unlike Claude's map-shaped registry):
   ```json
   {
     "version": 1,
     "plugins": [
       {
         "id": "compound-engineering",
         "root": "/home/user/.kimi-code/plugins/managed/compound-engineering",
         "source": "github",
         "enabled": true,
         "installedAt": "2026-06-25T00:20:37.761Z",
         "updatedAt": "2026-06-25T00:20:37.761Z",
         "originalSource": "https://github.com/EveryInc/compound-engineering-plugin/tree/main",
         "github": { "owner": "EveryInc", "repo": "compound-engineering-plugin",
                     "ref": { "kind": "branch", "value": "main" } }
       }
     ]
   }
   ```
2. `plugins/managed/<id>/`: The native plugin root. Its version is **not** recorded in `installed.json` — it must be read from `<root>/package.json` (fallback `<root>/plugin.json`) `version` field (e.g. `3.14.3`).
3. `config.toml`: Native user configuration. `extra_skill_dirs = []` lists additional skill trees Kimi loads. A live audit showed `extra_skill_dirs = []` while a ce-ai managed tree existed at `~/.kimi-code/compound-engineering/` — an orphan tree.

### ce-ai State & Content Management
- `state.installed_harnesses` kimi entry: `{ "name": "kimi", "scope": "global", "version": "compound-engineering-v3.24.0", ... }`.
- `state.release_provenance.tag` fallback: `"compound-engineering-v3.24.0"`.
- Managed tree: `<kimi_dir>/compound-engineering/` with `install-manifest.json`, `plugins/`, `skills/`.

### Divergence Mechanism
The two storage trees do not intersect:
- ce-ai writes/updates `~/.kimi-code/compound-engineering/` (tracked by state.json + install-manifest.json).
- Kimi's native manager executes from `~/.kimi-code/plugins/managed/compound-engineering/` at its own pinned version/branch.

Updating ce-ai therefore never updates the native plugin, and the divergence is completely silent — there was no probe at all in `src/harness/kimi.rs` (it only handled `mcp.json`).

### Existing Precedent: Claude (#327)
`check_claude_marketplace_divergence(state, cwd, claude_dir)` in `src/harness/claude.rs` establishes the pattern:
1. Harness guard via `state.installed_harnesses` (workspace scope applicability against `cwd`).
2. `ce_version` from harness entry, fallback `state.release_provenance.tag`.
3. Tolerant parse of the native registry; strict plugin-id filter.
4. Normalize + compare; return typed `Vec<Divergence>`; doctor prints advisory `doctor-info:`.

The Kimi probe mirrors this shape, adapted to Kimi's array registry, `enabled` flag, and version resolution via plugin metadata files.

## Evaluated Options & Architectural Tradeoffs

| Option | Description | Pros | Cons | Verdict |
|---|---|---|---|---|
| **A: Reuse Claude's map-shaped parser** | Parse Kimi's `installed.json` with `NativeInstalledPlugins` (BTreeMap) | Zero new code | Wrong schema: Kimi uses an array; every parse would fail or silently return empty | Rejected |
| **B: Auto-update native plugin** | Invoke Kimi's plugin update during `ce-ai upgrade`/`doctor --fix` | Automatic resolution | Violates read-only invariant; native manager owns its state; process/permission risk | Rejected |
| **C: Read-only typed probe + advisory doctor output** | Array-shaped tolerant parse, `enabled` filter, version from `package.json`/`plugin.json`, typed structs, `doctor-info:`/`doctor-warn:` | Strict read-only; mirrors proven #327 pattern; graceful degradation | User must remediate manually | **Selected** |
| **D: Treat disabled entries as divergent** | Report even `enabled: false` native entries | More information | Noise: a disabled plugin cannot shadow the managed tree; false-positive risk | Rejected |

## Orphan Managed Tree Detection
A ce-ai managed tree for kimi is only consumable by Kimi when `config.toml`'s `extra_skill_dirs` references it. Detection rule:
- Managed tree marker: `<kimi_dir>/compound-engineering/install-manifest.json` exists (distinguishes a ce-ai tree from arbitrary user directories).
- Reference check: parse `config.toml` (TOML crate already a dependency, used by codex/grok harnesses); collect `extra_skill_dirs` string array; the tree is referenced when any entry equals or contains the managed dir path (canonicalized best-effort).
- Missing/unparseable `config.toml` ⇒ not referenced ⇒ warn.

## Version Normalization
Reuse `normalize_plugin_version` from `src/harness/claude.rs` (already `pub`): trim, strip `compound-engineering-` / `compound-engineering@`, strip leading `v`. Compare normalized native (e.g. `3.14.3`) against normalized ce-ai (e.g. `compound-engineering-v3.24.0` → `3.24.0`).
