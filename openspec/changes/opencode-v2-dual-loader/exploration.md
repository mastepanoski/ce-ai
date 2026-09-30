# Technical Exploration: OpenCode V1/V2 Dual Plugin Loader & Command Architecture

## 1. Problem Investigation & Empirical Diagnostics

### 1.1 Config Key Renaming in OpenCode V2
In OpenCode V1, plugins were configured in `opencode.json` under the singular array key:
```json
{
  "plugin": [
    "/path/to/compound-engineering.js"
  ]
}
```
OpenCode V2 modernized the configuration schema to:
```json
{
  "plugins": [
    "@package/name",
    "/path/to/plugin/dir"
  ]
}
```
In `src/opencode/plugins.rs`, `has_session_start_plugin()` inspected solely `val.get("plugin").and_then(|p| p.as_array())`. When a user updated to OpenCode V2 or migrated their config to `"plugins"`, `has_session_start_plugin()` returned `false`, and `ce-ai doctor` triggered a permanent false-negative finding.

Furthermore, `merge_plugin` in `src/opencode/config.rs` always appended to `"plugin"`. On V2 installations, this generated a redundant, orphaned `"plugin"` key alongside the active `"plugins"` key.

### 1.2 Plugin Loader API Divergence
- **V1 Contract**: OpenCode V1 imported the file and called either the default function export or `export const CompoundEngineeringPlugin = async ({ client, ... }) => ({ config, event, ... })`.
- **V2 Contract**: OpenCode V2 enforces that every plugin export a default definition containing `id: string` and either `setup(ctx)` or `effect` function:
  ```javascript
  export default {
    id: "compound-engineering",
    setup(ctx) { ... }
  };
  ```
  If a V1 function export is provided, OpenCode V2 throws:
  `PluginModule.LoadError: Plugin must export a default definition with an id and an effect or setup function. (cause: SchemaError(Missing key at ["default"]))`.

### 1.3 Command Discovery in OpenCode V2
In OpenCode V1, slash commands were registered dynamically via the `config` hook:
```javascript
config.command[name] = { template, description };
```
In OpenCode V2, the plugin system replaced the monolithic `config` hook with domain-specific transforms (`ctx.command.transform`, `ctx.skill.transform`). However, OpenCode V2 also introduced native filesystem command discovery:
The background daemon scans `{command,commands}/**/*.md` inside configuration roots (`~/.config/opencode/` and `.opencode/`), parsing frontmatter description and treating the body as an executable template substituting `$ARGUMENTS` and `$1..$N`.

Additionally, OpenCode V2's `ConfigPluginSource.scan` explicitly rejects absolute file targets in `plugins: [...]`, logging:
`configured plugin path must be a directory`
and only auto-discovers file-based plugins when placed directly inside `~/.config/opencode/plugins/*.{js,ts}` or `.opencode/plugins/*.{js,ts}`.

---

## 2. Evaluated Architectural Options

### 2.1 Dual Export vs. Resolving Variant by Version
We evaluated two strategies for supporting both OpenCode V1 and V2:
- **Alternative 1: Separate Loaders resolved at Install/Sync time based on detected version**
  - *Mechanism*: Detect OpenCode version (`opencode --version`), install `compound-engineering-v1.js` if V1 or `compound-engineering-v2.js` if V2.
  - *Discarded because*:
    1. Brittle across multi-environment or portable configs (e.g. dotfiles synced between a machine with OpenCode V1 and another with OpenCode V2).
    2. Version detection can fail if `opencode` is not directly on PATH during `ce-ai install` (e.g. running inside containers, GUI wrappers, or custom toolchains).
    3. Doubles the asset surface and test matrix.
- **Alternative 2 (Selected): Single Dual-Compatible Export**
  - *Mechanism*: A unified ES module exposing:
    ```javascript
    export const CompoundEngineeringPlugin = async ({ project, client, $, directory, worktree }) => { ... };
    export default {
      id: "compound-engineering",
      server: CompoundEngineeringPlugin,
      async setup(ctx) { ... }
    };
    ```
  - *Trade-off Justification*:
    OpenCode V1 invokes `default.server` (and retains the named export `CompoundEngineeringPlugin`), while OpenCode V2 invokes `default.setup(ctx)`. No runtime version branching is needed inside the loader itself; the loader seamlessly works regardless of which OpenCode runtime executes it.

### 2.2 Command Strategy: Option (a) [Plugin Transform] vs. Option (b) [Materialized Files in `commands/`]
We evaluated how to ensure all `/ce-*` commands exist in V2:
- **Option (a): Register exclusively via `ctx.command.transform` in V2 plugin**
  - *Evaluation*: OpenCode V2's `ctx.command.transform` allows programmatic command addition. However, if plugin loading fails or is deferred, all commands vanish. Furthermore, OpenCode V2 cannot attach native markdown frontmatter parameters (such as `agent: ...`) to dynamically injected commands.
- **Option (b): Materialize commands as markdown files in `<config_dir>/commands/`**
  - *Evaluation*: OpenCode V2 natively watches `{command,commands}/**/*.md`. Materializing `<config_dir>/commands/<name>.md` for each user-invocable skill guarantees instantaneous, 100% reliable command availability in the UI and CLI without depending on asynchronous plugin initialization.
- **Decision: Synergistic Hybrid (Option b primary for V2, Option a fallback in plugin, V1 config hook preserved)**
  - *Implementation*:
    1. `ce-ai install` and `ce-ai sync` materialize `<config_dir>/commands/ce-*.md` with `<!-- ce-ai:managed-command -->` headers for all user-invocable CE skills.
    2. The dual plugin loader still includes `ctx.command.transform`, but skips any command already present in `ctx.command.list()`, ensuring total idempotence.
    3. The V1 `config` hook continues to populate `config.command[name]` for OpenCode 1.x users.
    4. `ce-ai uninstall` surgically removes the managed command markdown files.

---

## 3. Impact on OpenCode V1 Users (R5 Compliance)
A user running OpenCode 1.x experiences zero regressions:
1. `has_session_start_plugin()` checks both `"plugins"` and `"plugin"`, so existing V1 `"plugin"` configurations remain green.
2. The dual loader's `server: CompoundEngineeringPlugin` is invoked by V1's plugin loader, registering runtime commands via `config.command` and Turn-0 prompts via `client.session.prompt`.
3. OpenCode 1.x ignores the `<config_dir>/commands/` directory because it does not support file-based markdown command discovery.
4. `ce-ai doctor` evaluates the V1 loader capabilities and remains 100% green.
