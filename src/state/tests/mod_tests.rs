use super::{report_best_effort_remove, report_best_effort_write};
use tempfile::tempdir;

#[test]
fn remove_reporter_is_silent_on_ok_and_not_found() {
    assert!(!report_best_effort_remove("x", Ok(())));

    let tmp = tempdir().unwrap();
    let absent = tmp.path().join("absent.txt");
    assert!(!report_best_effort_remove(
        &absent,
        std::fs::remove_file(&absent)
    ));
}

#[test]
fn remove_reporter_warns_on_unexpected_io_error() {
    // Removing a non-empty directory fails with DirectoryNotEmpty —
    // an unexpected condition for a best-effort cleanup.
    let dir = tempdir().unwrap();
    let nested = dir.path().join("nested");
    std::fs::create_dir_all(nested.join("child")).unwrap();
    assert!(report_best_effort_remove(
        dir.path().join("nested"),
        std::fs::remove_dir(&nested)
    ));
}

#[test]
fn write_reporter_warns_on_error_and_stays_silent_on_ok() {
    assert!(!report_best_effort_write("x", Ok(())));
    assert!(report_best_effort_write(
        "/unreachable/path.json",
        Err(crate::error::CeError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        )))
    ));
}
