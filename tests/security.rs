//! Security Threat Matrix Audit & Verification Test Suite (ISO 27001 / ISO 27002 / NIST SP 800-53 CP-9/10).

use ce_ai::error::CeError;
use ce_ai::source::archive::extract_safe;
use ce_ai::state::state::State;
use ce_ai::state::write_atomic;
use tempfile::TempDir;

fn append_raw_entry(builder: &mut tar::Builder<Vec<u8>>, raw_path: &[u8], data: &[u8]) {
    let mut header = tar::Header::new_gnu();
    header.set_mode(0o644);
    header.set_size(data.len() as u64);
    let name = &mut header.as_old_mut().name;
    name[..raw_path.len()].copy_from_slice(raw_path);
    header.set_cksum();
    builder.append(&header, data).unwrap();
}

#[test]
fn path_traversal_relative_parent_rejected() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("extracted");

    let mut builder = tar::Builder::new(Vec::new());
    append_raw_entry(&mut builder, b"../pwned.txt", b"malicious payload");
    builder.finish().unwrap();
    let tar_bytes = builder.into_inner().unwrap();

    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    use std::io::Write;
    encoder.write_all(&tar_bytes).unwrap();
    let gz_bytes = encoder.finish().unwrap();

    let archive_path = tmp.path().join("malicious.tar.gz");
    std::fs::write(&archive_path, &gz_bytes).unwrap();

    let result = extract_safe(&archive_path, &target);
    assert!(
        result.is_err(),
        "Path traversal relative parent entry must be rejected before extraction"
    );
}

#[test]
fn path_traversal_absolute_path_rejected() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("extracted");

    let mut builder = tar::Builder::new(Vec::new());
    append_raw_entry(&mut builder, b"/etc/passwd", b"malicious payload");
    builder.finish().unwrap();
    let tar_bytes = builder.into_inner().unwrap();

    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    use std::io::Write;
    encoder.write_all(&tar_bytes).unwrap();
    let gz_bytes = encoder.finish().unwrap();

    let archive_path = tmp.path().join("malicious_abs.tar.gz");
    std::fs::write(&archive_path, &gz_bytes).unwrap();

    let result = extract_safe(&archive_path, &target);
    assert!(
        result.is_err(),
        "Absolute path entry must be rejected before extraction"
    );
}

#[test]
fn atomic_write_guarantees_file_integrity_and_zero_residual_tempfiles() {
    let tmp = TempDir::new().unwrap();
    let target_file = tmp.path().join("state.json");

    let content = r#"{"installed_harnesses":[],"model_assignments":{}}"#;
    write_atomic(&target_file, content.as_bytes()).expect("Atomic write should succeed");

    assert!(target_file.exists(), "Target file must exist");
    let read_back = std::fs::read_to_string(&target_file).unwrap();
    assert_eq!(read_back, content);

    // Verify zero residual tempfiles (.tmp-*) remain in target directory
    let entries = std::fs::read_dir(tmp.path()).unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(
            !name.starts_with(".tmp-"),
            "Zero temporary files should remain: found {name}"
        );
    }
}

#[test]
fn corrupted_json_state_returns_state_error() {
    let tmp = TempDir::new().unwrap();
    let state_file = tmp.path().join("state.json");
    std::fs::write(&state_file, r#"{"invalid": json syntax"#).unwrap();

    let result = State::load(&state_file);
    assert!(
        result.is_err(),
        "Corrupted state.json must return error instead of panicking"
    );
    match result.unwrap_err() {
        CeError::State(_) => (),
        err => panic!("Expected CeError::State, got {:?}", err),
    }
}
