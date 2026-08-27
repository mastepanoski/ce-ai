//! `ce-ai tools`: detection, installation, and management of companion sidecars
//! (Engram, CodeGraph, Context7, RTK), version freshness, and skill suggestions.

use crate::commands::Context;
use crate::error::CeError;
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

fn mcp_spec_for_tool(tool: &str) -> Option<(&'static str, &'static [&'static str])> {
    match tool {
        "context7" => Some(("npx", &["-y", "@upstash/context7-mcp@latest"])),
        "engram" => Some(("engram", &["serve"])),
        "rtk" => Some(("rtk", &["mcp"])),
        "codegraph" => Some(("codegraph", &["mcp"])),
        _ => None,
    }
}

fn install_tool(ctx: &Context, tool: &str) -> Result<(), CeError> {
    let tool_lower = tool.to_lowercase();

    let (cmd, args) = mcp_spec_for_tool(tool_lower.as_str()).ok_or_else(|| {
        CeError::Usage(format!(
            "unknown companion tool '{tool}'. Supported tools: engram, codegraph, context7, rtk"
        ))
    })?;

    let server_def = serde_json::json!({ "command": cmd, "args": args });

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
        for entry in &state.installed_harnesses {
            let Some(name) = entry["name"].as_str() else {
                continue;
            };
            if name == "pi" {
                if !ctx.quiet {
                    println!("info: pi harness does not support native MCP servers by design");
                }
                continue;
            }
            if name == "opencode" || name == "custom" || name == "deepseek" {
                continue;
            }
            let Ok(kind) = name.parse::<crate::harness::HarnessKind>() else {
                continue;
            };
            kind.register_tool_mcp(&home_dir, &tool_lower, cmd, args)?;
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
            workspace_root: None,
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
            workspace_root: None,
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
            workspace_root: None,
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
            workspace_root: None,
            dry_run: true,
            verbose: false,
            quiet: true,
        };

        let result = install_tool(&ctx, "engram");
        assert!(result.is_ok());
        assert!(!opencode_dir.join("opencode.json").exists());
    }
}
