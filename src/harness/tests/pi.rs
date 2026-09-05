use super::*;
use crate::harness::tests::HARNESS_ENV_LOCK;

#[test]
fn pi_adapter_default_paths() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("PI_CODING_AGENT_DIR");

    let adapter = PiAdapter;
    assert_eq!(adapter.kind(), HarnessKind::Pi);
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.kind().harness_dir(&home),
        PathBuf::from("/tmp/home/.pi/agent")
    );
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.pi/agent/skills")
    );
}

#[test]
fn pi_adapter_respects_pi_coding_agent_dir_env() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::set_var("PI_CODING_AGENT_DIR", "/custom/pi/dir");

    let adapter = PiAdapter;
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.kind().harness_dir(&home),
        PathBuf::from("/custom/pi/dir")
    );
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/custom/pi/dir/skills")
    );

    std::env::remove_var("PI_CODING_AGENT_DIR");
}

#[test]
fn pi_session_start_hook_lifecycle() {
    let tmp = tempfile::TempDir::new().unwrap();
    let ext_path = tmp.path().join(".pi/extensions/compound-engineering.ts");

    assert!(!has_session_start_hook(&ext_path));

    // Ensure hook in non-existent directory
    let changed = ensure_session_start_hook(&ext_path).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&ext_path));

    // Verify content
    let content = std::fs::read_to_string(&ext_path).unwrap();
    assert!(content.contains("before_agent_start"));
    assert!(content.contains("session_start"));
    assert!(content.contains("agent_end"));
    assert!(content.contains("session_before_compact"));
    assert!(content.contains(PI_HOOK_VERSION_MARKER));
    assert!(content.contains("ce-ai workflow resume"));

    // Idempotent second call
    let changed_second = ensure_session_start_hook(&ext_path).unwrap();
    assert!(!changed_second);
    assert!(has_session_start_hook(&ext_path));

    // Remove hook
    let removed = remove_session_start_hook(&ext_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&ext_path));
    assert!(!ext_path.exists());
    assert!(
        !tmp.path().join(".pi").exists(),
        "Empty .pi directory should be pruned"
    );

    let removed_second = remove_session_start_hook(&ext_path).unwrap();
    assert!(!removed_second);
}

#[test]
fn pi_session_start_hook_preserves_user_extensions() {
    let tmp = tempfile::TempDir::new().unwrap();
    let ext_dir = tmp.path().join(".pi/extensions");
    std::fs::create_dir_all(&ext_dir).unwrap();

    let user_ext = ext_dir.join("my-custom-tool.ts");
    std::fs::write(&user_ext, "export default function() {}").unwrap();

    let ce_ext = ext_dir.join("compound-engineering.ts");
    let changed = ensure_session_start_hook(&ce_ext).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&ce_ext));

    // Remove our hook
    let removed = remove_session_start_hook(&ce_ext).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&ce_ext));
    assert!(!ce_ext.exists());
    assert!(user_ext.exists(), "User extension must be preserved");
    assert!(
        ext_dir.exists(),
        "Directory with user extension must remain"
    );
}
