//! Backup listing and point-in-time recovery subcommand (`ce-ai backups`).

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::state::backups::{list_backups, restore_backup_by_id, restore_latest};
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct BackupsArgs {
    #[command(subcommand)]
    pub command: BackupsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum BackupsSubcommand {
    /// List historical backup snapshots.
    List {
        /// Filter backups by harness target (e.g. opencode, claude, pi, cursor).
        #[arg(short = 't', long)]
        harness: Option<String>,
    },
    /// Restore a specific historical backup snapshot.
    Restore {
        /// Timestamp or backup ID to restore (or 'latest').
        target_id: Option<String>,
        /// Target harness override (e.g. opencode, claude).
        #[arg(short = 't', long)]
        harness: Option<String>,
    },
}

pub fn run(ctx: &Context, args: &BackupsArgs) -> Result<(), CeError> {
    let backups_dir = ctx.config_dir.join("backups");
    match &args.command {
        BackupsSubcommand::List { harness } => {
            let filter = harness.as_deref();
            let entries = list_backups(&backups_dir, filter)?;

            if entries.is_empty() {
                if !ctx.quiet {
                    println!("No backups found under {}", backups_dir.display());
                }
                return Ok(());
            }

            if !ctx.quiet {
                println!(
                    "{:<26} {:<12} {:<24} {:<10}",
                    "BACKUP ID", "HARNESS", "TIMESTAMP", "SIZE"
                );
                println!("{}", "-".repeat(76));
                for entry in &entries {
                    println!(
                        "{:<26} {:<12} {:<24} {} bytes",
                        entry.id, entry.harness, entry.timestamp_rfc3339, entry.size_bytes
                    );
                }
            }
            Ok(())
        }
        BackupsSubcommand::Restore { target_id, harness } => {
            let target_harness = harness
                .as_deref()
                .unwrap_or("opencode")
                .parse::<HarnessKind>()?;
            let home_dir = crate::harness::home_dir_from_ctx(ctx);
            let config_dir = if target_harness == HarnessKind::Opencode {
                ctx.opencode_config_dir.clone()
            } else {
                target_harness.harness_dir(&home_dir)
            };
            let target_path = target_harness.config_path(&config_dir);

            if ctx.dry_run {
                if !ctx.quiet {
                    println!("dry-run: would restore backup to {}", target_path.display());
                }
                return Ok(());
            }

            let restored = match target_id.as_deref() {
                Some("latest") | None => {
                    restore_latest(&backups_dir, &target_path)?;
                    if !ctx.quiet {
                        println!(
                            "Successfully restored latest backup for harness '{}' to {}",
                            target_harness.as_str(),
                            target_path.display()
                        );
                    }
                    return Ok(());
                }
                Some(id) => restore_backup_by_id(&backups_dir, id, &target_path)?,
            };

            if !ctx.quiet {
                println!(
                    "Successfully restored backup '{}' for harness '{}' to {}",
                    restored.id,
                    restored.harness,
                    target_path.display()
                );
            }
            Ok(())
        }
    }
}
