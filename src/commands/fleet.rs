//! `ce-ai fleet`: Fleet version governance and synchronization across native AI coding agent harnesses.

use crate::commands::Context;
use crate::error::CeError;
use crate::fleet::{
    all_drivers, generate_fleet_status_report, FleetAction, FleetHarnessStatus, HarnessAlignment,
};
use crate::state::state::State;
use clap::{Args as ClapArgs, Subcommand};

#[derive(ClapArgs, Debug, Clone)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<FleetActionCommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum FleetActionCommand {
    /// Show installed harness versions and alignment with pinned version.
    Status {
        /// Emit status report in JSON format.
        #[arg(long)]
        json: bool,
    },
    /// Pin target Compound Engineering version for the fleet.
    Pin {
        /// Semantic version tag to pin (e.g. v1.76.0 or 1.76.0).
        version: String,
    },
    /// Synchronize all detected/installed harnesses with the pinned fleet version.
    Sync {
        /// Preview planned actions without writing.
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(ctx: &Context, args: Args) -> Result<(), CeError> {
    let action = args
        .action
        .unwrap_or(FleetActionCommand::Status { json: false });
    match action {
        FleetActionCommand::Status { json } => handle_status(ctx, json),
        FleetActionCommand::Pin { version } => handle_pin(ctx, &version),
        FleetActionCommand::Sync { dry_run } => handle_sync(ctx, dry_run),
    }
}

fn handle_status(ctx: &Context, json: bool) -> Result<(), CeError> {
    let report = generate_fleet_status_report(ctx)?;
    if json {
        let serialized = serde_json::to_string_pretty(&report).map_err(CeError::Json)?;
        println!("{serialized}");
        return Ok(());
    }

    println!("Fleet Version Governance:");
    let pinned_display = report
        .pinned_version
        .as_deref()
        .unwrap_or("(none — run 'ce-ai fleet pin <version>')");
    println!("  Pinned Version: {pinned_display}");
    println!(
        "  Fleet Aligned:  {}",
        if report.is_aligned { "yes" } else { "no" }
    );
    println!();
    println!("Harness Statuses:");
    if report.harnesses.is_empty() {
        println!("  (no harnesses detected or installed)");
    } else {
        for h in &report.harnesses {
            print_harness_status(h);
        }
    }

    Ok(())
}

fn print_harness_status(h: &FleetHarnessStatus) {
    let harness_name = h.harness.as_str();
    match &h.alignment {
        HarnessAlignment::Aligned => {
            let ver = h.installed_version.as_deref().unwrap_or("unknown");
            println!("  - {harness_name}: {ver} (aligned)");
        }
        HarnessAlignment::Divergent { installed, pinned } => {
            println!("  - {harness_name}: {installed} (divergent from pinned {pinned})");
        }
        HarnessAlignment::Missing { pinned } => {
            println!("  - {harness_name}: not installed (missing pinned {pinned})");
        }
        HarnessAlignment::Unmanaged { installed } => {
            println!("  - {harness_name}: {installed} (unmanaged)");
        }
    }
}

fn handle_pin(ctx: &Context, raw_version: &str) -> Result<(), CeError> {
    let clean = raw_version.trim();
    if clean.is_empty() {
        return Err(CeError::Usage("version tag cannot be empty".to_string()));
    }
    if clean.contains(' ') || clean.contains('/') || clean.contains('\\') {
        return Err(CeError::Usage(format!("invalid version tag '{clean}'")));
    }

    let normalized = if clean.starts_with('v') || clean.starts_with('V') {
        format!("v{}", &clean[1..])
    } else {
        format!("v{clean}")
    };

    let state_file = ctx.state_path();
    let mut state = if state_file.exists() {
        State::load(&state_file)?
    } else {
        State::new()
    };

    let mut fleet = state.fleet.unwrap_or_default();
    fleet.pinned_version = Some(normalized.clone());
    state.fleet = Some(fleet);

    let pretty = serde_json::to_string_pretty(&state)
        .map_err(|e| CeError::State(format!("failed to serialize state.json: {e}")))?;
    crate::state::write_atomic(&state_file, pretty.as_bytes())?;

    println!("Pinned fleet Compound Engineering version to {normalized}.");
    Ok(())
}

fn handle_sync(ctx: &Context, dry_run: bool) -> Result<(), CeError> {
    let effective_dry_run = dry_run || ctx.dry_run;
    let state_file = ctx.state_path();
    let mut state = if state_file.exists() {
        State::load(&state_file)?
    } else {
        State::new()
    };

    let pinned_version = match state
        .fleet
        .as_ref()
        .and_then(|f| f.pinned_version.as_deref())
    {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            return Err(CeError::Usage(
                "no fleet version is pinned; run 'ce-ai fleet pin <version>' first".to_string(),
            ));
        }
    };

    println!("Synchronizing fleet harnesses with pinned version {pinned_version}...");
    let drivers = all_drivers();
    let mut synced_any = false;

    for driver in &drivers {
        if !driver.is_installed(ctx) && driver.detect_version(ctx)?.is_none() {
            continue;
        }
        synced_any = true;
        let action = driver.plan_sync(ctx, &pinned_version)?;
        let harness_name = driver.harness_kind().as_str();

        match &action {
            FleetAction::UpToDate => {
                println!("  - {harness_name}: already up to date ({pinned_version})");
            }
            FleetAction::RunCommand {
                program,
                args,
                description,
            } => {
                if effective_dry_run {
                    println!(
                        "  [dry-run] {harness_name}: would run '{program} {}' ({description})",
                        args.join(" ")
                    );
                } else {
                    println!(
                        "  - {harness_name}: running '{program} {}' ({description})",
                        args.join(" ")
                    );
                    action.execute(false)?;
                }
            }
            FleetAction::UpdateConfig {
                file_path,
                key,
                value: _,
                description,
            } => {
                if effective_dry_run {
                    println!(
                        "  [dry-run] {harness_name}: would update {} in {} ({description})",
                        key,
                        file_path.display()
                    );
                } else {
                    println!(
                        "  - {harness_name}: updating {} in {} ({description})",
                        key,
                        file_path.display()
                    );
                    action.execute(false)?;
                }
            }
        }
    }

    if !synced_any {
        println!("No installed harnesses were detected to synchronize.");
    }

    if !effective_dry_run {
        let mut fleet = state.fleet.unwrap_or_default();
        fleet.last_sync = Some(chrono::Utc::now().to_rfc3339());
        state.fleet = Some(fleet);

        let pretty = serde_json::to_string_pretty(&state)
            .map_err(|e| CeError::State(format!("failed to serialize state.json: {e}")))?;
        crate::state::write_atomic(&state_file, pretty.as_bytes())?;
        println!("Fleet synchronization complete.");
    } else {
        println!("Dry-run complete: no changes were written.");
    }

    Ok(())
}
