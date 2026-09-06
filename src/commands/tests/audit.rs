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

#[test]
fn test_cli_compression_detector_supported_vs_unsupported() {
    let tmp = TempDir::new().unwrap();
    let audit_ctx = AuditCtx {
        home_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        repo_root: None,
    };

    let detector = CliCompressionDetector;
    let harnesses = vec![HarnessKind::Claude, HarnessKind::Opencode, HarnessKind::Pi];
    let checks = detector.detect(&audit_ctx, &harnesses);

    assert_eq!(checks.len(), 3);

    // Claude (supported, hook missing in fresh temp dir) -> Warn
    let claude_check = checks
        .iter()
        .find(|c| c.id == "cli-compression/claude")
        .unwrap();
    assert_eq!(claude_check.status, AuditStatus::Warn);

    // Opencode (unsupported) -> Info
    let opencode_check = checks
        .iter()
        .find(|c| c.id == "cli-compression/opencode")
        .unwrap();
    assert_eq!(opencode_check.status, AuditStatus::Info);

    // Pi (unsupported) -> Info
    let pi_check = checks
        .iter()
        .find(|c| c.id == "cli-compression/pi")
        .unwrap();
    assert_eq!(pi_check.status, AuditStatus::Info);
}
