//! Interactive TUI menu for ce-ai when executed without subcommands.

use std::fmt;
use std::io::IsTerminal;
use std::path::PathBuf;

use inquire::{Confirm, Select, Text};

use crate::commands::{doctor, install, models, status, sync, uninstall, upgrade, Context};
use crate::error::CeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainMenuAction {
    Status,
    Install,
    Models,
    Sync,
    Upgrade,
    Doctor,
    Uninstall,
    Exit,
}

impl fmt::Display for MainMenuAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainMenuAction::Status => write!(f, "📊  Status (View installed harnesses & drift)"),
            MainMenuAction::Install => write!(f, "📥  Install (Install CE plugin into OpenCode)"),
            MainMenuAction::Models => write!(f, "🤖  Models (Configure slots & profiles)"),
            MainMenuAction::Sync => write!(f, "🔄  Sync (Reconcile local drift)"),
            MainMenuAction::Upgrade => write!(f, "🚀  Upgrade (Fetch latest CE release)"),
            MainMenuAction::Doctor => write!(f, "🩺  Doctor (Health & diagnostic checks)"),
            MainMenuAction::Uninstall => {
                write!(f, "🗑️   Uninstall (Restore original configuration)")
            }
            MainMenuAction::Exit => write!(f, "❌  Exit"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelsSubAction {
    Set,
    List,
    SaveProfile,
    LoadProfile,
    Back,
}

impl fmt::Display for ModelsSubAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelsSubAction::Set => write!(f, "✏️   Set model assignment for a slot"),
            ModelsSubAction::List => write!(f, "📋  List current model assignments"),
            ModelsSubAction::SaveProfile => write!(f, "💾  Save named model profile"),
            ModelsSubAction::LoadProfile => write!(f, "📂  Load named model profile"),
            ModelsSubAction::Back => write!(f, "⬅️   Back to main menu"),
        }
    }
}

pub fn run_interactive(ctx: &Context) -> Result<(), CeError> {
    if !std::io::stdin().is_terminal() {
        return Err(CeError::Usage(
            "no subcommand provided — run 'ce-ai --help' for usage or run in an interactive terminal for TUI mode".into(),
        ));
    }

    println!("\n✨ Welcome to ce-ai — Compound Engineering Plugin Manager TUI ✨\n");

    loop {
        let options = vec![
            MainMenuAction::Status,
            MainMenuAction::Install,
            MainMenuAction::Models,
            MainMenuAction::Sync,
            MainMenuAction::Upgrade,
            MainMenuAction::Doctor,
            MainMenuAction::Uninstall,
            MainMenuAction::Exit,
        ];

        let selection = match Select::new("Select an action:", options).prompt() {
            Ok(choice) => choice,
            Err(_) => break, // User pressed Esc or Ctrl+C
        };

        println!();
        match selection {
            MainMenuAction::Status => {
                if let Err(err) = status::run(ctx) {
                    eprintln!("error: {err}");
                }
            }
            MainMenuAction::Install => {
                handle_install(ctx);
            }
            MainMenuAction::Models => {
                handle_models(ctx);
            }
            MainMenuAction::Sync => {
                handle_sync(ctx);
            }
            MainMenuAction::Upgrade => {
                handle_upgrade(ctx);
            }
            MainMenuAction::Doctor => {
                if let Err(err) = doctor::run(ctx) {
                    eprintln!("error: {err}");
                }
            }
            MainMenuAction::Uninstall => {
                handle_uninstall(ctx);
            }
            MainMenuAction::Exit => {
                println!("Goodbye!");
                break;
            }
        }
        println!();
    }

    Ok(())
}

fn handle_install(ctx: &Context) {
    let source_choice = Select::new(
        "Select installation source:",
        vec![
            "Latest GitHub release (default)",
            "Local source directory...",
        ],
    )
    .prompt();

    let source = match source_choice {
        Ok("Local source directory...") => match Text::new("Path to CE source tree:").prompt() {
            Ok(path) if !path.trim().is_empty() => Some(PathBuf::from(path.trim())),
            _ => return,
        },
        Ok(_) => None,
        Err(_) => return,
    };

    let dry_run = Confirm::new("Preview changes without modifying disk? (dry-run)")
        .with_default(ctx.dry_run)
        .prompt()
        .unwrap_or(ctx.dry_run);

    let mut install_ctx = ctx.clone();
    install_ctx.dry_run = dry_run;

    let args = install::Args {
        harness: "opencode".into(),
        source,
    };
    if let Err(err) = install::run(&install_ctx, &args) {
        eprintln!("error: {err}");
    }
}

fn handle_models(ctx: &Context) {
    loop {
        let sub_options = vec![
            ModelsSubAction::Set,
            ModelsSubAction::List,
            ModelsSubAction::SaveProfile,
            ModelsSubAction::LoadProfile,
            ModelsSubAction::Back,
        ];
        let selection = match Select::new("Models & Profiles:", sub_options).prompt() {
            Ok(choice) => choice,
            Err(_) => break,
        };

        println!();
        match selection {
            ModelsSubAction::Set => {
                let slot = match Text::new("Agent slot (e.g. sdd-explore, sdd-design):").prompt() {
                    Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
                    _ => continue,
                };
                let model = match Text::new("Model (e.g. opencode-go/kimi-k2.6):").prompt() {
                    Ok(m) if !m.trim().is_empty() => m.trim().to_string(),
                    _ => continue,
                };
                let args = models::Args {
                    command: models::ModelsCommand::Set(models::SetArgs { slot, model }),
                };
                if let Err(err) = models::run(ctx, &args) {
                    eprintln!("error: {err}");
                }
            }
            ModelsSubAction::List => {
                let args = models::Args {
                    command: models::ModelsCommand::List,
                };
                if let Err(err) = models::run(ctx, &args) {
                    eprintln!("error: {err}");
                }
            }
            ModelsSubAction::SaveProfile => {
                let name = match Text::new("Profile name to save:").prompt() {
                    Ok(n) if !n.trim().is_empty() => n.trim().to_string(),
                    _ => continue,
                };
                let args = models::Args {
                    command: models::ModelsCommand::Profile(models::ProfileArgs {
                        command: models::ProfileCommand::Save(models::ProfileNameArgs { name }),
                    }),
                };
                if let Err(err) = models::run(ctx, &args) {
                    eprintln!("error: {err}");
                }
            }
            ModelsSubAction::LoadProfile => {
                let name = match Text::new("Profile name to load:").prompt() {
                    Ok(n) if !n.trim().is_empty() => n.trim().to_string(),
                    _ => continue,
                };
                let args = models::Args {
                    command: models::ModelsCommand::Profile(models::ProfileArgs {
                        command: models::ProfileCommand::Load(models::ProfileNameArgs { name }),
                    }),
                };
                if let Err(err) = models::run(ctx, &args) {
                    eprintln!("error: {err}");
                }
            }
            ModelsSubAction::Back => break,
        }
        println!();
    }
}

fn handle_sync(ctx: &Context) {
    let dry_run = Confirm::new("Preview sync changes without writing? (dry-run)")
        .with_default(ctx.dry_run)
        .prompt()
        .unwrap_or(ctx.dry_run);

    let mut sync_ctx = ctx.clone();
    sync_ctx.dry_run = dry_run;

    if let Err(err) = sync::run(&sync_ctx) {
        eprintln!("error: {err}");
    }
}

fn handle_upgrade(ctx: &Context) {
    let target = Select::new(
        "Select upgrade target:",
        vec!["Latest GitHub release", "Specific release tag..."],
    )
    .prompt();

    let to = match target {
        Ok("Specific release tag...") => {
            match Text::new("Release tag (e.g. compound-engineering-v3.5.0):").prompt() {
                Ok(tag) if !tag.trim().is_empty() => Some(tag.trim().to_string()),
                _ => return,
            }
        }
        Ok(_) => None,
        Err(_) => return,
    };

    let args = upgrade::Args { to, source: None };
    if let Err(err) = upgrade::run(ctx, &args) {
        eprintln!("error: {err}");
    }
}

fn handle_uninstall(ctx: &Context) {
    let confirm = Confirm::new(
        "Are you sure you want to uninstall CE plugin and restore original OpenCode configuration?",
    )
    .with_default(false)
    .prompt()
    .unwrap_or(false);

    if confirm {
        let args = uninstall::Args {
            harness: "opencode".into(),
        };
        if let Err(err) = uninstall::run(ctx, &args) {
            eprintln!("error: {err}");
        }
    } else {
        println!("Uninstall cancelled.");
    }
}
