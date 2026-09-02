# Brainstorming: Guaranteed Turn-0 Drift Delivery for OpenCode via Native Plugin Lifecycle Hooks

## 1. Problem Statement
In `v1.31.0` (PR #279), `ce-ai` introduced guaranteed Turn-0 `RepoState` drift synchronization for Claude Code via `.claude/settings.json` `SessionStart` hooks. For OpenCode, however, synchronization currently relies exclusively on the text directive in the managed block of `AGENTS.md` ("Turn-0 Session Directives"). If the AI agent ignores or forgets this prompt directive, the session starts blind to uncommitted changes, branch switches, and manifest drift, suffering from the 5–8 turns of observation lag documented in arXiv:2608.26263v2.

OpenCode possesses a first-class, deterministic plugin architecture with 25+ lifecycle events. Specifically:
- OpenCode plugins subscribe to lifecycle events via an `event` handler receiving `{ event }`.
- When a new session starts, OpenCode fires the `session.created` event.
- The plugin context provides `{ project, client, $, directory, worktree }`.
- Context can be injected via `client.session.prompt` (`noReply: true`), `experimental.chat.system.transform` (`output.system`), and `experimental.session.compacting` (`output.context`).

## 2. In-Scope vs Out-of-Scope Boundaries
### In-Scope
1. Complete OpenCode plugin implementation in `.opencode/plugins/compound-engineering.js`.
2. Dynamic skill discovery + command registration (preserving upstream loader compatibility) PLUS `session.created` event handling, `experimental.chat.system.transform`, and `experimental.session.compacting`.
3. Execution of `ce-ai workflow resume` within the session's workspace `directory` and injection of its live output into context.
4. Embedded builtin loader in `ce-ai` binary so installation and sync do not depend on external upstream tarballs having the hook.
5. Idempotent install and surgical uninstall (`has_session_start_plugin`, `ensure_session_start_plugin`, `remove_session_start_plugin`) preserving all custom user plugins and settings in `opencode.json`.
6. Doctor diagnostic audit in `src/commands/doctor.rs` checking OpenCode plugin health and flagging missing/outdated hooks.
7. Documentation updates in `zero-step-drift-recovery-explained.md` and `harnesses-loops-and-context-masterclass.md`.

### Out-of-Scope
- Harnesses without confirmed lifecycle hook APIs (Pi, Antigravity) — covered by Turn-0 prompt directives.
- Modifying OpenCode's core binary.

## 3. Key Design Decisions
- **KD1: Centralized Event Architecture:** Use OpenCode's recommended `event: async ({ event }) => { if (event.type === 'session.created') ... }` pattern.
- **KD2: Multi-Layer Context Injection:** In addition to `session.created`, hook into `experimental.session.compacting` so `RepoState` survives context compaction.
- **KD3: Embedded Canonical Loader:** Embed `.opencode/plugins/compound-engineering.js` via `include_str!` in `src/opencode/plugins.rs` to guarantee offline reliability and decouple from upstream tarball release cycles.
- **KD4: Global vs Project-Level Scope:** OpenCode loads plugins registered in `~/.config/opencode/opencode.json` for all workspaces. Therefore, `ce-ai install --harness opencode` / `sync` / `uninstall` remains the primary lifecycle controller, while `init-prj` and `doctor` verify adoption.
