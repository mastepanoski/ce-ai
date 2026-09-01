use crate::source::release::{
    auth_header, extract_latest_tag_from_atom_feed, extract_tag_from_redirect_url,
    github_token_from_env, latest_ce_release, pinned_version_and_url, tag_tarball_url,
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

    std::env::set_var("GITHUB_TOKEN", "tok-456");
    assert_eq!(github_token_from_env().as_deref(), Some("tok-456"));
    std::env::remove_var("GITHUB_TOKEN");

    std::env::set_var("GH_TOKEN", "tok-789");
    assert_eq!(github_token_from_env().as_deref(), Some("tok-789"));
    std::env::remove_var("GH_TOKEN");

    assert_eq!(github_token_from_env(), None);
}

#[test]
fn extracts_tag_from_redirect_url() {
    assert_eq!(
        extract_tag_from_redirect_url(
            "https://github.com/everyinc/compound-engineering-plugin/releases/tag/compound-engineering-v3.24.0"
        ),
        Some("compound-engineering-v3.24.0".to_string())
    );

    assert_eq!(
        extract_tag_from_redirect_url(
            "https://github.com/everyinc/compound-engineering-plugin/releases/tag/compound-engineering-v3.24.0/"
        ),
        Some("compound-engineering-v3.24.0".to_string())
    );

    // Non-matching prefix
    assert_eq!(
        extract_tag_from_redirect_url(
            "https://github.com/everyinc/compound-engineering-plugin/releases/tag/v1.0.0"
        ),
        None
    );

    // Non-tag URL
    assert_eq!(
        extract_tag_from_redirect_url(
            "https://github.com/everyinc/compound-engineering-plugin/releases"
        ),
        None
    );
}

#[test]
fn extracts_latest_tag_from_atom_feed() {
    let feed = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <id>tag:github.com,2008:Repository/1073224021/compound-engineering-v3.23.4</id>
    <link rel="alternate" type="text/html" href="https://github.com/EveryInc/compound-engineering-plugin/releases/tag/compound-engineering-v3.23.4"/>
    <title>compound-engineering: v3.23.4</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/1073224021/compound-engineering-v3.24.0</id>
    <link rel="alternate" type="text/html" href="https://github.com/EveryInc/compound-engineering-plugin/releases/tag/compound-engineering-v3.24.0"/>
    <title>compound-engineering: v3.24.0</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/1073224021/v1.0.0</id>
    <link rel="alternate" type="text/html" href="https://github.com/EveryInc/compound-engineering-plugin/releases/tag/v1.0.0"/>
    <title>v1.0.0</title>
  </entry>
</feed>"#;

    assert_eq!(
        extract_latest_tag_from_atom_feed(feed),
        Some("compound-engineering-v3.24.0".to_string())
    );
}
