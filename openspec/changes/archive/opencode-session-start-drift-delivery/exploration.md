# Exploration: OpenCode Plugin Lifecycle Architecture & Context Injection

## 1. Technical Investigation of OpenCode Plugin Architecture

According to official OpenCode documentation (`https://open-code.ai/en/docs/plugins`) and the `@opencode-ai/plugin` SDK:
- **Plugin Entry Point:** An ECMAScript module exporting `default` and/or named `CompoundEngineeringPlugin`.
- **Plugin Function Context:**
  ```typescript
  export const CompoundEngineeringPlugin = async ({ project, client, $, directory, worktree }) => {
    return { ... };
  };
  ```
- **Lifecycle Events (`event` hook):**
  Plugins return an `event` async function receiving `{ event }`:
  - `event.type === 'session.created'`: Fired when a new user or agent session is initialized.
  - The session ID is available at `event.properties?.info?.id || event.properties?.sessionID || event.sessionID`.
- **Programmatic Context Delivery:**
  The `client` SDK object exposes `client.session.prompt`:
  ```typescript
  await client.session.prompt({
    path: { id: sessionId },
    body: {
      noReply: true,
      parts: [{ type: 'text', text: contextText }]
    }
  });
  ```
  `noReply: true` informs OpenCode that the message is injected context/instruction, not a user query awaiting turn execution.
- **Compaction Hook (`experimental.session.compacting`):**
  OpenCode triggers `experimental.session.compacting` before generating continuation summaries. Plugins receive `(input, output)` where `output.context` is an array of strings appended to the continuation prompt.

## 2. Source Loader Resolution in `ce-ai`

Investigation of `src/commands/install.rs` and `src/opencode/plugins.rs`:
- `install_loader(&source_path, &config_dir)` reads `.opencode/plugins/compound-engineering.js` from `source_path`.
- In production, `source_path` is extracted from the upstream GitHub tarball `everyinc/compound-engineering-plugin`.
- The upstream repository currently only registers `config.skills.paths` and `config.command`; it does not contain the `session.created` hook.
- If `ce-ai` relies exclusively on `source_path.join(SOURCE_LOADER_PATH)`, users would be tethered to upstream release cycles to receive `ce-ai`'s drift recovery features.
- **Tradeoff Decision:** Embed the canonical plugin JS in `src/opencode/plugins.rs` using `include_str!("../../../.opencode/plugins/compound-engineering.js")`. When `install_loader` runs, if the source file lacks `session.created` or is missing, use the embedded canonical loader. This guarantees determinism, zero drift, and offline installation safety.

## 3. Global vs Workspace Deployment Semantics

- **Claude Code:** Hooks are placed in `<workspace>/.claude/settings.json` because Claude executes hooks defined per-project.
- **OpenCode:** Plugins declared in `~/.config/opencode/opencode.json` (`plugin[]`) are global; they automatically activate in every workspace.
- **Resolution:**
  - `ce-ai install --harness opencode` installs the plugin globally.
  - `ce-ai sync` refreshes and verifies the plugin SHA256.
  - `ce-ai uninstall --harness opencode` surgically cleans up the plugin without touching user plugins.
  - `ce-ai doctor` verifies that if OpenCode is in `state.installed_harnesses`, the plugin is healthy and registered.
