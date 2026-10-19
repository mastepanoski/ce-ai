use super::*;
use tempfile::TempDir;

fn test_ctx() -> (TempDir, Context) {
    let tmp = TempDir::new().unwrap();
    let ctx = Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true).unwrap();
    (tmp, ctx)
}

#[test]
fn guard_enable_persists_state_and_level() {
    let (_tmp, ctx) = test_ctx();
    run_guard_enable(&ctx, "strict", Some("claude")).unwrap();

    let state = State::load(&ctx.state_path()).unwrap();
    let guard = state.guardrail.expect("guardrail should exist");
    assert!(guard.enabled);
    assert_eq!(guard.level, GuardLevel::Strict);
    assert_eq!(guard.harness.as_deref(), Some("claude"));
}

#[test]
fn guard_disable_cleans_flag() {
    let (_tmp, ctx) = test_ctx();
    run_guard_enable(&ctx, "junior", None).unwrap();
    run_guard_disable(&ctx, None).unwrap();

    let state = State::load(&ctx.state_path()).unwrap();
    let guard = state.guardrail.expect("guardrail should exist");
    assert!(!guard.enabled);
}

#[test]
fn guard_disable_rejects_a_different_harness_scope_without_mutating_state() {
    let (_tmp, ctx) = test_ctx();
    run_guard_enable(&ctx, "junior", Some("claude")).unwrap();

    let err = run_guard_disable(&ctx, Some("codex")).unwrap_err();
    assert!(matches!(err, CeError::Usage(_)));

    let state = State::load(&ctx.state_path()).unwrap();
    let guard = state.guardrail.expect("guardrail should remain configured");
    assert!(guard.enabled);
    assert_eq!(guard.harness.as_deref(), Some("claude"));
}

#[test]
fn guard_invalid_level_fails_fast() {
    let (_tmp, ctx) = test_ctx();
    let err = run_guard_enable(&ctx, "extreme", None).unwrap_err();
    assert!(matches!(err, CeError::Usage(_)));
}

#[test]
fn guard_dry_run_writes_nothing() {
    let (_tmp, mut ctx) = test_ctx();
    ctx.dry_run = true;
    run_guard_enable(&ctx, "strict", None).unwrap();

    assert!(!ctx.state_path().exists());
}

#[test]
fn guard_check_catastrophic_command_denied_deterministically() {
    let (_tmp, ctx) = test_ctx();
    // Test check on rm -rf /
    let res = run_guard_check(&ctx, "run_command", "rm -rf /", None, true, false);
    assert!(res.is_ok());

    let log_file = crate::decisions::risk_events_log_path(&ctx.config_dir);
    assert!(log_file.exists());
    let log_content = std::fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("\"policy\":\"deny\""));
}

#[test]
fn guard_check_safe_command_allowed() {
    let (_tmp, ctx) = test_ctx();
    let res = run_guard_check(&ctx, "run_command", "cargo test", None, false, false);
    assert!(res.is_ok());
}

#[test]
fn guard_setup_and_status() {
    let (_tmp, ctx) = test_ctx();
    run_guard_setup(&ctx, "recommended", None, None).unwrap();

    let state = State::load(&ctx.state_path()).unwrap();
    let dec = state.decisions.expect("decisions should exist");
    assert!(dec.enabled);
    assert!(dec.risk.enabled);

    // Status json check
    let res = run_guard_status(&ctx, true);
    assert!(res.is_ok());
}

#[test]
fn guard_test_runs_successfully() {
    let (_tmp, ctx) = test_ctx();
    let res = run_guard_test(&ctx);
    assert!(res.is_ok());
}
