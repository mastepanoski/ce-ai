use std::path::Path;
use tempfile::TempDir;

use crate::commands::archive_compact::*;
use crate::commands::workflow::ProbeStatus;
use crate::state::state::DocHygieneConfig;

fn create_sample_package(
    archive_dir: &Path,
    folder_name: &str,
    proposal: &str,
    spec: &str,
    completed: usize,
    total: usize,
) {
    let pkg_dir = archive_dir.join(folder_name);
    std::fs::create_dir_all(&pkg_dir).unwrap();

    let proposal_content = format!(
        "# Proposal: Test Feature\n\n## Problem Statement\n{}\n",
        proposal
    );
    std::fs::write(pkg_dir.join("proposal.md"), proposal_content).unwrap();

    let spec_content = format!(
        "# Specification\n\n## Scenario: Primary Flow\n- **WHEN** user executes\n- **THEN** {}\n",
        spec
    );
    std::fs::write(pkg_dir.join("spec.md"), spec_content).unwrap();

    let mut tasks_content = String::from("# Tasks\n\n");
    for i in 1..=total {
        let mark = if i <= completed { "[x]" } else { "[ ]" };
        tasks_content.push_str(&format!("- {} Task {}\n", mark, i));
    }
    std::fs::write(pkg_dir.join("tasks.md"), tasks_content).unwrap();
}

#[test]
fn test_date_to_quarter() {
    let q1 = chrono::NaiveDate::from_ymd_opt(2026, 2, 15).unwrap();
    assert_eq!(date_to_quarter(q1), "2026-Q1");

    let q2 = chrono::NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    assert_eq!(date_to_quarter(q2), "2026-Q2");

    let q3 = chrono::NaiveDate::from_ymd_opt(2026, 8, 26).unwrap();
    assert_eq!(date_to_quarter(q3), "2026-Q3");

    let q4 = chrono::NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
    assert_eq!(date_to_quarter(q4), "2026-Q4");
}

#[test]
fn test_extract_feature_slug() {
    assert_eq!(
        extract_feature_slug("2026-08-26-canonical-skills-adoption"),
        "canonical-skills-adoption"
    );
    assert_eq!(
        extract_feature_slug("blocking-gate-check"),
        "blocking-gate-check"
    );
    assert_eq!(extract_feature_slug("2026-09-12-short"), "short");
}

#[test]
fn test_resolve_archive_package_date_prefix_and_fallback() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    create_sample_package(
        &archive_dir,
        "2026-07-20-test-feature",
        "Problem A",
        "Spec A",
        5,
        5,
    );
    let pkg_path = archive_dir.join("2026-07-20-test-feature");

    // Prefix parsed without needing git
    let date = resolve_archive_package_date(root, &pkg_path, "2026-07-20-test-feature");
    assert_eq!(date, chrono::NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());

    // Legacy folder without prefix falls back to mtime or current date
    create_sample_package(&archive_dir, "legacy-folder", "Problem B", "Spec B", 3, 3);
    let legacy_path = archive_dir.join("legacy-folder");
    let legacy_date = resolve_archive_package_date(root, &legacy_path, "legacy-folder");
    assert_eq!(legacy_date, chrono::Utc::now().date_naive());
}

#[test]
fn test_collect_candidates_and_filtering() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    create_sample_package(
        &archive_dir,
        "2026-02-10-feat-q1",
        "Problem Q1",
        "Spec Q1",
        5,
        5,
    );
    create_sample_package(
        &archive_dir,
        "2026-08-15-feat-q3",
        "Problem Q3",
        "Spec Q3",
        10,
        10,
    );
    create_sample_package(
        &archive_dir,
        "2026-09-01-feat-sept",
        "Problem Sept",
        "Spec Sept",
        3,
        3,
    );

    // Reserved milestones directory and README must be skipped
    std::fs::create_dir_all(archive_dir.join("milestones")).unwrap();
    std::fs::write(archive_dir.join("README.md"), "# Archive Ledger\n").unwrap();

    let all_cands = collect_compaction_candidates(root, None);
    assert_eq!(all_cands.len(), 3);
    assert_eq!(all_cands[0].feature_name, "feat-q1");
    assert_eq!(all_cands[1].feature_name, "feat-q3");
    assert_eq!(all_cands[2].feature_name, "feat-sept");

    // Date filtering: before 2026-09-01
    let cutoff = chrono::NaiveDate::from_ymd_opt(2026, 9, 1);
    let filtered = collect_compaction_candidates(root, cutoff);
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].feature_name, "feat-q1");
    assert_eq!(filtered[1].feature_name, "feat-q3");
}

#[test]
fn test_group_candidates_by_milestone() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    create_sample_package(&archive_dir, "2026-02-10-feat-a", "P A", "S A", 5, 5);
    create_sample_package(&archive_dir, "2026-08-15-feat-b", "P B", "S B", 10, 10);

    let cands = collect_compaction_candidates(root, None);

    // Default: quarterly grouping
    let quarter_groups = group_candidates_by_milestone(cands.clone(), None);
    assert_eq!(quarter_groups.len(), 2);
    assert!(quarter_groups.contains_key("2026-Q1"));
    assert!(quarter_groups.contains_key("2026-Q3"));

    // Explicit custom milestone
    let custom_groups = group_candidates_by_milestone(cands, Some("custom-2026"));
    assert_eq!(custom_groups.len(), 1);
    assert!(custom_groups.contains_key("custom-2026"));
    assert_eq!(custom_groups["custom-2026"].len(), 2);
}

#[test]
fn test_generate_milestone_markdown() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    create_sample_package(
        &archive_dir,
        "2026-08-20-test-feature",
        "This is an important problem to solve.",
        "System behaves deterministically.",
        4,
        4,
    );

    let cands = collect_compaction_candidates(root, None);
    let md = generate_milestone_markdown("2026-Q3", &cands, Some("archive-2026-Q3.tar.gz"));

    assert!(md.contains("# Milestone Archive Rollup: 2026-Q3"));
    assert!(md.contains("- **Total Changes Compacted:** 1"));
    assert!(md.contains("[archive-2026-Q3.tar.gz](archive-2026-Q3.tar.gz)"));
    assert!(md.contains("| [test-feature](#test-feature) | 2026-08-20 | 4/4 |"));
    assert!(md.contains("### test-feature"));
    assert!(md.contains("This is an important problem to solve."));
    assert!(md.contains("System behaves deterministically."));
}

#[test]
fn test_create_and_verify_milestone_tarball() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    create_sample_package(&archive_dir, "2026-08-20-feat-a", "P A", "S A", 2, 2);
    create_sample_package(&archive_dir, "2026-08-21-feat-b", "P B", "S B", 3, 3);

    let cands = collect_compaction_candidates(root, None);
    let tarball_path = tmp.path().join("test-archive.tar.gz");

    create_milestone_tarball(&tarball_path, &cands).unwrap();
    assert!(tarball_path.exists());
    assert!(std::fs::metadata(&tarball_path).unwrap().len() > 0);

    // Verification succeeds
    assert!(verify_milestone_tarball(&tarball_path, &cands).is_ok());

    // Missing candidate in expected triggers verification failure
    let dummy_cand = CompactionCandidate {
        folder_name: "non-existent-folder".into(),
        feature_name: "non-existent".into(),
        path: tmp.path().join("missing"),
        date: chrono::Utc::now().date_naive(),
        quarter: "2026-Q3".into(),
        tasks_progress: (0, 0),
        proposal_summary: String::new(),
        spec_summary: String::new(),
    };
    let bad_expected = vec![dummy_cand];
    assert!(verify_milestone_tarball(&tarball_path, &bad_expected).is_err());
}

#[test]
fn test_update_archive_readme_ledger() {
    let tmp = TempDir::new().unwrap();
    let readme_path = tmp.path().join("README.md");

    let initial = "# OpenSpec Change Archive\n\nCompleted change folders live here.\n\n## Triage — active folders with open tasks\n\n| Folder | Open boxes |\n";
    std::fs::write(&readme_path, initial).unwrap();

    update_archive_readme_ledger(&readme_path, "2026-Q3", 42, Some("archive-2026-Q3.tar.gz"))
        .unwrap();

    let updated = std::fs::read_to_string(&readme_path).unwrap();
    assert!(updated.contains("## Compacted Milestones"));
    assert!(updated.contains("| 2026-Q3 | 42 | [milestones/2026-Q3.md](milestones/2026-Q3.md) | [milestones/archive-2026-Q3.tar.gz](milestones/archive-2026-Q3.tar.gz) |"));
    assert!(updated.contains("## Triage"));

    // Idempotent: running again does not duplicate row
    update_archive_readme_ledger(&readme_path, "2026-Q3", 42, Some("archive-2026-Q3.tar.gz"))
        .unwrap();
    let second_updated = std::fs::read_to_string(&readme_path).unwrap();
    assert_eq!(updated, second_updated);

    // Appending a second milestone works cleanly
    update_archive_readme_ledger(&readme_path, "2026-Q2", 15, None).unwrap();
    let third_updated = std::fs::read_to_string(&readme_path).unwrap();
    assert!(third_updated
        .contains("| 2026-Q2 | 15 | [milestones/2026-Q2.md](milestones/2026-Q2.md) | *(none)* |"));
}

#[test]
fn test_probe_archive_compaction_clean_and_debt() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    let config = DocHygieneConfig {
        stale_spec_days: 21,
        check_solution_paths: true,
        require_solution_frontmatter: true,
        archive_compaction_threshold: 2,
    };

    // 1 package <= 2 threshold -> Clean
    create_sample_package(&archive_dir, "2026-08-01-pkg-1", "P", "S", 1, 1);
    assert_eq!(probe_archive_compaction(root, &config), ProbeStatus::Clean);

    // 2 packages <= 2 threshold -> Clean
    create_sample_package(&archive_dir, "2026-08-02-pkg-2", "P", "S", 1, 1);
    assert_eq!(probe_archive_compaction(root, &config), ProbeStatus::Clean);

    // 3 packages > 2 threshold -> Debt
    create_sample_package(&archive_dir, "2026-08-03-pkg-3", "P", "S", 1, 1);
    let probe = probe_archive_compaction(root, &config);
    if let ProbeStatus::Debt(finding) = probe {
        assert_eq!(finding.uncompacted_count, 3);
        assert_eq!(finding.threshold, 2);
        assert_eq!(finding.oldest_package, Some("2026-08-01-pkg-1".into()));
    } else {
        panic!("expected ProbeStatus::Debt when packages exceed threshold");
    }
}

#[test]
fn test_run_archive_compact_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();
    std::fs::write(archive_dir.join("README.md"), "# Archive\n\n## Triage\n").unwrap();

    create_sample_package(&archive_dir, "2026-07-01-feat-1", "P1", "S1", 2, 2);
    create_sample_package(&archive_dir, "2026-07-15-feat-2", "P2", "S2", 3, 3);

    let ctx = crate::commands::Context {
        config_dir: tmp.path().join(".ce-ai"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: Some(root.to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    // 1. Dry run: disk remains untouched
    let dry_args = CompactArgs {
        dry_run: true,
        ..Default::default()
    };
    run_archive_compact(&ctx, &dry_args).unwrap();
    assert!(archive_dir.join("2026-07-01-feat-1").exists());
    assert!(!archive_dir.join("milestones").exists());

    // 2. Real compaction execution
    let exec_args = CompactArgs {
        dry_run: false,
        ..Default::default()
    };
    run_archive_compact(&ctx, &exec_args).unwrap();

    // Loose directories pruned
    assert!(!archive_dir.join("2026-07-01-feat-1").exists());
    assert!(!archive_dir.join("2026-07-15-feat-2").exists());

    // Milestones directory created with rollup and tarball
    let milestones_dir = archive_dir.join("milestones");
    assert!(milestones_dir.join("2026-Q3.md").exists());
    assert!(milestones_dir.join("archive-2026-Q3.tar.gz").exists());

    // README updated with milestone table
    let readme = std::fs::read_to_string(archive_dir.join("README.md")).unwrap();
    assert!(readme.contains("## Compacted Milestones"));
    assert!(readme.contains("2026-Q3"));
}

#[test]
fn test_run_archive_compact_keep_loose_and_no_tarball() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    create_sample_package(&archive_dir, "2026-05-10-feat-keep", "PK", "SK", 1, 1);

    let ctx = crate::commands::Context {
        config_dir: tmp.path().join(".ce-ai"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: Some(root.to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let args = CompactArgs {
        keep_loose: true,
        no_tarball: true,
        ..Default::default()
    };
    run_archive_compact(&ctx, &args).unwrap();

    // Loose directory preserved due to keep_loose
    assert!(archive_dir.join("2026-05-10-feat-keep").exists());

    // Rollup created, but no tarball due to no_tarball
    let milestones_dir = archive_dir.join("milestones");
    assert!(milestones_dir.join("2026-Q2.md").exists());
    assert!(!milestones_dir.join("archive-2026-Q2.tar.gz").exists());
}

#[test]
fn test_run_archive_compact_no_tarball_alone_preserves_loose_directories() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    create_sample_package(
        &archive_dir,
        "2026-06-15-feat-no-tb",
        "Problem Statement",
        "Spec",
        1,
        1,
    );

    let ctx = crate::commands::Context {
        config_dir: tmp.path().join(".ce-ai"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: Some(root.to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    // keep_loose: false, but no_tarball: true -> MUST preserve loose directories
    let args = CompactArgs {
        keep_loose: false,
        no_tarball: true,
        ..Default::default()
    };
    run_archive_compact(&ctx, &args).unwrap();

    assert!(archive_dir.join("2026-06-15-feat-no-tb").exists());
    let milestones_dir = archive_dir.join("milestones");
    assert!(milestones_dir.join("2026-Q2.md").exists());
    assert!(!milestones_dir.join("archive-2026-Q2.tar.gz").exists());
}

#[test]
fn test_run_archive_compact_invalid_before_date_usage_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    let ctx = crate::commands::Context {
        config_dir: tmp.path().join(".ce-ai"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: Some(root.to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let args = CompactArgs {
        before: Some("2026-99-99".to_string()),
        ..Default::default()
    };
    let res = run_archive_compact(&ctx, &args);
    assert!(matches!(res, Err(crate::error::CeError::Usage(_))));
}

#[test]
fn test_multibyte_utf8_resilience() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();

    // Folder with prefix and proposal with multibyte chars: em-dash, accented chars, emojis
    let long_multibyte_proposal = "Este es un problema complejo — con caracteres especiales como ñ, á, é, í, ó, ú, y emojis 🚀🎯 que superan el límite de caracteres para probar truncamiento seguro sin panics.";
    create_sample_package(
        &archive_dir,
        "2026-07-20-feat-ñandú",
        long_multibyte_proposal,
        "Escenario principal",
        1,
        1,
    );

    let ctx = crate::commands::Context {
        config_dir: tmp.path().join(".ce-ai"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: Some(root.to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let args = CompactArgs {
        keep_loose: true,
        ..Default::default()
    };
    run_archive_compact(&ctx, &args).unwrap();

    let milestones_dir = archive_dir.join("milestones");
    assert!(milestones_dir.join("2026-Q3.md").exists());
    assert!(milestones_dir.join("archive-2026-Q3.tar.gz").exists());
}
