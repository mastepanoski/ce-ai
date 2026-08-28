use super::*;
use tempfile::tempdir;

#[test]
fn complete_removes_journal_and_content_survives() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();

    let mut j = Journal::begin(&cfg, "install").unwrap();
    let f = tmp.path().join("f.txt");
    j.arm(&f).unwrap();
    std::fs::write(&f, b"new").unwrap();
    j.complete().unwrap();
    assert!(!journal_path(&cfg).exists());
    assert_eq!(std::fs::read(&f).unwrap(), b"new");
}

#[test]
fn begin_rolls_back_applied_ops_in_reverse() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();

    let user_file = tmp.path().join("user.txt");
    std::fs::write(&user_file, b"user-bytes").unwrap();
    let created = tmp.path().join("created.txt");

    let data = JournalData {
        command: "install".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
        ops: vec![
            RecordedOp {
                path: user_file.clone(),
                applied: true,
                prior: Some(b"user-bytes".to_vec()),
            },
            RecordedOp {
                path: created.clone(),
                applied: true,
                prior: None,
            },
        ],
    };
    write_atomic(
        &journal_path(&cfg),
        &serde_json::to_vec_pretty(&data).unwrap(),
    )
    .unwrap();

    Journal::begin(&cfg, "install").unwrap();

    assert_eq!(std::fs::read(&user_file).unwrap(), b"user-bytes");
    assert!(!created.exists(), "created file rolled back");
    assert!(journal_path(&cfg).exists(), "fresh journal started");
}

#[test]
fn corrupt_journal_is_treated_as_absent() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();
    std::fs::write(journal_path(&cfg), b"{not json").unwrap();

    Journal::begin(&cfg, "install").unwrap(); // must not panic
    assert!(journal_path(&cfg).exists());
}

#[test]
fn fault_injection_fails_after_n_successful_arms() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();
    std::env::set_var("CE_AI_FAIL_AFTER_WRITES", "1");

    let mut j = Journal::begin(&cfg, "install").unwrap();
    let a = tmp.path().join("a.txt");
    j.arm(&a).unwrap();

    let b = tmp.path().join("b.txt");
    let err = j.arm(&b).unwrap_err();
    assert!(matches!(err, CeError::Runtime(_)));

    std::env::remove_var("CE_AI_FAIL_AFTER_WRITES");
}

#[test]
fn recorded_command_reads_command_field() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();
    let j = Journal::begin(&cfg, "sync").unwrap();
    j.complete().unwrap();
    assert_eq!(recorded_command(&cfg), None);
}
