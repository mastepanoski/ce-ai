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
