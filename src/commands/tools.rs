//! `ce-ai tools`: detection, installation, and management of companion sidecars
//! (Engram, CodeGraph, Context7, RTK), version freshness, and skill suggestions.

use crate::commands::Context;
use crate::error::CeError;
use crate::source::tools_registry::{
    detect_tool_freshness, extract_tool_version, is_skill_configured, FreshnessStatus,
    ToolsRegistryCache,
};

use std::path::{Path, PathBuf};

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
    /// Initialize workspace index or configuration for a companion tool (e.g. codegraph).
    Init {
        /// Name of tool (codegraph).
        tool: String,
        /// Target project path (defaults to current directory).
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.action {
        Action::Status => status(ctx),
        Action::Install { tool } => install_tool(ctx, tool),
        Action::Init { tool, path } => init_tool(ctx, tool, path.as_deref()),
    }
}

pub fn status(ctx: &Context) -> Result<(), CeError> {
    let registry = ToolsRegistryCache::load_or_default(ctx);

    println!("== [Companion Tools, Memory Sidecars & Token Reducers Status] ==");
    for (name, info) in &registry.tools {
        let freshness = detect_tool_freshness(ctx, name, info);

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

    let unconfigured_skills: Vec<_> = registry
        .skills
        .iter()
        .filter(|(name, _)| !is_skill_configured(ctx, name))
        .collect();
    if !unconfigured_skills.is_empty() {
        println!("\n== [Skill Registry Suggestions] ==");
        for (name, skill) in unconfigured_skills {
            println!(
                "  ⚠️ {:<20} {} (suggested: '{}')",
                name, skill.description, skill.resolve_cmd
            );
        }
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
            if name == "custom" {
                if let Some(cfg) = entry
                    .get("custom")
                    .and_then(crate::harness::custom::CustomHarnessConfig::from_state_json)
                {
                    if let Some(mcp) = &cfg.mcp_file {
                        let env = std::collections::BTreeMap::new();
                        let _ = crate::harness::custom::register_custom_mcp_server(
                            mcp,
                            &tool_lower,
                            cmd,
                            args,
                            &env,
                        );
                    }
                }
                continue;
            }
            if name == "opencode" || name == "deepseek" {
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

pub fn init_tool(ctx: &Context, tool: &str, path: Option<&Path>) -> Result<(), CeError> {
    let tool_lower = tool.to_lowercase();
    match tool_lower.as_str() {
        "codegraph" => init_codegraph(ctx, path),
        _ => Err(CeError::Usage(format!(
            "tool '{tool}' does not support init. Supported tools for init: codegraph"
        ))),
    }
}

fn init_codegraph(ctx: &Context, path: Option<&Path>) -> Result<(), CeError> {
    let target_path = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(CeError::Io)?,
    };

    let codegraph_dir = target_path.join(".codegraph");
    if codegraph_dir.exists() {
        if !ctx.quiet {
            println!(
                "tools: codegraph index (.codegraph/) is already initialized at '{}'",
                target_path.display()
            );
        }
        return Ok(());
    }

    if ctx.dry_run {
        if !ctx.quiet {
            println!(
                "tools: [dry-run] would run 'codegraph init' in '{}'",
                target_path.display()
            );
        }
        return Ok(());
    }

    let check = std::process::Command::new("codegraph")
        .arg("--version")
        .output();

    match check {
        Ok(_) => {
            if !ctx.quiet {
                println!(
                    "tools: initializing CodeGraph index in '{}'...",
                    target_path.display()
                );
            }
            let status = std::process::Command::new("codegraph")
                .arg("init")
                .arg(&target_path)
                .status()
                .map_err(|e| {
                    CeError::Runtime(format!("failed to execute 'codegraph init': {e}"))
                })?;

            if status.success() {
                if !ctx.quiet {
                    println!(
                        "✓ Initialized CodeGraph index (.codegraph/) at '{}'",
                        target_path.display()
                    );
                }
                Ok(())
            } else {
                Err(CeError::Runtime(format!(
                    "'codegraph init' exited with status {status}"
                )))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(CeError::Usage(
            "codegraph binary not found on PATH. Install it first (e.g. npm install -g @colbymchenry/codegraph or check https://github.com/colbymchenry/codegraph)".into(),
        )),
        Err(e) => Err(CeError::Runtime(format!(
            "failed to probe 'codegraph --version': {e}"
        ))),
    }
}

fn is_in_path(name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.join(name).is_file() || dir.join(format!("{name}.exe")).is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
#[path = "tests/tools.rs"]
mod tests;
