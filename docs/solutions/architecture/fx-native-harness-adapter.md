---
title: "fx Native Harness Adapter Architecture & Integration"
category: "architecture"
tags: ["fx", "harness-adapter", "mcp-json", "skills", "openspec", "semver"]
date: "2026-08-23"
problem_type: "feature_implementation"
---

# fx Native Harness Adapter Architecture & Integration

## Problem Statement
Vercel Labs' `fx` coding agent (`~/.fx/`, `$FX_HOME`) requires a native harness adapter to manage its MCP configuration (`~/.fx/mcp.json`) and skills (`~/.fx/skills/`). The `fx` MCP schema uses a root key `mcp`, `"type": "local"`, array-form `command` (e.g. `["codegraph", "mcp"]`), and an `environment` map.

## Solution Architecture

### 1. Harness Path Resolution
- **Home Directory**: Defaults to `~/.fx/`, overridden by environment variable `$FX_HOME`.
- **Config Path**: `~/.fx/mcp.json` (or `$FX_HOME/mcp.json`).
- **Skills Directory**: `~/.fx/skills/`.
- **Project Rules**: `AGENTS.md` (root) and `.fx/AGENTS.md` (derived stub).

### 2. MCP JSON Schema & Serde Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FxMcpConfig {
    #[serde(default)]
    pub mcp: BTreeMap<String, FxMcpServer>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FxMcpServer {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub environment: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

### 3. Key Findings & Tradeoffs
- `FxAdapter::default_config_path` handles base paths, `.fx` directories, and direct `mcp.json` file paths cleanly.
- `register_fx_mcp_server` and `unregister_fx_mcp_server` perform atomic JSON mutations using `crate::state::write_atomic`, preserving unmanaged user settings and custom MCP servers.
- `install` copies managed skills into `~/.fx/skills/` without touching `~/.config/opencode`.
- `uninstall` removes `codegraph` and `engram` MCP entries from `~/.fx/mcp.json`, removes managed skills, and deletes `mcp.json` if empty of user settings.
