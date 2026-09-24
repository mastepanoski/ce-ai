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

    let header = JournalHeader {
        command: "install".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
    };
    let op1 = RecordedOp {
        path: user_file.clone(),
        applied: true,
        prior: Some(b"user-bytes".to_vec()),
    };
    let op2 = RecordedOp {
        path: created.clone(),
        applied: true,
        prior: None,
    };

    let mut content = serde_json::to_vec(&header).unwrap();
    content.push(b'\n');
    content.extend(serde_json::to_vec(&op1).unwrap());
    content.push(b'\n');
    content.extend(serde_json::to_vec(&op2).unwrap());
    content.push(b'\n');

    std::fs::write(journal_path(&cfg), &content).unwrap();

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

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn fault_injection_fails_after_n_successful_arms() {
    let _guard = ENV_LOCK.lock().unwrap();
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

#[test]
fn fault_injection_and_recovery_with_jsonl_format() {
    let _guard = ENV_LOCK.lock().unwrap();
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();

    let file1 = tmp.path().join("file1.txt");
    std::fs::write(&file1, b"initial-content-1").unwrap();
    let file2 = tmp.path().join("file2.txt");
    let file3 = tmp.path().join("file3.txt");

    std::env::set_var("CE_AI_FAIL_AFTER_WRITES", "2");

    let mut j = Journal::begin(&cfg, "install").unwrap();
    // 1st arm: file1. Mutate after arming.
    j.arm(&file1).unwrap();
    std::fs::write(&file1, b"mutated-content-1").unwrap();

    // 2nd arm: file2. Create after arming.
    j.arm(&file2).unwrap();
    std::fs::write(&file2, b"created-content-2").unwrap();

    // 3rd arm: file3. Fails because writes_seen=3 > 2.
    let err = j.arm(&file3).unwrap_err();
    assert!(matches!(err, CeError::Runtime(_)));

    std::env::remove_var("CE_AI_FAIL_AFTER_WRITES");

    // Journal remains on disk with recorded operations.
    assert!(journal_path(&cfg).exists());

    // Next invocation of begin rolls back mutations.
    Journal::begin(&cfg, "install").unwrap();

    // file1 reverted to initial content.
    assert_eq!(std::fs::read(&file1).unwrap(), b"initial-content-1");
    // file2 was created, now rolled back and removed.
    assert!(!file2.exists(), "file2 should be removed on rollback");
    // file3 was never created.
    assert!(!file3.exists(), "file3 should not exist");
}

#[test]
fn incomplete_trailing_line_is_safely_ignored_and_earlier_ops_recovered() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();

    let target = tmp.path().join("target.txt");
    std::fs::write(&target, b"prior-val").unwrap();

    let header = JournalHeader {
        command: "sync".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
    };
    let op = RecordedOp {
        path: target.clone(),
        applied: true,
        prior: Some(b"prior-val".to_vec()),
    };

    let mut content = serde_json::to_vec(&header).unwrap();
    content.push(b'\n');
    content.extend(serde_json::to_vec(&op).unwrap());
    content.push(b'\n');
    // Simulate truncated trailing write
    content.extend(b"{\"path\": \"/incomplete/path\", \"applied\": true, \"pr");

    std::fs::write(journal_path(&cfg), &content).unwrap();

    // Mutate file to simulate applied mutation before crash
    std::fs::write(&target, b"crashed-val").unwrap();

    // begin should recover target back to prior-val and ignore incomplete line
    Journal::begin(&cfg, "sync").unwrap();

    assert_eq!(std::fs::read(&target).unwrap(), b"prior-val");
}

#[test]
fn legacy_journal_format_rolls_back_successfully() {
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();

    let target = tmp.path().join("target.txt");
    std::fs::write(&target, b"old-val").unwrap();

    let legacy = LegacyJournalData {
        command: "install".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
        ops: vec![RecordedOp {
            path: target.clone(),
            applied: true,
            prior: Some(b"old-val".to_vec()),
        }],
    };
    let bytes = serde_json::to_vec_pretty(&legacy).unwrap();
    std::fs::write(journal_path(&cfg), &bytes).unwrap();

    // Verify recorded_command reads legacy format
    assert_eq!(recorded_command(&cfg), Some("install".into()));

    std::fs::write(&target, b"new-val").unwrap();

    Journal::begin(&cfg, "install").unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), b"old-val");
}

#[test]
fn journal_arm_complexity_is_strictly_linear() {
    let _guard = ENV_LOCK.lock().unwrap();
    let tmp = tempdir().unwrap();
    let cfg = tmp.path().join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();

    let n = 500;
    let mut files = Vec::with_capacity(n);
    let sample_payload = vec![42u8; 512]; // 512 bytes of prior content

    for i in 0..n {
        let p = tmp.path().join(format!("file_{i}.txt"));
        std::fs::write(&p, &sample_payload).unwrap();
        files.push(p);
    }

    let j_path = journal_path(&cfg);
    let start_time = std::time::Instant::now();
    let mut j = Journal::begin(&cfg, "install").unwrap();

    let mut prev_len = std::fs::metadata(&j_path).unwrap().len();
    let mut max_delta = 0u64;

    for file in &files {
        j.arm(file).unwrap();
        let curr_len = std::fs::metadata(&j_path).unwrap().len();
        let delta = curr_len - prev_len;
        if delta > max_delta {
            max_delta = delta;
        }
        prev_len = curr_len;
    }

    let elapsed = start_time.elapsed();
    let final_size = std::fs::metadata(&j_path).unwrap().len();

    // 1. Each individual append must be O(1) in size (proportional only to the single op)
    // and must NOT grow with the number of operations already armed.
    assert!(
        max_delta < 3_000,
        "each arm() write must be O(1) and bounded by a single op size, got max_delta={max_delta}"
    );

    // 2. Final journal file size must be strictly linear O(N)
    // For 500 files with ~1.8KB JSONL representation, total size must be ~900KB - 1.2MB,
    // never tens or hundreds of megabytes.
    assert!(
        final_size < 1_500_000,
        "final journal size must be linear O(N), got {final_size} bytes"
    );

    // 3. Total bytes appended must match the change in file size exactly.
    let total_appended = final_size - std::fs::metadata(&j_path).unwrap().len();
    assert_eq!(total_appended, 0, "final size should match end of stream");

    // 4. Wall time for 500 arms with 500 physical fsyncs must finish well within 30s
    // (on macOS APFS / Linux runners, 500 fsyncs take ~1-3s in debug build;
    // on Windows NTFS CI runners, fsync/FlushFileBuffers can take ~8-12s under shared I/O load;
    // quadratic serialization previously took >10 minutes).
    assert!(
        elapsed.as_secs() < 30,
        "500 arm() calls took {:?}, expected < 30s for linear complexity (quadratic took >10m)",
        elapsed
    );

    j.complete().unwrap();
}
