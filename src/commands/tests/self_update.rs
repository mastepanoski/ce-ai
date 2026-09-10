use tempfile::tempdir;

use crate::commands::self_update::{run_internal, Args};
use crate::commands::Context;
use crate::error::CeError;

fn ctx_in(dir: &std::path::Path) -> Context {
    Context {
        config_dir: dir.to_path_buf(),
        opencode_config_dir: dir.join(".config/opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    }
}

#[test]
fn test_self_update_unsupported_target_returns_usage_error() {
    let dir = tempdir().unwrap();
    let ctx = ctx_in(dir.path());
    let args = Args {
        check: false,
        to: None,
        force: false,
    };
    let target_bin = dir.path().join("ce-ai");

    let err = run_internal(&ctx, &args, None, Some(target_bin), None).unwrap_err();
    match err {
        CeError::Usage(ref msg) => {
            assert!(msg.contains("Unsupported host architecture"));
            assert_eq!(err.exit_code(), 2);
        }
        _ => panic!("Expected CeError::Usage, got {err:?}"),
    }
}

#[test]
fn test_self_update_check_flag_does_not_mutate_disk() {
    let dir = tempdir().unwrap();
    let ctx = ctx_in(dir.path());
    let args = Args {
        check: true,
        to: None,
        force: false,
    };
    let target_bin = dir.path().join("ce-ai");
    std::fs::write(&target_bin, b"original-v1.49.0").unwrap();

    // With unsupported or mock target, check flag still handles logic
    let target = Some("x86_64-unknown-linux-gnu");
    // Even if network fails or mock release is evaluated, check mode must not mutate target_bin
    let _ = run_internal(&ctx, &args, target, Some(target_bin.clone()), None);
    let current_data = std::fs::read(&target_bin).unwrap();
    assert_eq!(current_data, b"original-v1.49.0");
}

#[test]
fn test_self_update_dry_run_flag() {
    let dir = tempdir().unwrap();
    let mut ctx = ctx_in(dir.path());
    ctx.dry_run = true;
    let args = Args {
        check: false,
        to: None,
        force: false,
    };
    let target_bin = dir.path().join("ce-ai");
    std::fs::write(&target_bin, b"original-bytes").unwrap();

    let _ = run_internal(
        &ctx,
        &args,
        Some("x86_64-unknown-linux-gnu"),
        Some(target_bin.clone()),
        None,
    );
    assert_eq!(std::fs::read(&target_bin).unwrap(), b"original-bytes");
}

#[test]
fn test_upgrade_command_bin_flag_wiring() {
    use clap::Parser;

    #[derive(Parser)]
    struct Wrapper {
        #[command(subcommand)]
        cmd: crate::commands::registry::Commands,
    }

    let parsed = Wrapper::try_parse_from(["ce-ai", "upgrade", "--bin"]).unwrap();
    match parsed.cmd {
        crate::commands::registry::Commands::Upgrade(args) => {
            assert!(args.bin);
        }
        _ => panic!("Expected Commands::Upgrade"),
    }
}

#[test]
fn test_self_update_command_parsing() {
    use clap::Parser;

    #[derive(Parser)]
    struct Wrapper {
        #[command(subcommand)]
        cmd: crate::commands::registry::Commands,
    }

    let parsed = Wrapper::try_parse_from(["ce-ai", "self-update", "--check"]).unwrap();
    match parsed.cmd {
        crate::commands::registry::Commands::SelfUpdate(args) => {
            assert!(args.check);
            assert!(!args.force);
            assert_eq!(args.to, None);
        }
        _ => panic!("Expected Commands::SelfUpdate"),
    }

    let parsed_to =
        Wrapper::try_parse_from(["ce-ai", "self-update", "--to", "v1.50.0", "--force"]).unwrap();
    match parsed_to.cmd {
        crate::commands::registry::Commands::SelfUpdate(args) => {
            assert!(!args.check);
            assert!(args.force);
            assert_eq!(args.to, Some("v1.50.0".to_string()));
        }
        _ => panic!("Expected Commands::SelfUpdate"),
    }
}
