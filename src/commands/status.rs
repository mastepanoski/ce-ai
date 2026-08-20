//! `ce-ai status`: print installed harnesses, versions, and drift (CC-1).

use std::collections::BTreeMap;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::diff::{self, Action};
use crate::state::state::State;

pub fn run(ctx: &Context) -> Result<(), CeError> {
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    if state.installed_harnesses.is_empty() {
        println!("installed: none");
    } else {
        for harness in &state.installed_harnesses {
            println!(
                "installed: {} ({}, source: {})",
                harness["name"].as_str().unwrap_or("?"),
                harness["version"].as_str().unwrap_or("?"),
                harness["source"]["kind"].as_str().unwrap_or("?")
            );
            if ctx.verbose {
                if let Some(at) = harness["installed_at"].as_str() {
                    println!("  installed_at: {at}");
                }
            }
        }
    }

    // Drift: compare managed files on disk against the install manifest (SU-3).
    let managed = ctx.opencode_config_dir.join(MANAGED_DIR);
    match InstallManifest::load(&ctx.opencode_config_dir) {
        Ok(manifest) => {
            let desired: BTreeMap<String, String> = manifest
                .files
                .iter()
                .map(|f| (f.path.clone(), f.sha256.clone()))
                .collect();
            let drift = diff::diff(&desired, &desired, &managed);
            if drift.actions.is_empty() {
                println!("drift: none");
            }
            for action in &drift.actions {
                let (kind, path) = match action {
                    Action::Copy { path } => ("missing", path),
                    Action::Restore { path } => ("modified", path),
                    Action::Remove { path } => ("stale", path),
                };
                println!("drift: {kind} {path}");
            }
        }
        Err(_) => println!("drift: unknown (no install manifest)"),
    }
    Ok(())
}
