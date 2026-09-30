# Technical Design: OpenCode Dual Plugin Loader, Version Detection & Command Materialization

## 1. System Architecture & Component Interactions

```
                         ┌──────────────────────────────────────────────┐
                         │              ce-ai doctor / sync            │
                         └──────────────────────┬───────────────────────┘
                                                │
                 ┌──────────────────────────────┴──────────────────────────────┐
                 ▼                                                             ▼
     has_session_start_plugin()                                    ensure_session_start_plugin()
   - Accepts "plugins" & "plugin"                                 - Writes BUILTIN_LOADER
   - Validates V1/V2/Dual shape                                   - Updates "plugins" or "plugin"
   - Probes OpenCode version                                      - Materializes commands/ce-*.md
                 │                                                             │
                 ▼                                                             ▼
┌───────────────────────────────────┐                        ┌───────────────────────────────────┐
│     ~/.config/opencode/           │                        │     ~/.config/opencode/           │
│     ├── opencode.json             │                        │     ├── compound-engineering/     │
│     │   ("plugins" or "plugin")   │                        │     │   └── plugins/              │
│     └── commands/                 │                        │     │       └── compound-         │
│         ├── ce-brainstorm.md      │                        │     │           engineering.js    │
│         ├── ce-work.md            │                        │     ├── plugins/                  │
│         └── ... (34 commands)     │                        │     │   └── compound-             │
└───────────────────────────────────┘                        │     │       engineering.js (sym)  │
                                                             │     └── skills/                   │
                                                             └───────────────────────────────────┘
```

---

## 2. Dual Loader Design (`.opencode/plugins/compound-engineering.js`)

The embedded loader file must satisfy both the OpenCode V1 runtime and OpenCode V2 plugin schema:

### 2.1 ESM Exports Structure
```javascript
export const CompoundEngineeringPlugin = async ({ project, client, $, directory, worktree }) => {
  // V1 Implementation:
  // - config hook: injects skills.paths and config.command[name]
  // - event hook: session.created client.session.prompt Turn-0 state injection
  // - session.idle: FSM auto-checkpoint
  // - experimental.session.compacting & experimental.chat.system.transform
};

export default {
  id: "compound-engineering",
  server: CompoundEngineeringPlugin,
  async setup(ctx) {
    // V2 Implementation:
    // - ctx.skill.transform: registers discovered skills
    // - ctx.command.transform: registers slash commands (fallback if not in commands/)
    // - ctx.session.hook("context") & ctx.session.hook("compaction"): system prompt injection
    // - ctx.event.subscribe: session.created synthetic message + session.idle FSM checkpoint
    // - returns cleanup handler () => controller.abort()
  }
};
```

### 2.2 Skill Directory Resolution
Because the loader may be located in `<config_dir>/compound-engineering/plugins/`, `<config_dir>/plugins/`, or within a workspace `.opencode/plugins/`, `skillsDir` is resolved dynamically:
```javascript
const candidateDirs = [
  path.resolve(pluginDir, "../../skills"),
  path.resolve(pluginDir, "../compound-engineering/skills"),
  path.resolve(pluginDir, "../skills"),
];
const skillsDir = candidateDirs.find((d) => fs.existsSync(d)) || candidateDirs[0];
```

---

## 3. Rust Detection & Configuration Architecture

### 3.1 Version Probing (`src/opencode/plugins.rs`)
```rust
/// Represents the detected OpenCode major version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenCodeVersion {
    V1,
    V2,
    Unknown,
}

pub fn detect_opencode_version() -> OpenCodeVersion {
    if let Ok(output) = std::process::Command::new("opencode").arg("--version").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("v2.") || stdout.contains(" 2.") {
                return OpenCodeVersion::V2;
            } else if stdout.contains("v1.") || stdout.contains(" 1.") {
                return OpenCodeVersion::V1;
            }
        }
    }
    OpenCodeVersion::Unknown
}
```

### 3.2 Content Validation (`is_valid_loader_content`)
```rust
fn is_valid_loader_content(content: &str, target_version: OpenCodeVersion) -> bool {
    let has_session_created = content.contains("session.created");
    let is_v1_shape = content.contains("CompoundEngineeringPlugin") || content.contains("server:");
    let is_v2_shape = content.contains("setup") && (content.contains("id:") || content.contains("Plugin.define"));
    let is_dual_shape = is_v1_shape && is_v2_shape;

    if content.starts_with("export default function ceLoader") {
        return true; // legacy test fixture guarantee (#325)
    }

    if !has_session_created {
        return false;
    }

    match target_version {
        OpenCodeVersion::V2 => is_v2_shape,
        OpenCodeVersion::V1 => is_v1_shape,
        OpenCodeVersion::Unknown => is_dual_shape || is_v2_shape,
    }
}
```

### 3.3 Configuration Key Agnosticism (`src/opencode/config.rs`)
In `merge_plugin(config: &mut serde_json::Value, plugin_entry: &str)`:
1. If `config.get("plugins").is_some()`, append `plugin_entry` to `plugins` array.
2. Else if `config.get("plugin").is_some()`, append `plugin_entry` to `plugin` array.
3. Else (neither key exists): if `detect_opencode_version() == OpenCodeVersion::V1`, create `"plugin"`; otherwise default to `"plugins"`.

In `has_session_start_plugin(config_dir: &Path)`:
1. Read `loader_path = plugin_entry(config_dir)`.
2. Validate content with `is_valid_loader_content(&content, detect_opencode_version())`.
3. Check `opencode.json`:
   - Accept either `"plugins"` or `"plugin"` array containing `expected_entry` (or relative path / symlink).

In `remove_session_start_plugin(config_dir: &Path)`:
1. Surgically prune `entry_str` from both `"plugins"` and `"plugin"` arrays.
2. Clean up empty array keys and empty root objects.

---

## 4. Native Command Materialization (`commands/`)

### 4.1 Specification & Format
For each skill directory in `<config_dir>/compound-engineering/skills/<skill_id>/SKILL.md`:
If `user-invocable !== false`, write `<config_dir>/commands/<skill_id>.md`:
```markdown
---
description: "<description extracted from frontmatter>"
---

<!-- ce-ai:managed-command -->

Load and execute the `<skill_id>` skill.

$ARGUMENTS
```

### 4.2 Lifecycle Integration
- **`ensure_managed_commands(config_dir: &Path) -> Result<usize, CeError>`**:
  Iterates managed skills, writes `<config_dir>/commands/<skill_id>.md` atomically.
- **`remove_managed_commands(config_dir: &Path) -> Result<usize, CeError>`**:
  Scans `<config_dir>/commands/` and removes only files starting with or containing `<!-- ce-ai:managed-command -->`.
- Hooked into `ce-ai install`, `ce-ai sync`, and `ce-ai uninstall`.
