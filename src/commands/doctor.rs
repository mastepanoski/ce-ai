//! `ce-ai doctor`: report config-validity, diff (drift), and state-consistency
//! findings; exits non-zero when any finding exists.

use std::collections::BTreeMap;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::read_config;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::diff::{self, Action};
use crate::state::state::State;

pub fn run(ctx: &Context) -> Result<(), CeError> {
    let mut findings: Vec<String> = Vec::new();

    // Config validity: opencode.json must parse (D4).
    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    if let Err(err) = read_config(&opencode_json) {
        findings.push(format!("config-invalid: {err}"));
    }

    // Diff: managed files vs the install manifest (SU-3).
    let manifest = InstallManifest::load(&ctx.opencode_config_dir);
    if let Ok(manifest) = &manifest {
        let desired: BTreeMap<String, String> =
            manifest.files.iter().map(|f| (f.path.clone(), f.sha256.clone())).collect();
        let managed = ctx.opencode_config_dir.join(MANAGED_DIR);
        for action in diff::diff(&desired, &desired, &managed).actions {
            let (kind, path) = match action {
                Action::Copy { path } => ("missing", path),
                Action::Restore { path } => ("modified", path),
                Action::Remove { path } => ("stale", path),
            };
            findings.push(format!("diff: {kind} {path}"));
        }
    }

    // State consistency: the opencode state entry and the manifest must agree.
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    let has_entry = state.installed_harnesses.iter().any(|h| h["name"].as_str() == Some("opencode"));
    if has_entry != manifest.is_ok() {
        findings.push("state-inconsistent: opencode state entry and install manifest disagree".into());
    }

    for finding in &findings {
        println!("{finding}");
    }
    if findings.is_empty() {
        println!("doctor: ok");
        return Ok(());
    }
    Err(CeError::Runtime(format!("doctor found {} finding(s)", findings.len())))
}