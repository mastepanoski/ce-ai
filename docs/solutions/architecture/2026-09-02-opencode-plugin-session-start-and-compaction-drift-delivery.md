---
module: opencode
tags:
  - opencode
  - session-start
  - drift-delivery
  - plugins
  - compaction
  - repostate
problem_type: architecture
title: "OpenCode Native Plugin Lifecycle Hook & Compaction Drift Delivery"
applies_when: "When encountering issues related to opencode native plugin lifecycle hook & compaction drift delivery in architecture."
---

# OpenCode Native Plugin Lifecycle Hook & Compaction Drift Delivery

## Context & Problem
In `ce-ai v1.31.0`, Turn-0 `RepoState` drift synchronization was guaranteed automatically for Claude Code via `.claude/settings.json` `SessionStart` hooks. For OpenCode, synchronization was mediated exclusively through the prompt directive in `AGENTS.md`. If an LLM skipped or forgot this directive, it began its session blind to manifest drift, active branch switches, and OpenSpec tasks, leading to the 5–8 turns of observation lag noted in arXiv:2608.26263v2.

## Technical Solution

### 1. Canonical OpenCode Plugin (`.opencode/plugins/compound-engineering.js`)
We implemented a canonical OpenCode **V2** plugin (default export `{ id: "compound-engineering", setup(ctx) }`; V1 function exports no longer load under OpenCode V2) that integrates with the server runtime:
1. **Dynamic Skill & Command Registration (`setup` transforms):**
   Parses `SKILL.md` frontmatter from `../../skills` and registers each skill via `ctx.skill.transform` (`Skill.Info { id, name, description, path, content }`, idempotent via `editor.get` → `update`/`add`). Every `user-invocable` skill also gets a slash command via `ctx.command.transform` whose `execute` submits the preserved V1 template (`Load and execute the \`<name>\` skill.\n\n$ARGUMENTS`, arguments := `prompt.text`) through `ctx.session.prompt`; names already present in `ctx.command.list()` keep precedence.
2. **Deterministic Turn-0 Resumption (`ctx.event.subscribe`):**
   Subscribes to the public event stream (aborted via `AbortController` on unload). On `event.type === 'session.created'` it resolves the top-level `event.sessionID`, runs `ce-ai workflow resume` within `ctx.location.directory`, and delivers live state context via:
   ```javascript
   await ctx.session.synthetic({ sessionID, text: stateOutput });
   ```
   `synthetic` is the V2-native context-only message (the replacement for the retired V1 `client.session.prompt({ noReply: true })` idiom).
3. **Compaction Survival (`ctx.session.hook("context")` + `ctx.session.hook("compaction")`):**
   Appends live `RepoState` to `event.system` for every agent-loop request and every checkpoint-summary request, so canonical drift status survives context compaction and is re-injected fresh on the first post-compaction model call.
4. **Turn-End Checkpoints (`session.idle`):**
   Runs `ce-ai workflow resume` on every `session.idle` event to evaluate 7-stage FSM progression (output discarded; side effect only).

### 2. Embedded Builtin Loader & Decoupled Reliability
To eliminate external dependencies on upstream release tarballs (`everyinc/compound-engineering-plugin`), `src/opencode/plugins.rs` embeds the canonical loader via `include_str!("../../.opencode/plugins/compound-engineering.js")`.
- `install_loader`: If the source loader fails content validation (must carry the `session.created` SessionStart marker AND the V2 `setup` signature — V1 function exports are classified as stale so OpenCode V2's "must export a default definition with an id and an effect or setup function" error cannot recur), it automatically uses `BUILTIN_LOADER`. Legacy `ceLoader` function exports remain valid per the #325 loader-safety guarantee (sync/upgrade never regress a newer loader).
- `ensure_session_start_plugin`: Idempotently writes `BUILTIN_LOADER` using `write_atomic` and registers the plugin in `opencode.json`.
- `remove_session_start_plugin`: Surgically strips managed entries (`plugin`, `skills.paths`, `agent`) and cleans up the file if no user configurations remain, preserving custom user configurations.

### 3. Doctor Health Audit
`ce-ai doctor` inspects `has_session_start_plugin(&ctx.opencode_config_dir)` when OpenCode is registered in `state.installed_harnesses`. If the plugin loader is missing, tampered, or unregistered in `opencode.json`, `doctor` emits a clear actionable finding.

## Verification
- Node.js ESM validation passes syntax and structural export checks.
- 100% unit tests passing in `src/opencode/tests/plugins.rs`.
- Integration tests in `tests/cli.rs` verify tampering detection and self-healing via `ce-ai sync`.
- `make e2e` containerized validation passes all 10 real-world execution gates.
