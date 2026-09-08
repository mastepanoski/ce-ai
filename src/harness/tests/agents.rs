use super::*;
use tempfile::TempDir;

fn config_with(body: &str) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("opencode.json");
    std::fs::write(&path, body).unwrap();
    (tmp, path)
}

#[test]
fn creates_structural_agent_without_model_or_variant() {
    let (_tmp, path) = config_with("{}");
    let created = ensure_orchestrator_agent(&path, &HarnessKind::Opencode).unwrap();
    assert!(created);
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let agent = &config["agent"][ORCHESTRATOR_AGENT];
    assert_eq!(agent["mode"], "primary");
    assert_eq!(
        agent["description"],
        "CE AI Orchestrator - coordinates compound engineering"
    );
    assert_eq!(agent["permission"]["question"], "allow");
    assert!(agent.get("model").is_none());
    assert!(agent.get("variant").is_none());
}

#[test]
fn preserves_existing_agent_entry_verbatim() {
    let (_tmp, path) =
        config_with(r#"{"agent":{"ce-ai":{"model":"user/custom-model","description":"mine"}}}"#);
    let created = ensure_orchestrator_agent(&path, &HarnessKind::Opencode).unwrap();
    assert!(!created);
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let agent = &config["agent"][ORCHESTRATOR_AGENT];
    assert_eq!(agent["model"], "user/custom-model");
    assert_eq!(agent["description"], "mine");
    assert_eq!(agent.get("mode"), None);
}

#[test]
fn skips_markdown_based_harnesses() {
    let (_tmp, path) = config_with("# rules");
    let created = ensure_orchestrator_agent(&path, &HarnessKind::Cursor).unwrap();
    assert!(!created);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# rules");
}

#[test]
fn test_mid_tier_slot_constants_and_predicates() {
    assert_eq!(CODE_REVIEW_MID_TIER_SLOT, "ce-code-review-mid-tier");
    assert_eq!(CE_AGENT_STAGE_SLOTS.len(), 6);
    assert_eq!(CE_AGENT_SLOTS.len(), 7);
    assert!(CE_AGENT_SLOTS.contains(&CODE_REVIEW_MID_TIER_SLOT));
    for stage in CE_AGENT_STAGE_SLOTS {
        assert!(CE_AGENT_SLOTS.contains(&stage));
        assert!(is_stage_slot(stage));
        assert!(!is_tier_slot(stage));
    }
    assert!(is_tier_slot(CODE_REVIEW_MID_TIER_SLOT));
    assert!(!is_stage_slot(CODE_REVIEW_MID_TIER_SLOT));
    assert!(!is_tier_slot("definitely-unknown"));
    assert!(!is_stage_slot("definitely-unknown"));
}
