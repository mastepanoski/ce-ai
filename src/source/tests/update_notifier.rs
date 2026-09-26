use tempfile::tempdir;

use super::*;

#[test]
fn test_cache_write_and_read_roundtrip() {
    let dir = tempdir().unwrap();
    let cache = UpdateCheckCache {
        last_checked_at: "2026-09-26T01:00:00Z".to_string(),
        latest_version: "1.70.0".to_string(),
        latest_tag: "v1.70.0".to_string(),
        release_url: "https://github.com/mastepanoski/ce-ai/releases/tag/v1.70.0".to_string(),
    };

    assert!(read_cache(dir.path()).is_none());
    write_cache(dir.path(), &cache).unwrap();

    let loaded = read_cache(dir.path()).expect("cache should exist");
    assert_eq!(loaded, cache);
}

#[test]
fn test_is_cache_stale() {
    let now = chrono::Utc::now();
    let fresh_time = (now - chrono::Duration::hours(2)).to_rfc3339();
    let stale_time = (now - chrono::Duration::hours(25)).to_rfc3339();

    let fresh_cache = UpdateCheckCache {
        last_checked_at: fresh_time,
        latest_version: "1.70.0".to_string(),
        latest_tag: "v1.70.0".to_string(),
        release_url: "https://example.com".to_string(),
    };
    assert!(!is_cache_stale(&fresh_cache, 24));

    let stale_cache = UpdateCheckCache {
        last_checked_at: stale_time,
        latest_version: "1.70.0".to_string(),
        latest_tag: "v1.70.0".to_string(),
        release_url: "https://example.com".to_string(),
    };
    assert!(is_cache_stale(&stale_cache, 24));

    let malformed_cache = UpdateCheckCache {
        last_checked_at: "not-a-timestamp".to_string(),
        latest_version: "1.70.0".to_string(),
        latest_tag: "v1.70.0".to_string(),
        release_url: "https://example.com".to_string(),
    };
    assert!(is_cache_stale(&malformed_cache, 24));
}

#[test]
fn test_is_newer_version() {
    assert!(is_newer_version("1.70.0", "1.69.1"));
    assert!(is_newer_version("v1.70.0", "1.69.1"));
    assert!(is_newer_version("2.0.0", "1.69.1"));
    assert!(is_newer_version("1.69.2", "1.69.1"));
    assert!(!is_newer_version("1.69.1", "1.69.1"));
    assert!(!is_newer_version("1.69.0", "1.69.1"));
    assert!(!is_newer_version("1.45.0", "1.69.1"));
}

#[test]
fn test_format_update_banner() {
    let banner = format_update_banner("1.69.1", "v1.70.0");
    assert!(banner.contains("Update available: ce-ai v1.69.1 → v1.70.0"));
    assert!(banner.contains("Run 'ce-ai self-update' or 'brew upgrade ce-ai' to upgrade"));
    assert!(banner.starts_with('╭'));
    assert!(banner.ends_with('╯'));
}

#[test]
fn test_suppression_logic() {
    // All clean & enabled -> not suppressed
    assert!(!is_suppressed_internal(false, false, false, true, true));

    // Quiet flag suppresses
    assert!(is_suppressed_internal(true, false, false, true, true));

    // CE_NO_UPDATE_NOTIFIER suppresses
    assert!(is_suppressed_internal(false, true, false, true, true));

    // CI suppresses
    assert!(is_suppressed_internal(false, false, true, true, true));

    // Non-terminal suppresses
    assert!(is_suppressed_internal(false, false, false, false, true));

    // Config disabled suppresses
    assert!(is_suppressed_internal(false, false, false, true, false));
}
