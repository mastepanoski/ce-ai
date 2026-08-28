use super::*;

#[test]
fn ok_maps_to_exit_zero() {
    assert_eq!(result_exit_code(&Ok(())), 0);
}

#[test]
fn runtime_errors_map_to_exit_one() {
    assert_eq!(CeError::Runtime("boom".into()).exit_code(), 1);
    let io = std::io::Error::other("disk full");
    assert_eq!(CeError::Io(io).exit_code(), 4);
    assert_eq!(CeError::State("state.json corrupt".into()).exit_code(), 3);
    assert_eq!(CeError::Network("timeout".into()).exit_code(), 5);
    let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    assert_eq!(CeError::Json(json).exit_code(), 1);
}

#[test]
fn usage_error_maps_to_exit_two() {
    assert_eq!(CeError::Usage("missing subcommand".into()).exit_code(), 2);
}

#[test]
fn verification_error_maps_to_exit_six() {
    assert_eq!(
        CeError::Verification("archive hash mismatch".into()).exit_code(),
        6
    );
    assert!(CeError::Verification("archive hash mismatch".into())
        .to_string()
        .contains("verification error: archive hash mismatch"));
}

#[test]
fn display_includes_error_context() {
    assert!(CeError::Usage("missing subcommand".into())
        .to_string()
        .contains("missing subcommand"));
    assert!(CeError::Runtime("no state file".into())
        .to_string()
        .contains("no state file"));
}
