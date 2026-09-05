# Design: MCP Companion Tools & Skill Suggestions Detection

## Architecture & Shared Helper

In `src/source/tools_registry.rs`, introduce three helper functions:

```rust
/// Collects candidate MCP configuration paths across active and supported harnesses.
pub fn find_mcp_config_paths(ctx: &Context) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let state_path = ctx.config_dir.join("state.json");
    let state = crate::state::state::State::load(&state_path).unwrap_or_default();

    // 1. OpenCode configuration (workspace-resolved or global)
    let opencode_dir = ctx.resolve_opencode_dir(&state);
    paths.push(opencode_dir.join("opencode.json"));
    if opencode_dir != ctx.opencode_config_dir {
        paths.push(ctx.opencode_config_dir.join("opencode.json"));
    }

    // 2. Workspace root configs
    if let Some(ws) = &ctx.workspace_root {
        paths.push(ws.join("opencode.json"));
        paths.push(ws.join(".cursor").join("mcp.json"));
        paths.push(ws.join(".claude.json"));
    }

    // 3. Native harness configurations
    let home_dir = crate::harness::home_dir_from_ctx(ctx);
    for entry in &state.installed_harnesses {
        let Some(name) = entry["name"].as_str() else { continue; };
        if let Ok(kind) = name.parse::<crate::harness::HarnessKind>() {
            match kind {
                crate::harness::HarnessKind::Cursor => {
                    paths.push(home_dir.join(".cursor").join("mcp.json"));
                }
                crate::harness::HarnessKind::Claude => {
                    paths.push(crate::harness::claude::ClaudeAdapter.default_config_path(&home_dir));
                }
                crate::harness::HarnessKind::Codex => {
                    paths.push(crate::harness::codex::CodexAdapter.default_config_path(&home_dir));
                }
                crate::harness::HarnessKind::Copilot => {
                    paths.push(crate::harness::copilot::CopilotAdapter.default_config_path(&home_dir));
                }
                crate::harness::HarnessKind::Kimi => {
                    paths.push(crate::harness::kimi::KimiAdapter.default_config_path(&home_dir));
                }
                crate::harness::HarnessKind::Agy => {
                    paths.push(crate::harness::agy::AgyAdapter.default_config_path(&home_dir));
                }
                crate::harness::HarnessKind::Fx => {
                    paths.push(crate::harness::fx::FxAdapter.default_config_path(&home_dir));
                }
                _ => {}
            }
        }
    }

    paths.push(crate::harness::claude::ClaudeAdapter.default_config_path(&home_dir));
    paths.push(home_dir.join(".cursor").join("mcp.json"));

    let mut seen = std::collections::HashSet::new();
    paths.retain(|p| seen.insert(p.clone()));
    paths
}

/// Checks whether an MCP server matching `name` is registered in any candidate harness configuration.
pub fn is_mcp_server_configured(ctx: &Context, name: &str) -> bool {
    let norm_target = name.to_lowercase();
    let stripped_target = norm_target.replace('-', "").replace('_', "");

    for path in find_mcp_config_paths(ctx) {
        if !path.exists() { continue; }
        let Ok(content) = std::fs::read_to_string(&path) else { continue; };
        if !content.contains("mcpServers") && !content.contains("mcp_servers") { continue; }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else { continue; };

        let mcp_obj = val.get("mcpServers").or_else(|| val.get("mcp_servers"));
        if let Some(serde_json::Value::Object(map)) = mcp_obj {
            for (key, server_val) in map {
                let norm_key = key.to_lowercase();
                let stripped_key = norm_key.replace('-', "").replace('_', "");

                if norm_key == norm_target || stripped_key == stripped_target {
                    return true;
                }

                if let Some(cmd) = server_val.get("command").and_then(|c| c.as_str()) {
                    if cmd.to_lowercase().contains(&norm_target) {
                        return true;
                    }
                }
                if let Some(serde_json::Value::Array(args)) = server_val.get("args") {
                    for arg in args {
                        if let Some(s) = arg.as_str() {
                            if s.to_lowercase().contains(&norm_target) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Checks whether a skill suggestion is configured either as an MCP server or in the skill registry.
pub fn is_skill_configured(ctx: &Context, name: &str) -> bool {
    if is_mcp_server_configured(ctx, name) {
        return true;
    }
    let registry_path = ctx.config_dir.join("skills-registry.json");
    if let Ok(reg) = crate::source::registry::SkillRegistry::load(&registry_path) {
        if reg.skills.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
            return true;
        }
    }
    if let Some(ws) = &ctx.workspace_root {
        let ws_reg_path = ws.join(".ce-ai").join("skills-registry.json");
        if let Ok(reg) = crate::source::registry::SkillRegistry::load(&ws_reg_path) {
            if reg.skills.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
                return true;
            }
        }
    }
    false
}

/// Evaluates tool freshness comparing installed CLI version or MCP server presence.
pub fn detect_tool_freshness(
    ctx: &Context,
    tool_name: &str,
    info: &CompanionToolInfo,
) -> FreshnessStatus {
    if let Some(installed_ver) = extract_tool_version(tool_name) {
        return evaluate_freshness(Some(&installed_ver), &info.latest_version);
    }
    if is_mcp_server_configured(ctx, tool_name) {
        return FreshnessStatus::Ok {
            version: info.latest_version.clone(),
        };
    }
    FreshnessStatus::Missing
}
```

## Call Site Refactoring
- In `src/commands/doctor.rs`:
  - Replace `extract_tool_version(name)` and `evaluate_freshness(...)` with `detect_tool_freshness(ctx, name, info)`.
  - Filter `registry.skills` through `!is_skill_configured(ctx, name)` before printing `skill-suggestion`.
- In `src/commands/tools.rs`:
  - Replace `extract_tool_version(name)` and `evaluate_freshness(...)` with `detect_tool_freshness(ctx, name, info)`.
  - Filter `registry.skills` through `!is_skill_configured(ctx, name)` before printing suggestions table.
