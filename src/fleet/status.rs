//! Fleet auditing and status computation across installed harnesses.

use crate::commands::Context;
use crate::error::CeError;
use crate::fleet::driver::{FleetHarnessStatus, FleetStatusReport, HarnessAlignment};
use crate::fleet::drivers::{all_drivers, is_version_match};

/// Computes the fleet status report comparing installed harness versions against the pinned version.
pub fn generate_fleet_status_report(ctx: &Context) -> Result<FleetStatusReport, CeError> {
    let state_file = ctx.state_path();
    let pinned_version = if state_file.exists() {
        let state = crate::state::state::State::load(&state_file)?;
        state.fleet.and_then(|f| f.pinned_version)
    } else {
        None
    };

    let mut harnesses = Vec::new();
    let mut all_aligned = true;
    let mut any_installed = false;

    for driver in all_drivers() {
        let is_inst = driver.is_installed(ctx);
        let installed_ver = driver.detect_version(ctx)?;
        if is_inst || installed_ver.is_some() {
            any_installed = true;
            let alignment = match (&pinned_version, &installed_ver) {
                (Some(pinned), Some(inst)) => {
                    if is_version_match(inst, pinned) {
                        HarnessAlignment::Aligned
                    } else {
                        all_aligned = false;
                        HarnessAlignment::Divergent {
                            installed: inst.clone(),
                            pinned: pinned.clone(),
                        }
                    }
                }
                (Some(pinned), None) => {
                    all_aligned = false;
                    HarnessAlignment::Missing {
                        pinned: pinned.clone(),
                    }
                }
                (None, Some(inst)) => {
                    all_aligned = false;
                    HarnessAlignment::Unmanaged {
                        installed: inst.clone(),
                    }
                }
                (None, None) => HarnessAlignment::Aligned,
            };

            harnesses.push(FleetHarnessStatus {
                harness: driver.harness_kind(),
                installed_version: installed_ver,
                is_installed: is_inst,
                alignment,
            });
        }
    }

    let is_aligned = pinned_version.is_some() && any_installed && all_aligned;

    Ok(FleetStatusReport {
        pinned_version,
        harnesses,
        is_aligned,
    })
}
