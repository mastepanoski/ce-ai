use super::*;
use tempfile::TempDir;

#[test]
fn test_check_adoption_block_status_file_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::FileMissing
    );
}

#[test]
fn test_check_adoption_block_status_block_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    fs::write(&path, "# Hello World\n").unwrap();
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::BlockMissing
    );
}

#[test]
fn test_check_adoption_block_status_stale_version() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    let content = format!(
        "{} v=1\nOld Content\n{}\n",
        BLOCK_BEGIN_MARKER, BLOCK_END_MARKER
    );
    fs::write(&path, content).unwrap();
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::StaleVersion { version: 1 }
    );
}

#[test]
fn test_reconcile_project_harness_hooks_upgrades_stale_hooks() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path();

    // 1. Setup Claude with only SessionStart
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    let claude_settings = claude_dir.join("settings.json");
    let old_claude = serde_json::json!({
        "hooks": {
            "SessionStart": [
                { "matcher": ".*", "commands": [{ "type": "command", "command": "ce-ai workflow resume" }] }
            ]
        }
    });
    fs::write(
        &claude_settings,
        serde_json::to_string(&old_claude).unwrap(),
    )
    .unwrap();

    // 2. Setup Cursor with only sessionStart
    let cursor_dir = project_dir.join(".cursor");
    fs::create_dir_all(&cursor_dir).unwrap();
    let cursor_hooks = cursor_dir.join("hooks.json");
    let old_cursor = serde_json::json!({
        "version": 1,
        "hooks": {
            "sessionStart": [
                { "command": "ce-ai workflow resume" }
            ]
        }
    });
    fs::write(&cursor_hooks, serde_json::to_string(&old_cursor).unwrap()).unwrap();

    // 3. Setup Pi with v=1 extension
    let pi_dir = project_dir.join(".pi").join("extensions");
    fs::create_dir_all(&pi_dir).unwrap();
    let pi_ext = pi_dir.join("compound-engineering.ts");
    let old_pi = "// ce-ai:hook v=1\nexport default function() {}";
    fs::write(&pi_ext, old_pi).unwrap();

    // Run reconciliation
    reconcile_project_harness_hooks(project_dir, "Test block").unwrap();

    // Verify Claude was upgraded to have Stop and PreCompact
    let claude_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&claude_settings).unwrap()).unwrap();
    assert!(claude_val["hooks"]["Stop"].is_array());
    assert!(claude_val["hooks"]["PreCompact"].is_array());

    // Verify Cursor was upgraded to have stop
    let cursor_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cursor_hooks).unwrap()).unwrap();
    assert!(cursor_val["hooks"]["stop"].is_array());

    // Verify Pi was upgraded to v=2
    let pi_content = fs::read_to_string(&pi_ext).unwrap();
    assert!(pi_content.contains("// ce-ai:hook v=2"));
    assert!(pi_content.contains("agent_end"));
    assert!(pi_content.contains("session_before_compact"));
}
