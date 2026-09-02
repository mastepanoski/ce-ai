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
---

# OpenCode Native Plugin Lifecycle Hook & Compaction Drift Delivery

## Context & Problem
In `ce-ai v1.31.0`, Turn-0 `RepoState` drift synchronization was guaranteed automatically for Claude Code via `.claude/settings.json` `SessionStart` hooks. For OpenCode, synchronization was mediated exclusively through the prompt directive in `AGENTS.md`. If an LLM skipped or forgot this directive, it began its session blind to manifest drift, active branch switches, and OpenSpec tasks, leading to the 5–8 turns of observation lag noted in arXiv:2608.26263v2.

## Technical Solution

### 1. Canonical OpenCode Plugin (`.opencode/plugins/compound-engineering.js`)
We implemented a canonical plugin exporting `CompoundEngineeringPlugin` and `default` that integrates with OpenCode's event-driven runtime:
1. **Dynamic Skill Discovery & Command Registration (`config` hook):**
   Parses `SKILL.md` frontmatter from `../../skills` and populates `config.skills.paths` and `config.command`.
2. **Deterministic Turn-0 Resumption (`event` hook):**
   Subscribes to `event.type === 'session.created'`. Resolves the session identifier (`event.properties?.info?.id || event.properties?.sessionID || event.sessionID`), runs `ce-ai workflow resume` within the session's workspace directory, and delivers live state context via:
   ```javascript
   await client.session.prompt({
     path: { id: sessionId },
     body: {
       noReply: true,
       parts: [{ type: "text", text: stateOutput }],
     },
   });
   ```
3. **Compaction Survival (`experimental.session.compacting` hook):**
   Appends live `RepoState` to `output.context` before OpenCode synthesizes its continuation summary.
4. **Prompt Transformation (`experimental.chat.system.transform` hook):**
   Provides an additional layer of state delivery into system context where supported.

### 2. Embedded Builtin Loader & Decoupled Reliability
To eliminate external dependencies on upstream release tarballs (`everyinc/compound-engineering-plugin`), `src/opencode/plugins.rs` embeds the canonical loader via `include_str!("../../.opencode/plugins/compound-engineering.js")`.
- `install_loader`: If the source loader lacks `session.created`, it automatically uses `BUILTIN_LOADER`.
- `ensure_session_start_plugin`: Idempotently writes `BUILTIN_LOADER` using `write_atomic` and registers the plugin in `opencode.json`.
- `remove_session_start_plugin`: Surgically strips managed entries (`plugin`, `skills.paths`, `agent`) and cleans up the file if no user configurations remain, preserving custom user configurations.

### 3. Doctor Health Audit
`ce-ai doctor` inspects `has_session_start_plugin(&ctx.opencode_config_dir)` when OpenCode is registered in `state.installed_harnesses`. If the plugin loader is missing, tampered, or unregistered in `opencode.json`, `doctor` emits a clear actionable finding.

## Verification
- Node.js ESM validation passes syntax and structural export checks.
- 100% unit tests passing in `src/opencode/tests/plugins.rs`.
- Integration tests in `tests/cli.rs` verify tampering detection and self-healing via `ce-ai sync`.
- `make e2e` containerized validation passes all 10 real-world execution gates.
