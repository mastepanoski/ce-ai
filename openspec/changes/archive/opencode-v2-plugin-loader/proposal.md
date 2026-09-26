# Proposal: Port OpenCode Plugin Loader to the V2 Plugin API

## Problem Statement

OpenCode V2 (verified against installed v2.0.18) rejects the canonical CE plugin loader
(`.opencode/plugins/compound-engineering.js`) at load time with:

> Plugin must export a default definition with an id and an effect or setup function.
> Reference: err_c926271d

The loader still uses the V1 plugin shape (a default-exported async function returning a
hooks object). V1 plugin implementations do not run in V2
(<https://opencode.ai/v2/docs/build/plugins/migrate-v1>):
a V2 plugin must default-export a definition object carrying `id` plus `setup(ctx)` (or
`effect`). The failure was observed on the ce-ai repository's own checkout, and every
installed copy (`~/.config/opencode/compound-engineering/plugins/compound-engineering.js`,
embedded into the binary via `include_str!`) carries the same incompatible shape, so all
OpenCode V2 users of ce-ai lose:

- Turn-0 `RepoState` drift delivery on `session.created` (synthetic context injection).
- Turn-end FSM checkpoint evaluation on `session.idle`.
- Compaction-time and per-request system-context state injection.
- Dynamic skill discovery + slash-command registration from `skills/`.

## Scope Boundaries

### In Scope

1. **Port the canonical loader** (`.opencode/plugins/compound-engineering.js`) to the V2
   API (`export default { id, async setup(ctx) }`) with functional parity:
   - `ctx.skill.transform` registration for every discovered `SKILL.md` (replaces the V1
     `config.skills.paths` push; `user-invocable: false` entries register as skills but
     get no command — V1 parity).
   - `ctx.command.transform` registration of invocable skill commands (replaces the V1
     `config.command` shim; user-configured commands keep precedence).
   - `ctx.event.subscribe()` handling of `session.created` (inject state via
     `ctx.session.synthetic`, the V2 replacement for `noReply` prompts — same idiom as
     OpenCode's built-in Plan-mode reminder) and `session.idle` (checkpoint side effect).
   - `ctx.session.hook("context", ...)` + `ctx.session.hook("compaction", ...)` state
     injection into `event.system` (replaces `experimental.chat.system.transform` and
     `experimental.session.compacting`; the context hook guarantees post-compaction
     delivery on the next model call).
2. **Tighten loader validity detection** (`is_valid_loader_content` in
   `src/opencode/plugins.rs`) so V1-format loaders are treated as outdated and replaced by
   the embedded V2 builtin through `install` / `sync` / `ensure_session_start_plugin`, and
   surfaced by `doctor` — while preserving the #325 loader-safety guarantee for newer
   function-export loaders (legacy `ceLoader*` stubs and the v9 upgrade fixture).
3. **Tests**: unit coverage for V1-rejection/V2-acceptance of loader content.
4. **Docs & versioning**: solution doc + user guides describing the plugin mechanism,
   `CHANGELOG.md`, SemVer patch bump (`1.72.0` → `1.72.1`).

### Out of Scope

- **Renaming the managed `plugin` key to `plugins` in `opencode.json`**: OpenCode v2.0.18
  normalizes legacy config (`plugins: e.plugin`, `skills: [...skills.paths]`,
  `commands: e.command`) at load time, so the ce-ai-managed config remains functional.
  Migration is a separate change.
- **The upstream `compound-engineering@git+...` package entry** some users still have in
  `opencode.json`: owned by EveryInc/compound-engineering-plugin; ce-ai only manages its
  own loader file (removal instructions already exist via `ce-ai uninstall`).
- **Caching `ce-ai workflow resume` output** across hooks: behavioral change beyond the
  port; the V1 cadence (per request/turn) is preserved.
- **Effect-style plugin runtime** (`effect` instead of `setup`): the loader targets the
  Promise API, which plain `.js` plugins can use without importing `@opencode/plugin`.

## Risk Evaluation & Mitigation

- **Registry duplicate risk**: in the managed install layout the same skills directory is
  both scanned from `skills.paths` config and discovered by the plugin transform.
  *Mitigation*: the skill transform uses `editor.get(id)` → `update`/`add`, which is
  idempotent; command registration skips names already present in `ctx.command.list()` so
  user config keeps precedence (mirrors the V1 `!(name in config.command)` guard).
- **Setup-time failure would fail the whole plugin**: any throw inside `setup(ctx)` makes
  OpenCode mark the plugin failed.
  *Mitigation*: all subprocess/filesystem work stays inside `try/catch` (V1 parity); the
  event subscription loop swallows abort errors on unload; command `execute` delegates to
  `ctx.session.prompt` with the same template text shape as V1.
- **Future-loader regression (#325)**: a newer release's loader must never be overwritten
  by an older builtin.
  *Mitigation*: validity keeps accepting function-export `ceLoader*` stubs; covered by the
  existing v9 tarball upgrade test and new unit tests.
- **Event shape drift**: V2 events carry `sessionID` at the top level
  (`{type, sessionID, info}`), not under `properties`.
  *Mitigation*: the handler reads `event.sessionID` with defensive fallbacks.

## Success Criteria

- `node` ESM import of the loader yields `default.id === "compound-engineering"` and a
  function `setup` (OpenCode's exact contract).
- The plugin loads successfully in an isolated OpenCode v2.0.18 instance
  (`opencode plugin list` shows it without the err_c926271d failure), registering skills
  and commands.
- `is_valid_loader_content` rejects the exact V1 file, accepts the V2 file, and still
  accepts `ceLoader*` stubs (existing test suite stays green).
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test`, and `make e2e` all pass.
