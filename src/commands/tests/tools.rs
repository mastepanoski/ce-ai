use super::*;
use tempfile::TempDir;

#[test]
fn test_tools_status_runs_without_panic() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    assert!(status(&ctx).is_ok());
}

#[test]
fn test_tools_install_registers_mcp_server_atomically_without_clobbering() {
    let tmp = TempDir::new().unwrap();
    let opencode_dir = tmp.path().join("opencode");
    std::fs::create_dir_all(&opencode_dir).unwrap();
    let config_file = opencode_dir.join("opencode.json");

    // Write pre-existing user config
    std::fs::write(
        &config_file,
        serde_json::to_vec_pretty(&serde_json::json!({
            "mcpServers": {
                "custom-user-mcp": { "command": "my-mcp" }
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: opencode_dir.clone(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let result = install_tool(&ctx, "context7");
    assert!(result.is_ok());

    // Verify config was updated atomically preserving user mcp
    let val: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&config_file).unwrap()).unwrap();
    let mcp = val.get("mcpServers").unwrap().as_object().unwrap();
    assert!(mcp.contains_key("custom-user-mcp"));
    assert!(mcp.contains_key("context7"));
}

#[test]
fn test_tools_install_unknown_tool_fails_usage() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let err = install_tool(&ctx, "invalid-tool").unwrap_err();
    assert!(matches!(err, CeError::Usage(_)));
}

#[test]
fn test_tools_install_dry_run_makes_no_changes() {
    let tmp = TempDir::new().unwrap();
    let opencode_dir = tmp.path().join("opencode");
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: opencode_dir.clone(),
        workspace_root: None,
        dry_run: true,
        verbose: false,
        quiet: true,
    };

    let result = install_tool(&ctx, "engram");
    assert!(result.is_ok());
    assert!(!opencode_dir.join("opencode.json").exists());
}

#[test]
fn test_tools_init_unsupported_tool_fails_usage() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let err = init_tool(&ctx, "unsupported-tool", Some(tmp.path())).unwrap_err();
    assert!(matches!(err, CeError::Usage(_)));
}

#[test]
fn test_tools_init_codegraph_when_already_initialized() {
    let tmp = TempDir::new().unwrap();
    let codegraph_dir = tmp.path().join(".codegraph");
    std::fs::create_dir_all(&codegraph_dir).unwrap();

    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let result = init_tool(&ctx, "codegraph", Some(tmp.path()));
    assert!(result.is_ok());
}

#[test]
fn test_tools_init_codegraph_dry_run_does_not_create_index() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: true,
        verbose: false,
        quiet: true,
    };

    let result = init_tool(&ctx, "codegraph", Some(tmp.path()));
    assert!(result.is_ok());
    assert!(!tmp.path().join(".codegraph").exists());
}
