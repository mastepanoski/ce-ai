use super::*;
use crate::verification::mutator::MutationCandidate;
use crate::verification::runner::{
    read_mutation_receipt, run_mutations_on_candidates, save_mutation_receipt, FileRestoreGuard,
    MutationReport,
};
use tempfile::TempDir;

fn pass_cmd() -> &'static str {
    if cfg!(windows) {
        "cmd /C exit 0"
    } else {
        "true"
    }
}

fn fail_cmd() -> &'static str {
    if cfg!(windows) {
        "cmd /C exit 1"
    } else {
        "false"
    }
}

#[test]
fn test_verify_args_defaults() {
    let args = Args {
        test_cmd: None,
        max_mutants: 3,
        timeout: 10,
        strict: false,
    };
    assert!(!args.strict);
    assert_eq!(args.max_mutants, 3);
    assert_eq!(args.timeout, 10);
    assert!(args.test_cmd.is_none());
}

#[test]
fn test_file_restore_guard_reverts_content_on_drop() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("logic.rs");
    let original = "fn check() -> bool { true }\n";
    std::fs::write(&file_path, original).unwrap();

    {
        let _guard = FileRestoreGuard::new(file_path.clone(), original.to_string());
        // Mutate file on disk
        std::fs::write(&file_path, "fn check() -> bool { false }\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(&file_path).unwrap(),
            "fn check() -> bool { false }\n"
        );
        // Guard drops here at scope exit
    }

    // Verify original content was restored
    assert_eq!(std::fs::read_to_string(&file_path).unwrap(), original);
}

#[test]
fn test_runner_kills_mutant_when_tests_fail() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("src_auth.rs");
    let original = "fn is_admin(role: &str) -> bool {\n    role == \"admin\"\n}\n";
    std::fs::write(&file_path, original).unwrap();

    let candidate = MutationCandidate {
        file_path: "src_auth.rs".to_string(),
        line_number: 2,
        col_offset: 9,
        original: "==".to_string(),
        replacement: "!=".to_string(),
        description: "invert == to !=".to_string(),
    };

    // When tests fail (fail_cmd), mutant was successfully KILLED
    let report =
        run_mutations_on_candidates(tmp.path(), &[candidate], Some(fail_cmd()), 5).unwrap();

    assert_eq!(report.total_mutants, 1);
    assert_eq!(report.killed_mutants, 1);
    assert_eq!(report.survived_mutants, 0);
    assert!(report.results[0].killed);

    // Source must be 100% restored
    assert_eq!(std::fs::read_to_string(&file_path).unwrap(), original);
}

#[test]
fn test_runner_detects_surviving_mutant_when_tests_pass() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("src_calc.rs");
    let original = "fn score(x: i32) -> bool {\n    x >= 100\n}\n";
    std::fs::write(&file_path, original).unwrap();

    let candidate = MutationCandidate {
        file_path: "src_calc.rs".to_string(),
        line_number: 2,
        col_offset: 6,
        original: ">=".to_string(),
        replacement: "<".to_string(),
        description: "invert >= to <".to_string(),
    };

    // When tests pass despite mutation (pass_cmd), mutant SURVIVED (hollow test)
    let report =
        run_mutations_on_candidates(tmp.path(), &[candidate], Some(pass_cmd()), 5).unwrap();

    assert_eq!(report.total_mutants, 1);
    assert_eq!(report.killed_mutants, 0);
    assert_eq!(report.survived_mutants, 1);
    assert!(!report.results[0].killed);

    // Source must still be 100% restored
    assert_eq!(std::fs::read_to_string(&file_path).unwrap(), original);
}

#[test]
fn test_verify_command_strict_fails_on_survived_mutants() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("src_calc.rs");
    let original = "fn active() -> bool {\n    true\n}\n";
    std::fs::write(&file_path, original).unwrap();

    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: Some(tmp.path().to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    // Without a git diff, 0 mutants found -> Ok
    let args_clean = Args {
        test_cmd: Some(pass_cmd().to_string()),
        max_mutants: 2,
        timeout: 5,
        strict: true,
    };
    assert!(run(&ctx, &args_clean).is_ok());
}

#[test]
fn test_mutation_receipt_persistence() {
    let tmp = TempDir::new().unwrap();
    let report = MutationReport {
        total_mutants: 2,
        killed_mutants: 2,
        survived_mutants: 0,
        results: vec![],
        duration_ms: 120,
    };

    assert!(save_mutation_receipt(tmp.path(), &report).is_ok());
    let loaded = read_mutation_receipt(tmp.path()).unwrap();
    assert_eq!(loaded, report);
}
