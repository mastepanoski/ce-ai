# Proposal: Guaranteed Turn-0 Drift Delivery for OpenCode via Native Plugin Lifecycle Hooks

## Problem Statement
In `ce-ai v1.31.0`, Turn-0 `RepoState` synchronization was guaranteed for Claude Code via native `.claude/settings.json` `SessionStart` hooks. In OpenCode, synchronization remains an un-enforced textual directive in `AGENTS.md`. If the agent starts a session without invoking `ce-ai workflow resume`, it operates with stale assumptions regarding git branches, working tree diffs, manifest SHA256 hashes, and OpenSpec task progress.

OpenCode's plugin architecture provides official lifecycle events. By implementing a native OpenCode plugin subscribed to `session.created` and `experimental.session.compacting`, `ce-ai` can automatically execute `ce-ai workflow resume` and inject live `RepoState` into OpenCode sessions at Turn 0 without human intervention.

## In-Scope
1. Supply `.opencode/plugins/compound-engineering.js` in the `ce-ai` repository, implementing:
   - Dynamic skill discovery and slash-command registration.
   - `session.created` lifecycle event subscription invoking `ce-ai workflow resume`.
   - `experimental.session.compacting` hook injecting `RepoState` across context compaction.
   - Resilient context injection using `client.session.prompt` with `noReply: true`.
2. Embed the canonical loader inside the `ce-ai` binary (`BUILTIN_LOADER`) so `ce-ai install` and `ce-ai sync` do not rely on third-party upstream tarballs.
3. Idempotent lifecycle methods: `has_session_start_plugin`, `ensure_session_start_plugin`, `remove_session_start_plugin` with atomic writes (`write_atomic`) and preservation of user plugins in `opencode.json`.
4. Diagnostic checks in `ce-ai doctor` flagging missing or outdated OpenCode plugin hooks.
5. Updating user documentation and masterclass guides.

## Out-of-Scope
- Unsupported or unverified harness APIs (Pi, Antigravity).
- Modifying OpenCode internal core runtime.

## Risk Evaluation & Mitigation
- **Risk:** `ce-ai` command not in PATH when OpenCode plugin runs.
  - *Mitigation:* The plugin uses `child_process.spawnSync` wrapped in try/catch; if `ce-ai` fails or is not found, the plugin logs gracefully and does not block OpenCode startup.
- **Risk:** Duplicate execution if plugin is loaded globally and locally.
  - *Mitigation:* Single global registration in `~/.config/opencode/opencode.json` (`plugin[]`). `init-prj` does not duplicate the file into `.opencode/plugins/` when global installation is present.

## Success Criteria
- OpenCode plugin compiles/runs cleanly under Node.js and Bun.
- `ce-ai doctor` detects missing/outdated OpenCode plugin hooks.
- 100% test matrix passing across Linux, macOS, and Windows.
