//! Fleet harness driver trait and core action / status data structures.

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetAction {
    UpToDate,
    RunCommand {
        program: String,
        args: Vec<String>,
        description: String,
    },
    UpdateConfig {
        file_path: PathBuf,
        key: String,
        value: serde_json::Value,
        description: String,
    },
}

impl FleetAction {
    pub fn execute(&self, dry_run: bool) -> Result<(), CeError> {
        match self {
            FleetAction::UpToDate => Ok(()),
            FleetAction::RunCommand {
                program,
                args,
                description: _,
            } => {
                if dry_run {
                    return Ok(());
                }
                let status = std::process::Command::new(program)
                    .args(args)
                    .status()
                    .map_err(CeError::Io)?;
                if !status.success() {
                    return Err(CeError::Runtime(format!(
                        "command '{program} {args:?}' exited with non-zero status: {:?}",
                        status.code()
                    )));
                }
                Ok(())
            }
            FleetAction::UpdateConfig {
                file_path,
                key,
                value,
                description: _,
            } => {
                if dry_run {
                    return Ok(());
                }
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut root: serde_json::Value = if file_path.exists() {
                    let content = std::fs::read_to_string(file_path)?;
                    serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
                } else {
                    serde_json::json!({})
                };
                if let Some(obj) = root.as_object_mut() {
                    obj.insert(key.clone(), value.clone());
                }
                let pretty = serde_json::to_string_pretty(&root)
                    .map_err(|e| CeError::State(format!("failed to serialize json: {e}")))?;
                crate::state::write_atomic(file_path, pretty.as_bytes())?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessAlignment {
    Aligned,
    Divergent { installed: String, pinned: String },
    Missing { pinned: String },
    Unmanaged { installed: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetHarnessStatus {
    pub harness: HarnessKind,
    pub installed_version: Option<String>,
    pub is_installed: bool,
    pub alignment: HarnessAlignment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetStatusReport {
    pub pinned_version: Option<String>,
    pub harnesses: Vec<FleetHarnessStatus>,
    pub is_aligned: bool,
}

pub trait FleetHarnessDriver: Send + Sync {
    fn harness_kind(&self) -> HarnessKind;
    fn is_installed(&self, ctx: &Context) -> bool;
    fn detect_version(&self, ctx: &Context) -> Result<Option<String>, CeError>;
    fn plan_sync(&self, ctx: &Context, target_version: &str) -> Result<FleetAction, CeError>;
}
