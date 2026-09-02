# Technical Design: Guaranteed Turn-0 Drift Delivery for OpenCode

## 1. System Architecture

```
┌────────────────────────────────────────────────────────┐
│               OpenCode Runtime Session                 │
└──────────────────────────┬─────────────────────────────┘
                           │ 1. Triggers session.created event
                           ▼
┌────────────────────────────────────────────────────────┐
│  ~/.config/opencode/compound-engineering/plugins/     │
│             compound-engineering.js                    │
└──────────────────────────┬─────────────────────────────┘
                           │ 2. Runs `ce-ai workflow resume`
                           │    in project workspace directory
                           ▼
┌────────────────────────────────────────────────────────┐
│                 ce-ai CLI Binary                       │
│    (Probes live Git tree, SHA manifest, OpenSpec)      │
└──────────────────────────┬─────────────────────────────┘
                           │ 3. Returns live RepoState summary
                           ▼
┌────────────────────────────────────────────────────────┐
│       client.session.prompt({ noReply: true })         │
│     (Injected into agent context at Turn 0)            │
└────────────────────────────────────────────────────────┘
```

## 2. Plugin Contract: `.opencode/plugins/compound-engineering.js`

```javascript
import path from "path";
import fs from "fs";
import { spawnSync } from "child_process";
import { fileURLToPath } from "url";

const pluginDir = path.dirname(fileURLToPath(import.meta.url));
const skillsDir = path.resolve(pluginDir, "../../skills");

// 1. Skill parsing and command registration (backward-compatible)
// ...

// 2. Helper to run `ce-ai workflow resume`
function getRepoState(cwd) {
  try {
    const res = spawnSync("ce-ai", ["workflow", "resume"], {
      cwd: cwd || process.cwd(),
      encoding: "utf8",
      timeout: 5000,
    });
    if (res.status === 0 && res.stdout) {
      return res.stdout.trim();
    }
  } catch {
    // Fail gracefully if ce-ai is not on PATH or execution fails
  }
  return null;
}

export const CompoundEngineeringPlugin = async ({ project, client, $, directory, worktree }) => {
  const cwd = directory || worktree || process.cwd();

  return {
    // Config hook: register skills and commands
    config: async (config) => {
      config.skills = config.skills || {};
      config.skills.paths = config.skills.paths || [];
      if (!config.skills.paths.includes(skillsDir)) {
        config.skills.paths.push(skillsDir);
      }
      config.command = config.command || {};
      for (const [name, cmd] of Object.entries(skillCommands)) {
        if (!(name in config.command)) {
          config.command[name] = cmd;
        }
      }
    },

    // Centralized event listener
    event: async ({ event }) => {
      if (event.type === "session.created") {
        const sessionId = event.properties?.info?.id || event.properties?.sessionID || event.sessionID;
        const stateOutput = getRepoState(cwd);
        if (sessionId && stateOutput && client?.session?.prompt) {
          try {
            await client.session.prompt({
              path: { id: sessionId },
              body: {
                noReply: true,
                parts: [{ type: "text", text: stateOutput }],
              },
            });
          } catch {
            // Ignore prompt delivery failure
          }
        }
      }
    },

    // Compaction survival hook
    "experimental.session.compacting": async (input, output) => {
      const stateOutput = getRepoState(cwd);
      if (stateOutput && output && Array.isArray(output.context)) {
        output.context.push(stateOutput);
      }
    },
  };
};

export default CompoundEngineeringPlugin;
```

## 3. Rust Engine Integration (`src/opencode/plugins.rs`)

### 3.1 Embedded Builtin Loader
```rust
pub const BUILTIN_LOADER: &str = include_str!("../../../.opencode/plugins/compound-engineering.js");
```

### 3.2 Canonical Plugin Helpers
1. `has_session_start_plugin(config_dir: &Path) -> bool`:
   Returns true if `plugin_entry(config_dir)` exists, contains `"session.created"`, and is registered in `opencode.json` (`plugin[]`).
2. `ensure_session_start_plugin(config_dir: &Path) -> Result<bool, CeError>`:
   Writes `BUILTIN_LOADER` to `plugin_entry(config_dir)` via `write_atomic` if missing or outdated, and merges path into `opencode.json` via `merge_plugin`.
3. `remove_session_start_plugin(config_dir: &Path) -> Result<bool, CeError>`:
   Removes the loader path from `opencode.json` (`plugin[]`) while preserving all custom user plugins, and deletes the loader file.

## 4. Health Diagnostics (`src/commands/doctor.rs`)

```rust
if state.installed_harnesses.iter().any(|h| h.name == "opencode") {
    if !crate::opencode::plugins::has_session_start_plugin(&ctx.opencode_config_dir) {
        findings.push(format!(
            "opencode: SessionStart plugin missing or outdated in '{}' — run 'ce-ai sync' or 'ce-ai install --harness opencode' to update",
            ctx.opencode_config_dir.display()
        ));
    }
}
```
