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
        /// Filter by repository constellation system identifier.
        #[arg(long)]
        system: Option<String>,
    },
    /// Pin target Compound Engineering version for the fleet.
    Pin {
        /// Semantic version tag to pin (e.g. v1.76.0 or 1.76.0), or 'latest' to resolve the newest release.
        version: String,
    },
    /// Synchronize all detected/installed harnesses with the pinned fleet version.
    Sync {
        /// Preview planned actions without writing.
        #[arg(long)]
        dry_run: bool,
        /// Filter by repository constellation system identifier.
        #[arg(long)]
        system: Option<String>,
    },
    /// Update fleet harnesses to the latest (or specified) version in a single step (pin + sync).
    Update(UpdateArgs),
    /// Configure an automated periodic schedule (launchd/systemd/cron) to run 'ce-ai update'.
    Schedule(crate::fleet::schedule::ScheduleArgs),
}

/// Arguments for `ce-ai update` and `ce-ai fleet update`.
#[derive(ClapArgs, Debug, Clone, PartialEq, Eq)]
pub struct UpdateArgs {
    /// Semantic version tag to pin (e.g. v3.31.0 or 3.31.0), defaults to 'latest'.
    pub version: Option<String>,
    /// Target version tag (alias for positional version argument).
    #[arg(long)]
    pub to: Option<String>,
    /// Preview planned actions without writing.
    #[arg(long)]
    pub dry_run: bool,
    /// Also update the ce-ai CLI binary itself to the latest release.
    #[arg(long)]
    pub bin: bool,
    /// Filter by repository constellation system identifier.
    #[arg(long)]
    pub system: Option<String>,
}

pub fn run(ctx: &Context, args: Args) -> Result<(), CeError> {
    let action = args.action.unwrap_or(FleetActionCommand::Status {
        json: false,
        system: None,
    });
    match action {
        FleetActionCommand::Status { json, system } => handle_status(ctx, json, system.as_deref()),
        FleetActionCommand::Pin { version } => handle_pin(ctx, &version),
        FleetActionCommand::Sync { dry_run, system } => {
            handle_sync_with_filter(ctx, dry_run, system.as_deref())
        }
        FleetActionCommand::Update(args) => handle_update(ctx, &args),
        FleetActionCommand::Schedule(args) => crate::fleet::schedule::handle_schedule(ctx, &args),
    }
}

pub fn handle_status(
    ctx: &Context,
    json: bool,
    system_filter: Option<&str>,
) -> Result<(), CeError> {
    let repo_root = ctx.repo_root();
    let current_manifest = crate::compat::arch::load_system_manifest(&repo_root)?;

    if let Some(target_system) = system_filter {
        let manifest = match current_manifest.as_ref() {
            Some(m) if m.system == target_system => m,
            Some(m) => {
                return Err(CeError::Usage(format!(
                    "current repository system '{}' does not match requested constellation '{target_system}'",
                    m.system
                )));
            }
            None => {
                return Err(CeError::Usage(format!(
                    "no system.yaml found in current repository; cannot filter by constellation '{target_system}'"
                )));
            }
        };

        let mut dep_statuses = Vec::new();
        for dep in &manifest.dependencies {
            let status = crate::compat::arch::resolve_dependency_status(&repo_root, dep);
            dep_statuses.push((dep, status));
        }

        if json {
            let report = generate_fleet_status_report(ctx)?;
            let payload = serde_json::json!({
                "system": manifest.system,
                "component": manifest.component,
                "role": manifest.role,
                "dependencies": dep_statuses.iter().map(|(dep, st)| {
                    serde_json::json!({
                        "component": dep.component,
                        "role": dep.role,
                        "path": dep.path,
                        "remote_repo": dep.remote_repo,
                        "contract": dep.contract,
                        "status": st.display_label(),
                    })
                }).collect::<Vec<_>>(),
                "fleet_governance": report,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).map_err(CeError::Json)?
            );
            return Ok(());
        }

        println!("Repository Constellation Governance:");
        println!("  System:    {}", manifest.system);
        let role_label = manifest.role.as_deref().unwrap_or("unspecified");
        println!("  Component: {} ({role_label})", manifest.component);
        println!();
        println!("Dependencies (Local-First, Remote-Graceful):");
        if dep_statuses.is_empty() {
            println!("  (no external constellation dependencies declared)");
        } else {
            for (dep, status) in &dep_statuses {
                let dep_role = dep.role.as_deref().unwrap_or("component");
                println!(
                    "  - {} ({}): {}",
                    dep.component,
                    dep_role,
                    status.display_label()
                );
            }
        }
        println!();
    }

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

pub fn resolve_target_version(raw_version: &str) -> Result<String, CeError> {
    let clean = raw_version.trim();
    if clean.is_empty() {
        return Err(CeError::Usage("version tag cannot be empty".to_string()));
    }
    if clean.contains(' ') || clean.contains('/') || clean.contains('\\') {
        return Err(CeError::Usage(format!("invalid version tag '{clean}'")));
    }

    let normalized = if clean.eq_ignore_ascii_case("latest") {
        let client = reqwest::blocking::Client::new();
        let token = crate::source::release::resolve_github_token();
        let tag = crate::source::release::resolve_latest_release(&client, token.as_deref())?
            .ok_or_else(|| {
                CeError::Usage(
                    "no 'compound-engineering-v*' release found on GitHub — specify an explicit version tag (e.g. 'v3.30.4')".to_string(),
                )
            })?;
        let ver = crate::compat::release::CeRelease::parse_tag(&tag)
            .map(|rel| format!("v{}", rel.version))
            .unwrap_or_else(|| tag.clone());
        println!("Resolved latest Compound Engineering release: {ver}");
        ver
    } else if let Some(rel) = crate::compat::release::CeRelease::parse_tag(clean) {
        format!("v{}", rel.version)
    } else if clean.starts_with('v') || clean.starts_with('V') {
        format!("v{}", &clean[1..])
    } else {
        format!("v{clean}")
    };

    Ok(normalized)
}

pub fn handle_pin(ctx: &Context, raw_version: &str) -> Result<(), CeError> {
    let normalized = resolve_target_version(raw_version)?;

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

pub fn validate_system_filter(ctx: &Context, system_filter: Option<&str>) -> Result<(), CeError> {
    if let Some(target_system) = system_filter {
        let repo_root = ctx.repo_root();
        let current_manifest = crate::compat::arch::load_system_manifest(&repo_root)?;
        match current_manifest {
            Some(ref m) if m.system == target_system => {
                println!(
                    "Constellation filter active: verified repository belongs to constellation '{}' (component: {}).",
                    m.system, m.component
                );
            }
            Some(ref m) => {
                return Err(CeError::Usage(format!(
                    "current repository system '{}' does not match requested constellation '{target_system}'",
                    m.system
                )));
            }
            None => {
                return Err(CeError::Usage(format!(
                    "no system.yaml found in current repository; cannot filter sync by constellation '{target_system}'"
                )));
            }
        }
    }
    Ok(())
}

pub fn handle_sync(ctx: &Context, dry_run: bool) -> Result<(), CeError> {
    handle_sync_with_filter(ctx, dry_run, None)
}

pub fn handle_sync_with_filter(
    ctx: &Context,
    dry_run: bool,
    system_filter: Option<&str>,
) -> Result<(), CeError> {
    validate_system_filter(ctx, system_filter)?;

    let state_file = ctx.state_path();
    let state = if state_file.exists() {
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

    let effective_dry_run = dry_run || ctx.dry_run;
    sync_harnesses_with_version(ctx, &pinned_version, effective_dry_run, None)
}

pub fn sync_harnesses_with_version(
    ctx: &Context,
    pinned_version: &str,
    effective_dry_run: bool,
    system_filter: Option<&str>,
) -> Result<(), CeError> {
    validate_system_filter(ctx, system_filter)?;

    println!("Synchronizing fleet harnesses with pinned version {pinned_version}...");
    let drivers = all_drivers();
    let mut synced_any = false;

    for driver in &drivers {
        if !driver.is_installed(ctx) && driver.detect_version(ctx)?.is_none() {
            continue;
        }
        synced_any = true;
        let action = driver.plan_sync(ctx, pinned_version)?;
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
        let state_file = ctx.state_path();
        let mut state = if state_file.exists() {
            State::load(&state_file)?
        } else {
            State::new()
        };

        let mut fleet = state.fleet.unwrap_or_default();
        fleet.last_sync = Some(chrono::Utc::now().to_rfc3339());
        state.fleet = Some(fleet);

        let now = chrono::Utc::now().to_rfc3339();
        for h in &mut state.installed_harnesses {
            if let Some(name) = h.get("name").and_then(|n| n.as_str()) {
                if drivers.iter().any(|d| d.harness_kind().as_str() == name) {
                    if let Some(obj) = h.as_object_mut() {
                        obj.insert("version".to_string(), serde_json::json!(pinned_version));
                        obj.insert("last_synced_at".to_string(), serde_json::json!(now));
                    }
                }
            }
        }

        let pretty = serde_json::to_string_pretty(&state)
            .map_err(|e| CeError::State(format!("failed to serialize state.json: {e}")))?;
        crate::state::write_atomic(&state_file, pretty.as_bytes())?;
        println!("Fleet synchronization complete.");
    } else {
        println!("Dry-run complete: no changes were written.");
    }

    Ok(())
}

pub fn handle_update(ctx: &Context, args: &UpdateArgs) -> Result<(), CeError> {
    let effective_dry_run = args.dry_run || ctx.dry_run;

    if args.bin {
        if effective_dry_run {
            println!("[dry-run] would update ce-ai CLI binary to latest release");
        } else {
            crate::commands::self_update::run(
                ctx,
                &crate::commands::self_update::Args {
                    check: false,
                    to: None,
                    force: false,
                },
            )?;
            println!();
        }
    }

    let raw_target = args
        .to
        .as_deref()
        .or(args.version.as_deref())
        .unwrap_or("latest");

    let normalized = resolve_target_version(raw_target)?;

    if effective_dry_run {
        println!("[dry-run] would pin fleet Compound Engineering version to {normalized}.");
    } else {
        handle_pin(ctx, &normalized)?;
    }

    sync_harnesses_with_version(ctx, &normalized, effective_dry_run, args.system.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_handle_pin_explicit_version() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        handle_pin(&ctx, "3.30.4").unwrap();

        let state = State::load(&ctx.state_path()).unwrap();
        assert_eq!(
            state.fleet.unwrap().pinned_version,
            Some("v3.30.4".to_string())
        );

        handle_pin(&ctx, "compound-engineering-v3.31.0").unwrap();
        let state2 = State::load(&ctx.state_path()).unwrap();
        assert_eq!(
            state2.fleet.unwrap().pinned_version,
            Some("v3.31.0".to_string())
        );
    }

    #[test]
    fn test_handle_pin_empty_or_invalid() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let err1 = handle_pin(&ctx, "  ").unwrap_err();
        assert_eq!(err1.exit_code(), 2);

        let err2 = handle_pin(&ctx, "invalid version with spaces").unwrap_err();
        assert_eq!(err2.exit_code(), 2);
    }

    #[test]
    fn test_handle_pin_latest_when_online() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        // When network is reachable, "latest" resolves a valid SemVer tag
        if let Ok(()) = handle_pin(&ctx, "latest") {
            let state = State::load(&ctx.state_path()).unwrap();
            let pinned = state.fleet.unwrap().pinned_version.unwrap();
            assert!(pinned.starts_with('v'));
            assert!(pinned.len() > 2);
        }
    }

    #[test]
    fn test_handle_sync_updates_installed_harnesses_and_pi_config() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        std::fs::create_dir_all(&ctx.config_dir).unwrap();
        let pi_dir = ctx.home_dir().join(".pi").join("agent");
        std::fs::create_dir_all(&pi_dir).unwrap();

        let mut state = State::new();
        state.fleet = Some(crate::state::state::FleetState {
            pinned_version: Some("v3.30.5".to_string()),
            last_sync: None,
        });
        state.installed_harnesses.push(serde_json::json!({
            "name": "pi",
            "scope": "global",
            "version": "compound-engineering-v3.30.4",
            "installed_at": "2026-09-08T00:50:40+00:00"
        }));
        state.save(&ctx.state_path()).unwrap();

        handle_sync(&ctx, false).unwrap();

        // Verify state.json was updated
        let reloaded = State::load(&ctx.state_path()).unwrap();
        assert_eq!(
            reloaded.installed_harnesses[0]["version"],
            serde_json::json!("v3.30.5")
        );
        assert!(reloaded.installed_harnesses[0]["last_synced_at"].is_string());

        // Verify pi settings.json was written
        let settings_path = pi_dir.join("settings.json");
        assert!(settings_path.exists());
        let settings_val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert_eq!(
            settings_val["compound_engineering_version"],
            serde_json::json!("v3.30.5")
        );
    }

    #[test]
    fn test_fleet_status_with_system_filter_matching_and_mismatch() {
        let temp = tempdir().unwrap();
        let ws = temp.path().join("workspace");
        let arch_dir = ws.join("docs").join("arch");
        std::fs::create_dir_all(&arch_dir).unwrap();

        let system_yaml = r#"
system: payments-platform
component: billing-service
role: service
dependencies:
  - component: auth-service
    contract: auth.json
"#;
        std::fs::write(arch_dir.join("system.yaml"), system_yaml).unwrap();

        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: Some(ws.clone()),
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        // 1. Matching system filter succeeds
        assert!(handle_status(&ctx, false, Some("payments-platform")).is_ok());
        // JSON mode succeeds
        assert!(handle_status(&ctx, true, Some("payments-platform")).is_ok());

        // 2. Mismatching system filter returns Usage error
        let err = handle_status(&ctx, false, Some("logistics-platform")).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(err
            .to_string()
            .contains("does not match requested constellation"));
    }

    #[test]
    fn test_fleet_sync_with_system_filter_validation() {
        let temp = tempdir().unwrap();
        let ws = temp.path().join("workspace");
        let arch_dir = ws.join("docs").join("arch");
        std::fs::create_dir_all(&arch_dir).unwrap();

        let system_yaml = r#"
system: core-platform
component: gateway
"#;
        std::fs::write(arch_dir.join("system.yaml"), system_yaml).unwrap();

        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: Some(ws),
            dry_run: true,
            verbose: false,
            quiet: false,
        };

        // Mismatched constellation fails before checking pinned version
        let err = handle_sync_with_filter(&ctx, true, Some("unrelated-system")).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(err
            .to_string()
            .contains("does not match requested constellation"));
    }

    #[test]
    fn test_resolve_target_version() {
        assert_eq!(resolve_target_version("3.30.4").unwrap(), "v3.30.4");
        assert_eq!(resolve_target_version("v3.31.0").unwrap(), "v3.31.0");
        assert_eq!(
            resolve_target_version("compound-engineering-v3.32.0").unwrap(),
            "v3.32.0"
        );

        let err_empty = resolve_target_version("   ").unwrap_err();
        assert_eq!(err_empty.exit_code(), 2);

        let err_invalid = resolve_target_version("invalid/version").unwrap_err();
        assert_eq!(err_invalid.exit_code(), 2);
    }

    #[test]
    fn test_handle_update_explicit_version() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        std::fs::create_dir_all(&ctx.config_dir).unwrap();
        let pi_dir = ctx.home_dir().join(".pi").join("agent");
        std::fs::create_dir_all(&pi_dir).unwrap();

        let mut state = State::new();
        state.installed_harnesses.push(serde_json::json!({
            "name": "pi",
            "scope": "global",
            "version": "compound-engineering-v3.30.4",
            "installed_at": "2026-09-08T00:50:40+00:00"
        }));
        state.save(&ctx.state_path()).unwrap();

        let update_args = UpdateArgs {
            version: Some("3.31.5".to_string()),
            to: None,
            dry_run: false,
            bin: false,
            system: None,
        };

        handle_update(&ctx, &update_args).unwrap();

        let reloaded = State::load(&ctx.state_path()).unwrap();
        assert_eq!(
            reloaded.fleet.unwrap().pinned_version,
            Some("v3.31.5".to_string())
        );
        assert_eq!(
            reloaded.installed_harnesses[0]["version"],
            serde_json::json!("v3.31.5")
        );

        let settings_path = pi_dir.join("settings.json");
        assert!(settings_path.exists());
        let settings_val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert_eq!(
            settings_val["compound_engineering_version"],
            serde_json::json!("v3.31.5")
        );
    }

    #[test]
    fn test_handle_update_dry_run_does_not_mutate_state() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        std::fs::create_dir_all(&ctx.config_dir).unwrap();
        let mut state = State::new();
        state.fleet = Some(crate::state::state::FleetState {
            pinned_version: Some("v3.30.0".to_string()),
            last_sync: None,
        });
        state.save(&ctx.state_path()).unwrap();

        let update_args = UpdateArgs {
            version: None,
            to: Some("3.32.0".to_string()),
            dry_run: true,
            bin: false,
            system: None,
        };

        handle_update(&ctx, &update_args).unwrap();

        let reloaded = State::load(&ctx.state_path()).unwrap();
        // Pinned version in state MUST remain unchanged during dry-run
        assert_eq!(
            reloaded.fleet.unwrap().pinned_version,
            Some("v3.30.0".to_string())
        );
    }
}
