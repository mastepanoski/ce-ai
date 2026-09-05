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

#[test]
fn test_mcp_and_skill_detection() {
    let tmp = tempfile::tempdir().unwrap();
    let config_dir = tmp.path().join("ce-ai");
    let opencode_dir = tmp.path().join("opencode");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&opencode_dir).unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: opencode_dir.clone(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    // Initially neither is configured
    assert!(!is_mcp_server_configured(&ctx, "context7"));
    assert!(!is_skill_configured(&ctx, "sequential-thinking"));

    let tool_info = CompanionToolInfo {
        name: "context7".into(),
        label: "Context7".into(),
        category: "MCP Server".into(),
        min_version: "1.0.0".into(),
        latest_version: "1.0.0".into(),
        install_cmd: "ce-ai tools install context7".into(),
    };
    assert_eq!(
        detect_tool_freshness(&ctx, "context7", &tool_info),
        FreshnessStatus::Missing
    );

    // Write opencode.json with mcpServers
    let opencode_json = opencode_dir.join("opencode.json");
    let config = serde_json::json!({
        "mcpServers": {
            "context7": {
                "command": "npx",
                "args": ["-y", "@upstash/context7-mcp@latest"]
            },
            "sequential-thinking": {
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
            }
        }
    });
    std::fs::write(
        &opencode_json,
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // Now both must be detected
    assert!(is_mcp_server_configured(&ctx, "context7"));
    assert!(is_skill_configured(&ctx, "sequential-thinking"));
    // Also test hyphen-stripped lookup
    assert!(is_skill_configured(&ctx, "sequentialthinking"));

    assert_eq!(
        detect_tool_freshness(&ctx, "context7", &tool_info),
        FreshnessStatus::Ok {
            version: "1.0.0".into()
        }
    );
}
