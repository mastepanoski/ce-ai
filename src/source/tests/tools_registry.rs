use super::*;

#[test]
fn test_version_parsing() {
    assert_eq!(
        parse_version_token("engram 1.2.0"),
        Some("1.2.0".to_string())
    );
    assert_eq!(
        parse_version_token("codegraph v0.5.0"),
        Some("0.5.0".to_string())
    );
    assert_eq!(parse_version_token("no-version-here"), None);
}

#[test]
fn test_version_comparison() {
    assert!(is_version_at_least("1.2.0", "1.2.0"));
    assert!(is_version_at_least("1.3.0", "1.2.0"));
    assert!(!is_version_at_least("1.1.9", "1.2.0"));
}

#[test]
fn test_evaluate_freshness() {
    assert_eq!(
        evaluate_freshness(Some("1.2.0"), "1.2.0"),
        FreshnessStatus::Ok {
            version: "1.2.0".into()
        }
    );
    assert_eq!(
        evaluate_freshness(Some("1.0.0"), "1.2.0"),
        FreshnessStatus::Outdated {
            current: "1.0.0".into(),
            expected: "1.2.0".into()
        }
    );
    assert_eq!(evaluate_freshness(None, "1.2.0"), FreshnessStatus::Missing);
}
