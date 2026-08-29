use super::*;
use tempfile::TempDir;

#[test]
fn test_doctor_strict_flag_default() {
    let args = Args::default();
    assert!(!args.strict);
}

#[test]
fn test_doctor_runs_on_clean_context() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::write(
        ctx.config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();
    let args = Args::default();
    assert!(run(&ctx, &args).is_ok());
}

mod branch_protection_tests {
    use super::github_slug_from_url;

    #[test]
    fn github_slug_parses_ssh_https_and_rejects_other_hosts() {
        assert_eq!(
            github_slug_from_url("git@github.com:mastepanoski/ce-ai.git").as_deref(),
            Some("mastepanoski/ce-ai")
        );
        assert_eq!(
            github_slug_from_url("https://github.com/mastepanoski/ce-ai/").as_deref(),
            Some("mastepanoski/ce-ai")
        );
        assert_eq!(
            github_slug_from_url("https://gitlab.com/group/proj.git"),
            None
        );
    }
}
