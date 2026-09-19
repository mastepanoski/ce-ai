use super::*;

#[test]
fn test_mask_api_key() {
    assert_eq!(mask_api_key(""), "******");
    assert_eq!(mask_api_key("12345"), "******");
    assert_eq!(mask_api_key("123456"), "******");
    assert_eq!(mask_api_key("ts-1234567890abcdef"), "ts****...****cdef");
    assert_eq!(mask_api_key("  ts-1234567890abcdef  "), "ts****...****cdef");
}

#[test]
fn test_save_and_resolve_api_key_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let creds_path = tmp.path().join("credentials.toml");

    assert_eq!(resolve_api_key(Some(&creds_path)), None);

    let saved_path = save_api_key("ts-secret-key-12345", Some(&creds_path)).expect("save key");
    assert_eq!(saved_path, creds_path);
    assert!(creds_path.exists());

    let resolved = resolve_api_key(Some(&creds_path));
    assert_eq!(resolved.as_deref(), Some("ts-secret-key-12345"));

    // Overwrite with a new key
    save_api_key("ts-new-key-67890", Some(&creds_path)).expect("save key");
    let resolved2 = resolve_api_key(Some(&creds_path));
    assert_eq!(resolved2.as_deref(), Some("ts-new-key-67890"));
}

#[test]
fn test_save_empty_key_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let creds_path = tmp.path().join("credentials.toml");

    let err = save_api_key("   ", Some(&creds_path)).expect_err("empty key should fail");
    assert!(matches!(err, CeError::Usage(_)));
}

#[cfg(unix)]
#[test]
fn test_credentials_file_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().expect("tempdir");
    let creds_path = tmp.path().join("credentials.toml");

    save_api_key("ts-test-perms", Some(&creds_path)).expect("save key");

    let metadata = std::fs::metadata(&creds_path).expect("metadata");
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "credentials.toml must have 0600 permissions");
}

#[test]
fn test_read_api_key_from_reader() {
    // Valid key with trailing newline
    let input = "  ts-custom-key-999 \n";
    let key = read_api_key_from_reader(input.as_bytes()).expect("valid key");
    assert_eq!(key, "ts-custom-key-999");

    // Valid key with Windows CRLF
    let input_crlf = "ts-crlf-key-888\r\n";
    let key_crlf = read_api_key_from_reader(input_crlf.as_bytes()).expect("valid crlf key");
    assert_eq!(key_crlf, "ts-crlf-key-888");

    // Empty input fails with Usage error
    let empty = "";
    let err = read_api_key_from_reader(empty.as_bytes()).expect_err("empty should fail");
    assert!(matches!(err, CeError::Usage(_)));

    // Whitespace-only input fails with Usage error
    let whitespace = "   \t \n";
    let err2 = read_api_key_from_reader(whitespace.as_bytes()).expect_err("whitespace should fail");
    assert!(matches!(err2, CeError::Usage(_)));
}
