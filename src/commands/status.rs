//! `ce-ai status`: print installed harnesses, versions, and drift (CC-1).

use std::collections::BTreeMap;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::diff::{self, Action};
use crate::state::state::State;

use crate::harness::HarnessKind;
use std::collections::HashSet;

pub fn run(ctx: &Context) -> Result<(), CeError> {
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    let mut installed_list = Vec::new();
    let mut seen = HashSet::new();

    for harness in &state.installed_harnesses {
        let name = harness["name"].as_str().unwrap_or("?").to_string();
        let ver = harness["version"].as_str().unwrap_or("?").to_string();
        let src = harness["source"]["kind"]
            .as_str()
            .unwrap_or("?")
            .to_string();
        seen.insert(name.clone());
        installed_list.push((name, ver, src));
    }

    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::Path::new(&home);
        for h in HarnessKind::detect_ce_installed_harnesses(home_path) {
            let name = h.to_string();
            if !seen.contains(&name) {
                seen.insert(name.clone());
                installed_list.push((name, "host-detected".to_string(), "local".to_string()));
            }
        }
    }

    if installed_list.is_empty() {
        println!("installed: none");
    } else {
        for (name, version, source) in &installed_list {
            println!("installed: {name} ({version}, source: {source})");
            if ctx.verbose {
                println!("  verbose: enabled for {name}");
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
