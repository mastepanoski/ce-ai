//! TUI CLI spawn vectors (KTD3, R5) — extracted from monolithic tui.rs.
//! Pure arg builders, no I/O.

pub fn status_args() -> Vec<String> {
    vec!["status".into()]
}

pub fn install_cmd_args(harness: &str, dry_run: bool) -> Vec<String> {
    let mut args = vec!["install".into(), "--harness".into(), harness.into()];
    if dry_run {
        args.push("--dry-run".into());
    }
    args
}

pub fn models_list_args() -> Vec<String> {
    vec!["models".into(), "list".into()]
}

pub fn sync_cmd_args(dry_run: bool) -> Vec<String> {
    let mut args = vec!["sync".into()];
    if dry_run {
        args.push("--dry-run".into());
    }
    args
}

pub fn workflow_status_args() -> Vec<String> {
    vec!["workflow".into(), "status".into()]
}

pub fn upgrade_cmd_args() -> Vec<String> {
    vec!["upgrade".into()]
}

pub fn doctor_cmd_args() -> Vec<String> {
    vec!["doctor".into()]
}

pub fn uninstall_cmd_args(harness: &str) -> Vec<String> {
    vec![
        "uninstall".into(),
        "--harness".into(),
        harness.into(),
        "--yes".into(),
    ]
}

pub fn init_prj_args() -> Vec<String> {
    vec!["init-prj".into()]
}

pub fn skills_list_args() -> Vec<String> {
    vec!["skills".into(), "list".into()]
}

pub fn tools_status_args() -> Vec<String> {
    vec!["tools".into(), "status".into()]
}

pub fn usage_report_args() -> Vec<String> {
    vec!["usage".into(), "report".into()]
}

pub fn audit_args() -> Vec<String> {
    vec!["audit".into()]
}
