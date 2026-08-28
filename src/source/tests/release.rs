use crate::source::release::{
    auth_header, github_token_from_env, latest_ce_release, pinned_version_and_url, tag_tarball_url,
};

#[test]
fn picks_latest_compound_engineering_release_tag() {
    let payload = br#"[
        {"tag_name": "v2.5.0"},
        {"tag_name": "compound-engineering-v3.10.0"},
        {"tag_name": "compound-engineering-v3.4.2"},
        {"tag_name": "v1.0.0"},
        {"tag_name": "compound-engineering-v2.9.1"}
    ]"#;
    assert_eq!(
        latest_ce_release(payload).unwrap().as_deref(),
        Some("compound-engineering-v3.10.0")
    );
}

#[test]
fn no_matching_release_yields_none() {
    let payload = br#"[{"tag_name": "v2.5.0"}, {"tag_name": "v1.0.0"}]"#;
    assert_eq!(latest_ce_release(payload).unwrap(), None);
}

#[test]
fn pinned_version_resolves_tag_tarball_url() {
    let (version, url) =
        pinned_version_and_url(Some("compound-engineering-v3.4.2".to_string())).unwrap();
    assert_eq!(version, "compound-engineering-v3.4.2");
    assert_eq!(url, tag_tarball_url("compound-engineering-v3.4.2"));
}

#[test]
fn missing_release_is_a_usage_error_never_main_fallback() {
    let err = pinned_version_and_url(None).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("--to <tag>"));
    assert!(!err.to_string().contains("main.tar.gz"));
}

#[test]
fn tag_tarball_url_points_at_release_tag() {
    let url = tag_tarball_url("compound-engineering-v3.4.2");
    assert!(url.ends_with("/archive/refs/tags/compound-engineering-v3.4.2.tar.gz"));
}

#[test]
fn github_token_builds_bearer_header() {
    assert_eq!(
        auth_header(Some("secret")),
        Some("Bearer secret".to_string())
    );
    assert_eq!(auth_header(None), None);
}

#[test]
fn github_token_reads_from_environment() {
    std::env::set_var("CE_AI_GITHUB_TOKEN", "tok-123");
    assert_eq!(github_token_from_env().as_deref(), Some("tok-123"));
    std::env::remove_var("CE_AI_GITHUB_TOKEN");
    assert_eq!(github_token_from_env(), None);
}
