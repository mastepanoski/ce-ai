//! GitHub releases resolution (RED — tests only; implementation lands in
//! task 3.4).
//!
//! SF-1: resolve the latest CE GitHub release (tags `compound-engineering-v*`).
//! SF-2: fall back to the `main` branch tarball when release metadata is
//! missing. Optional `CE_AI_GITHUB_TOKEN` passthrough. Unit tests never touch
//! the network — parsing is exercised on fixture payloads.

#[cfg(test)]
mod tests {
    use crate::source::release::{auth_header, github_token_from_env, latest_ce_release, main_tarball_url};

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
    fn main_tarball_fallback_url_points_at_main_branch() {
        let url = main_tarball_url();
        assert!(url.ends_with("/everyinc/compound-engineering-plugin/archive/refs/heads/main.tar.gz"));
    }

    #[test]
    fn github_token_builds_bearer_header() {
        assert_eq!(auth_header(Some("secret")), Some("Bearer secret".to_string()));
        assert_eq!(auth_header(None), None);
    }

    #[test]
    fn github_token_reads_from_environment() {
        std::env::set_var("CE_AI_GITHUB_TOKEN", "tok-123");
        assert_eq!(github_token_from_env().as_deref(), Some("tok-123"));
        std::env::remove_var("CE_AI_GITHUB_TOKEN");
        assert_eq!(github_token_from_env(), None);
    }
}