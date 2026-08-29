use super::*;
use tempfile::TempDir;

#[test]
fn test_check_adoption_block_status_file_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::FileMissing
    );
}

#[test]
fn test_check_adoption_block_status_block_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    fs::write(&path, "# Hello World\n").unwrap();
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::BlockMissing
    );
}

#[test]
fn test_check_adoption_block_status_stale_version() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    let content = format!(
        "{} v=1\nOld Content\n{}\n",
        BLOCK_BEGIN_MARKER, BLOCK_END_MARKER
    );
    fs::write(&path, content).unwrap();
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::StaleVersion { version: 1 }
    );
}
