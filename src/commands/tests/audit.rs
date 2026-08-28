use super::*;
use tempfile::TempDir;

#[test]
fn test_audit_score_math() {
    let checks = vec![
        AuditCheck {
            id: "c1".into(),
            category: "repo".into(),
            status: AuditStatus::Pass,
            satisfied_by: None,
            detail: "ok".into(),
        },
        AuditCheck {
            id: "c2".into(),
            category: "repo".into(),
            status: AuditStatus::Warn,
            satisfied_by: None,
            detail: "warn".into(),
        },
    ];
    let (score, pass, warn) = AuditReport::compute_score(&checks);
    assert_eq!(score, 75);
    assert_eq!(pass, 1);
    assert_eq!(warn, 1);
}

#[test]
fn test_audit_run_runs_cleanly() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    let args = Args::default();
    assert!(run(&ctx, &args).is_ok());
}
