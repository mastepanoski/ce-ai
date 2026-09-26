# Design: OpenCode V2 Plugin Loader

## Architecture

The canonical loader remains a single dependency-free ESM file at
`.opencode/plugins/compound-engineering.js`, embedded into the ce-ai binary via
`include_str!("../../.opencode/plugins/compound-engineering.js")`
(`src/opencode/plugins.rs::BUILTIN_LOADER`) and installed to
`<opencode-config>/compound-engineering/plugins/compound-engineering.js`.

### V1 → V2 mapping

| V1 mechanism (current file) | V2 replacement (ported file) |
| --- | --- |
| Default export: async function returning hooks object | Default export: `{ id: "compound-engineering", async setup(ctx) {...} }` |
| `config` hook: `config.skills.paths.push(skillsDir)` | `await ctx.skill.transform(...)` — one `Skill.Info` per discovered `SKILL.md` (`id`/`name` from frontmatter `name`, `description`, `path` = absolute SKILL.md path, `content` = full file). `user-invocable: false` skills are still registered as skills (V1 parity). Idempotent via `editor.get(id)` → `update`/`add`. |
| `config` hook: `config.command[name] = { template, description }` | `await ctx.command.transform(...)` — `CommandDefinition { name, description?, execute }`. `execute({ sessionID, prompt, delivery })` expands the V1 template (`Load and execute the \`<name>\` skill.\n\n$ARGUMENTS`, `$ARGUMENTS` := `prompt.text`) and calls `ctx.session.prompt({ sessionID, text, delivery })`. Names already present in `ctx.command.list()` are skipped (user config precedence, mirrors V1 `!(name in config.command)`). `user-invocable: false` skills get no command. |
| `event` hook: `session.created` → `client.session.prompt({ noReply: true, parts })` | `ctx.event.subscribe({ signal })` → on `session.created`, `ctx.session.synthetic({ sessionID, text })`. Session ID read from top-level `event.sessionID` (defensive fallback to legacy `properties` shapes). |
| `event` hook: `session.idle` → `getRepoState(cwd)` side effect | Same, inside the subscription loop (checkpoint evaluation side effect preserved). |
| `experimental.chat.system.transform` → `output.system.push(...)` | `await ctx.session.hook("context", (event) => event.system.push({ type: "text", text }))` |
| `experimental.session.compacting` → `output.context.push(...)` | `await ctx.session.hook("compaction", (event) => event.system.push(...))` — keeps the checkpoint-summary request state-aware; post-compaction continuity is guaranteed because the `context` hook re-injects fresh state on the next agent-loop request. |
| Plugin ctx `directory`/`worktree` | `ctx.location.directory` (fallback `process.cwd()`) |
| (no equivalent) | Cleanup: `setup` returns `() => controller.abort()` so the event subscription stops on plugin unload. |

### Module-level behavior

- Top-level code only resolves `pluginDir`/`skillsDir` constants. Skill discovery
  (`loadSkills`) and every subprocess/filesystem interaction happen inside `setup` or
  hooks, wrapped in the same graceful `try/catch` guards as V1 (plugin load must never
  fail because `ce-ai` is missing from PATH or a directory is absent).
- The named export `CompoundEngineeringPlugin` is dropped: OpenCode only consumes the
  default export, and no test or document imports the named symbol.

## Loader validity (`src/opencode/plugins.rs`)

```rust
fn is_valid_loader_content(content: &str) -> bool {
    content.contains("session.created")
        && (content.contains("setup") || content.starts_with("export default function ceLoader"))
}
```

- The V2 canonical file satisfies both terms (`session.created` event filter, `setup`).
- The exact V1 file is rejected (no `setup` substring, no `ceLoader` prefix) →
  `resolve_loader_bytes` substitutes `BUILTIN_LOADER`, `ensure_session_start_plugin`
  rewrites it, and `doctor` reports "SessionStart plugin missing or outdated".
- `export default function ceLoader...` stubs (unit-test fixtures, Dockerfile.e2e, and
  the v9 upgrade tarball simulating a newer function-export loader) remain valid,
  preserving the #325 guarantee that sync/upgrade never regress a newer loader.

## Data Flow (unchanged from V1)

1. Plugin loads at location startup → skills/commands registered.
2. `session.created` → `ce-ai workflow resume` (5s timeout) → output injected as a
   synthetic context message.
3. Every model request (`context` hook) and checkpoint summary (`compaction` hook) →
   current state appended to system context.
4. `session.idle` → `ce-ai workflow resume` evaluates FSM stage progression.

## Testing

- `src/opencode/tests/plugins.rs`: new cases —
  - `is_valid_loader_content` accepts the real V2 loader (via `resolve_loader_bytes` on a
    fixture source tree), rejects the real V1 loader shape, and still accepts
    `export default function ceLoader() {}` / `ceLoaderV9` fixtures.
  - `has_session_start_plugin` returns false when the managed loader on disk is a
    V1-format file (doctor surfaces the actionable finding).
- Existing suite (`tests/cli.rs`, `src/opencode/tests/*`) must stay green: all loader
  fixtures use `ceLoader*` stubs except the v9 tarball, which matches the preserved
  branch.
- Standalone load check: import the file with Node and assert `default.id` +
  `typeof default.setup === "function"`.
- Isolated live check: `XDG_CONFIG_HOME=<tmp>` OpenCode v2.0.18 instance loads the plugin
  (no err_c926271d) and lists registered skills/commands.
- Full gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D
  warnings`, `cargo test`, `make e2e`.

## Affected Files

- `.opencode/plugins/compound-engineering.js` (rewritten, ~150 LOC)
- `src/opencode/plugins.rs` (validity function + doc comment, ~8 LOC)
- `src/opencode/tests/plugins.rs` (new unit tests, ~60 LOC)
- `docs/solutions/architecture/2026-09-02-opencode-plugin-session-start-and-compaction-drift-delivery.md`
- `docs/user-guide/fsm-and-checkpoints-explained.md`,
  `docs/user-guide/harnesses-loops-and-context-masterclass.md`,
  `docs/user-guide/zero-step-drift-recovery-explained.md` (mechanism references)
- `CHANGELOG.md`, `Cargo.toml` (1.72.0 → 1.72.1)
