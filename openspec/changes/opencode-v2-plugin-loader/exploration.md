# Exploration: OpenCode V2 Plugin API Migration

## Evidence Gathered (installed OpenCode v2.0.18, `/home/otta/.opencode/bin/opencode`)

1. **Load contract**: the binary contains the exact validation error
   `Plugin must export a default definition with an id and an effect or setup function.`
   The V2 docs (`/v2/docs/build/plugins`) define the shape as
   `export default Plugin.define({ id, async setup(ctx) {...} })`; for plain `.js` files
   without the `@opencode/plugin` import, the structurally identical object literal
   `{ id, setup }` satisfies the same check.
2. **Config normalization**: minified config-loading code in the binary shows legacy keys
   are migrated at load time:
   `skills: e.skills && [...e.skills.paths ?? [], ...e.skills.urls ?? []],
   commands: wd(e.command), ..., plugins: e.plugin`.
   → The ce-ai-managed `plugin` + `skills.paths` config remains functional on v2.0.18;
   no config-key migration is required for this fix.
   Confirmed against the live schema: `Config.InfoEncoded` exposes `plugins` (array of
   string | `{package, options}`) and `skills` (array of strings).
3. **Event shapes** (from the binary's event schema table):
   - `session.created` → `{ type, sessionID, info, ... }` (top-level `sessionID`).
   - `session.idle` → `{ type, sessionID }`.
4. **Context injection idiom**: OpenCode's own built-in Plan-mode reminder does
   `e.session.synthetic({ sessionID: s.data.sessionID, text, resume: !1 })` on
   `session.created` — the V2-native replacement for V1's
   `client.session.prompt({ body: { noReply: true, parts } })` (the V2
   `session.prompt` request body no longer has `noReply`; `Session.Inbox.Synthetic` is the
   dedicated context-only message type).
5. **Hook API** (V2 docs + OpenAPI):
   - `ctx.session.hook("context", (event) => event.system.push({type:"text", text}))`
     replaces `experimental.chat.system.transform`.
   - `ctx.session.hook("compaction", ...)` receives the same context-hook event shape for
     the checkpoint-summary request — the closest V2 equivalent of V1's
     `experimental.session.compacting` (`output.context` no longer exists; injecting into
     the compaction request's system context keeps the summary state-aware, and the
     `context` hook guarantees fresh state is delivered again on the first post-compaction
     model call).
   - `ctx.event.subscribe({ signal })` replaces the V1 `event` hook; cleanup via
     `AbortController` in the function returned from `setup`.
   - `ctx.location.directory` replaces the V1 `directory` plugin-context argument.
   - `ctx.command.transform(editor => editor.add({ name, description, execute }))` with
     `execute({ sessionID, prompt, delivery })` → `ctx.session.prompt({...})` replaces the
     V1 `config.command[name] = { template, description }`; TUI command submission puts
     the arguments in `text` (observed: `api.session.command({sessionID, name, text:
     lr.arguments, ...})`), so `prompt.text` is the V2 equivalent of `$ARGUMENTS`.
   - `ctx.skill.transform` (`Skill.Info { id, name, description?, autoinvoke?, path,
     content }`) replaces the `config.skills.paths` push.
6. **Live failure**: the user's server reports the repo-local plugin
   (`~/Desarrollo/ce-ai/.opencode/plugins/compound-engineering.js`, auto-discovered from
   `.opencode/plugins/`) as failed with Reference `err_c926271d`. The same V1 file is
   embedded in the ce-ai binary (`include_str!`) and installed into
   `~/.config/opencode/compound-engineering/plugins/`.

## Options Evaluated

### Option A — Port only the plugin file (minimal)

Rewrites the loader to `{ id, setup }`. Fixes the immediate load failure for the repo
checkout and future installs.

- *Rejected as incomplete*: installed users keep the stale V1 file on disk;
  `is_valid_loader_content` still classifies it as valid (`session.created` substring), so
  `doctor` reports healthy and `ensure_session_start_plugin` never rewrites it. Only a
  post-upgrade `ce-ai sync` (sha drift against the new source tree) would repair it,
  silently.

### Option B — Port the plugin file AND tighten loader validity detection (chosen)

Option A plus a stricter `is_valid_loader_content`: require the V2 signature
(`session.created` **and** `setup`), while still accepting function-export `ceLoader*`
stubs (test fixtures, #325 loader-safety contract). This makes every ce-ai surface
(install fallback, sync restore, ensure-session-start self-heal, doctor audit) treat V1
loaders as outdated and converge them to the embedded V2 builtin.

### Option C — Dual V1/V2 export shape (like the docs' "Support V1 and V2 from one package")

The migration guide shows `{ ...Plugin.define(...), async server() {...} }` for packages.
- *Rejected*: the loader is a plain `.js` file, not a package importing
  `@opencode/plugin`; the dual-shape exists for npm packages serving both runtimes. V1
  OpenCode is end-of-life for this harness path (V2 is the documented current version),
  and the V2 check only inspects the default export — a plain V2 object is sufficient.

## Explicitly Ruled Out

- **Config key migration (`plugin` → `plugins`)**: evidence (2) shows v2.0.18 normalizes
  it; changing ce-ai's writers would touch install/sync/doctor/tests for zero functional
  gain today.
- **Keeping the V1 `client.session.prompt({noReply})` shape**: the field does not exist in
  the V2 API (verified in the OpenAPI request schema); `synthetic` is the supported
  mechanism, proven by OpenCode's own built-in usage.
- **Dropping the command shim because V2 lists skills natively**: the repo's design
  documents command registration as the contract; the shim also carries the exact
  `Load and execute the \`<skill>\` skill.` prompt template. Kept for parity; native
  skill invocation remains available independently.
