//! `ce-ai tools`: detection, installation, and management of companion sidecars
//! (Engram, CodeGraph, Context7, RTK), version freshness, and skill suggestions.

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessAdapter;
use crate::source::tools_registry::{
    evaluate_freshness, extract_tool_version, FreshnessStatus, ToolsRegistryCache,
};

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand)]
pub enum Action {
    /// Check installation, version freshness, and health status of companion tools.
    Status,
    /// Install or provision a specific companion tool (engram, codegraph, context7, rtk).
    Install {
        /// Name of tool (engram, codegraph, context7, rtk).
        tool: String,
    },
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.action {
        Action::Status => status(ctx),
        Action::Install { tool } => install_tool(ctx, tool),
    }
}

pub fn status(ctx: &Context) -> Result<(), CeError> {
    let registry = ToolsRegistryCache::load_or_default(ctx);

    println!("== [Companion Tools, Memory Sidecars & Token Reducers Status] ==");
    for (name, info) in &registry.tools {
        let installed = extract_tool_version(name);
        let freshness = evaluate_freshness(installed.as_deref(), &info.latest_version);

        let icon = match &freshness {
            FreshnessStatus::Ok { .. } => "✅",
            FreshnessStatus::Outdated { .. } => "⚠️",
            FreshnessStatus::Missing => "❌",
            FreshnessStatus::Offline { .. } => "🌐",
        };

        let hint = match &freshness {
            FreshnessStatus::Ok { version } => format!("v{version} (ok)"),
            FreshnessStatus::Outdated { current, expected } => {
                format!(
                    "v{current} (outdated -> v{expected} available; run '{}')",
                    info.install_cmd
                )
            }
            FreshnessStatus::Missing => format!("not found (suggested: '{}')", info.install_cmd),
            FreshnessStatus::Offline { current } => format!("v{current} (offline)"),
        };

        println!(
            "  {icon} {:<12} [{:<16}] ({}) : {}",
            name, info.category, info.label, hint
        );
    }

    println!("\n== [Skill Registry Suggestions] ==");
    for (name, skill) in &registry.skills {
        println!(
            "  ⚠️ {:<20} {} (suggested: '{}')",
            name, skill.description, skill.resolve_cmd
        );
    }

    println!("\n== [Orchestrator Readiness] ==");
    let ce_version = env!("CARGO_PKG_VERSION");
    println!("  ✓ ce-ai CLI            v{} (ok)", ce_version);

    Ok(())
}

fn install_tool(ctx: &Context, tool: &str) -> Result<(), CeError> {
    let tool_lower = tool.to_lowercase();

    let server_def = match tool_lower.as_str() {
        "context7" => serde_json::json!({
            "command": "npx",
            "args": ["-y", "@upstash/context7-mcp@latest"]
        }),
        "engram" => serde_json::json!({
            "command": "engram",
            "args": ["serve"]
        }),
        "rtk" => serde_json::json!({
            "command": "rtk",
            "args": ["mcp"]
        }),
        "codegraph" => serde_json::json!({
            "command": "codegraph",
            "args": ["mcp"]
        }),
        _ => {
            return Err(CeError::Usage(format!(
                "unknown companion tool '{tool}'. Supported tools: engram, codegraph, context7, rtk"
            )))
        }
    };

    println!("tools: provisioning companion tool '{tool_lower}'...");

    if ctx.dry_run {
        println!(
            "tools: [dry-run] would merge '{tool_lower}' MCP server definition into opencode.json"
        );
        return Ok(());
    }

    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    crate::opencode::config::register_mcp_server(&opencode_json, &tool_lower, server_def)?;

    let home_dir = crate::harness::home_dir_from_ctx(ctx);
    let state_path = ctx.config_dir.join("state.json");
    if let Ok(state) = crate::state::state::State::load(&state_path) {
        let cursor_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("cursor"));
        if cursor_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let cursor_mcp = home_dir.join(".cursor").join("mcp.json");
            crate::harness::cursor::register_cursor_mcp_server(
                &cursor_mcp,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let claude_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("claude"));
        if claude_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let claude_adapter = crate::harness::claude::ClaudeAdapter;
            let claude_config = claude_adapter.default_config_path(&home_dir);
            crate::harness::claude::register_claude_mcp_server(
                &claude_config,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let codex_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("codex"));
        if codex_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let codex_adapter = crate::harness::codex::CodexAdapter;
            let codex_config = codex_adapter.default_config_path(&home_dir);
            crate::harness::codex::register_codex_mcp_server(
                &codex_config,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let copilot_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("copilot"));
        if copilot_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let copilot_adapter = crate::harness::copilot::CopilotAdapter;
            let copilot_config = copilot_adapter.default_config_path(&home_dir);
            crate::harness::copilot::register_copilot_mcp_server(
                &copilot_config,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let kimi_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("kimi"));
        if kimi_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let kimi_adapter = crate::harness::kimi::KimiAdapter;
            let kimi_config = kimi_adapter.default_config_path(&home_dir);
            crate::harness::kimi::register_kimi_mcp_server(
                &kimi_config,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let grok_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("grok"));
        if grok_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let grok_adapter = crate::harness::grok::GrokAdapter;
            let grok_config = grok_adapter.default_config_path(&home_dir);
            crate::harness::grok::register_grok_mcp_server(
                &grok_config,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let agy_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("agy"));
        if agy_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let agy_adapter = crate::harness::agy::AgyAdapter;
            let agy_config = agy_adapter.default_config_path(&home_dir);
            crate::harness::agy::register_agy_mcp_server(
                &agy_config,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let fx_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("fx"));
        if fx_installed {
            let (cmd, args_vec) = match tool_lower.as_str() {
                "context7" => ("npx", vec!["-y", "@upstash/context7-mcp@latest"]),
                "engram" => ("engram", vec!["serve"]),
                "rtk" => ("rtk", vec!["mcp"]),
                "codegraph" => ("codegraph", vec!["mcp"]),
                _ => ("", vec![]),
            };
            let empty_env = std::collections::BTreeMap::new();
            let fx_adapter = crate::harness::fx::FxAdapter;
            let fx_config = fx_adapter.default_config_path(&home_dir);
            crate::harness::fx::register_fx_mcp_server(
                &fx_config,
                &tool_lower,
                cmd,
                &args_vec,
                &empty_env,
            )?;
        }

        let pi_installed = state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some("pi"));
        if pi_installed && !ctx.quiet {
            println!("info: pi harness does not support native MCP servers by design");
        }
    }

    let probe_version = extract_tool_version(&tool_lower);
    let binary_name = match tool_lower.as_str() {
        "context7" => "npx",
        _ => &tool_lower,
    };
    let binary_on_path = is_in_path(binary_name);

    let status_detail = if let Some(version) = &probe_version {
        format!("v{version} (ok)")
    } else if binary_on_path {
        "registered (binary active on PATH)".into()
    } else {
        format!("registered (note: '{binary_name}' binary not found on PATH; install binary to enable execution)")
    };

    println!("tools: '{tool_lower}' MCP server registration completed ({status_detail}).");

    Ok(())
}

fn is_in_path(name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.join(name).is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tools_status_runs_without_panic() {
        let tmp = TempDir::new().unwrap();
        let ctx = Context {
            config_dir: tmp.path().to_path_buf(),
            opencode_config_dir: tmp.path().to_path_buf(),
            dry_run: false,
            verbose: false,
            quiet: true,
        };
        assert!(status(&ctx).is_ok());
    }

    #[test]
    fn test_tools_install_registers_mcp_server_atomically_without_clobbering() {
        let tmp = TempDir::new().unwrap();
        let opencode_dir = tmp.path().join("opencode");
        std::fs::create_dir_all(&opencode_dir).unwrap();
        let config_file = opencode_dir.join("opencode.json");

        // Write pre-existing user config
        std::fs::write(
            &config_file,
            serde_json::to_vec_pretty(&serde_json::json!({
                "mcpServers": {
                    "custom-user-mcp": { "command": "my-mcp" }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = Context {
            config_dir: tmp.path().to_path_buf(),
            opencode_config_dir: opencode_dir.clone(),
            dry_run: false,
            verbose: false,
            quiet: true,
        };

        let result = install_tool(&ctx, "context7");
        assert!(result.is_ok());

        // Verify config was updated atomically preserving user mcp
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_file).unwrap()).unwrap();
        let mcp = val.get("mcpServers").unwrap().as_object().unwrap();
        assert!(mcp.contains_key("custom-user-mcp"));
        assert!(mcp.contains_key("context7"));
    }

    #[test]
    fn test_tools_install_unknown_tool_fails_usage() {
        let tmp = TempDir::new().unwrap();
        let ctx = Context {
            config_dir: tmp.path().to_path_buf(),
            opencode_config_dir: tmp.path().to_path_buf(),
            dry_run: false,
            verbose: false,
            quiet: true,
        };

        let err = install_tool(&ctx, "invalid-tool").unwrap_err();
        assert!(matches!(err, CeError::Usage(_)));
    }

    #[test]
    fn test_tools_install_dry_run_makes_no_changes() {
        let tmp = TempDir::new().unwrap();
        let opencode_dir = tmp.path().join("opencode");
        let ctx = Context {
            config_dir: tmp.path().to_path_buf(),
            opencode_config_dir: opencode_dir.clone(),
            dry_run: true,
            verbose: false,
            quiet: true,
        };

        let result = install_tool(&ctx, "engram");
        assert!(result.is_ok());
        assert!(!opencode_dir.join("opencode.json").exists());
    }
}
