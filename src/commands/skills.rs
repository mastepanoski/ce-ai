//! CLI subcommand handler: `ce-ai skills`.
//!
//! Subcommands: `list`, `resolve`, `doctor`.

use clap::{Args as ClapArgs, Subcommand};

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::source::registry::SkillRegistry;

#[derive(ClapArgs, Debug, Clone)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// List indexed skills across all active host harnesses.
    List {
        /// Filter catalog by harness kind (e.g. opencode, claude, cursor).
        #[arg(long)]
        harness: Option<String>,

        /// Output catalog in machine-readable JSON format.
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Resolve exact SKILL.md paths for sub-agent prompt injection.
    Resolve {
        /// Target host harness kind (e.g. opencode, claude, pi, cursor).
        #[arg(long, default_value = "opencode")]
        harness: String,

        /// Search query or task keyword (positional).
        #[arg(value_name = "QUERY")]
        query_pos: Option<String>,

        /// Search query or task keyword (flag).
        #[arg(long)]
        query: Option<String>,

        /// Output resolution result in machine-readable JSON format.
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Run diagnostic health check on skill registry integrity (alias to ce-ai doctor probe).
    Doctor,
    /// Put pre-existing `ce-*` skill copies under ce-ai management.
    Adopt {
        /// Harness whose skills root is scanned (name or `all`).
        #[arg(long, default_value = "all")]
        harness: String,

        /// Confirm every adoptable surface without an interactive prompt.
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let registry_path = ctx.config_dir.join("skills-registry.json");
    let registry = SkillRegistry::load(&registry_path)?;

    match &args.action {
        Action::List { harness, json } => {
            let filter_harness = match harness {
                Some(h) => Some(h.parse::<HarnessKind>()?),
                None => None,
            };

            if *json {
                let filtered_skills: Vec<_> = registry
                    .skills
                    .iter()
                    .filter(|s| {
                        if let Some(h) = filter_harness {
                            s.harness_paths.contains_key(h.as_str())
                        } else {
                            true
                        }
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&filtered_skills)?);
            } else {
                println!("📦 Skill Registry Catalog (v{}):", registry.version);
                println!(
                    "{:<20} {:<10} {:<40} SHA256",
                    "NAME", "SCOPE", "DESCRIPTION"
                );
                println!("{}", "-".repeat(80));

                for skill in &registry.skills {
                    if let Some(h) = filter_harness {
                        if !skill.harness_paths.contains_key(h.as_str()) {
                            continue;
                        }
                    }
                    let desc_chars: String = skill.description.chars().take(37).collect();
                    let desc = if skill.description.chars().count() > 37 {
                        format!("{}...", desc_chars)
                    } else {
                        skill.description.clone()
                    };
                    println!(
                        "{:<20} {:<10} {:<40} {}",
                        skill.name,
                        skill.scope,
                        desc,
                        &skill.sha256[..8]
                    );
                }
            }
        }
        Action::Resolve {
            harness,
            query_pos,
            query,
            json,
        } => {
            let effective_query = query_pos
                .as_deref()
                .or(query.as_deref())
                .unwrap_or("")
                .trim();
            if effective_query.is_empty() {
                return Err(CeError::Usage(
                    "missing search query: provide a positional query or --query <QUERY>".into(),
                ));
            }

            let harness_kind = harness.parse::<HarnessKind>()?;
            let (status, skills, markdown) = registry.resolve(harness_kind, effective_query);

            if status == "fallback-fuzzy" {
                eprintln!(
                    "⚠️ Warning: Skill resolution degraded to fallback-fuzzy for query '{}'",
                    effective_query
                );
            }

            if *json {
                let output = serde_json::json!({
                    "resolution_status": status,
                    "query": effective_query,
                    "harness": harness_kind.as_str(),
                    "skills": skills,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print!("{}", markdown);
            }
        }
        Action::Doctor => {
            let findings = crate::source::registry::check_skill_registry_health(ctx)?;
            if findings.is_empty() {
                if !ctx.quiet {
                    println!("✅ Skill registry integrity is healthy.");
                }
            } else {
                println!("⚠️ Skill Registry Diagnostics ({} issues):", findings.len());
                for f in findings {
                    println!("  - {}", f);
                }
                return Err(CeError::Runtime(
                    "skill registry integrity check failed".into(),
                ));
            }
        }
        Action::Adopt { harness, yes } => {
            crate::commands::adopt::run(
                ctx,
                &crate::commands::adopt::Args {
                    harness: harness.clone(),
                    yes: *yes,
                },
            )?;
        }
    }

    Ok(())
}
