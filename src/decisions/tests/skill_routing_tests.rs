use super::*;
use crate::decisions::mock::MockDecisionProvider;
use crate::decisions::types::DecisionAnswer;
use crate::decisions::DecisionMode;
use sha2::Digest;
use std::fs;

#[test]
fn test_skill_routing_config_defaults_and_serde() {
    let config = SkillRoutingConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.minimum_confidence_pct, 70);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: SkillRoutingConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_skill_router_disabled_returns_fallback() {
    let config = SkillRoutingConfig {
        enabled: false,
        ..Default::default()
    };
    let router = SkillRouter::new(&config, None);
    let (cats, classifications, fallback, rationale, latency) =
        router.classify_intent("Audit cookies for security");

    assert!(cats.is_empty());
    assert!(classifications.is_empty());
    assert!(fallback);
    assert!(rationale.contains("disabled"));
    assert_eq!(latency, 0);
}

#[test]
fn test_skill_router_classifies_intent_with_threshold() {
    let config = SkillRoutingConfig {
        enabled: true,
        minimum_confidence_pct: 70,
    };

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "security",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.95,
            },
        )
        .with_canned_answer(
            "testing",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.80,
            },
        )
        .with_canned_answer(
            "architecture",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.50, // below 70% threshold
            },
        )
        .with_canned_answer(
            "debugging",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.90,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = SkillRouter::new(&config, Some(&engine));
    let (cats, classifications, fallback, rationale, _latency) =
        router.classify_intent("Audit authentication cookies and generate unit tests");

    assert!(!fallback);
    assert_eq!(cats, vec!["security", "testing"]);
    assert!(rationale.contains("security"));
    assert!(rationale.contains("testing"));

    let sec = classifications
        .iter()
        .find(|c| c.category == "security")
        .unwrap();
    assert!(sec.selected);
    assert_eq!(sec.confidence, 0.95);

    let test = classifications
        .iter()
        .find(|c| c.category == "testing")
        .unwrap();
    assert!(test.selected);
    assert_eq!(test.confidence, 0.80);

    let arch = classifications
        .iter()
        .find(|c| c.category == "architecture")
        .unwrap();
    assert!(!arch.selected);
    assert_eq!(arch.confidence, 0.50);
}

#[test]
fn test_skill_router_provider_error_falls_back() {
    let config = SkillRoutingConfig {
        enabled: true,
        ..Default::default()
    };
    let mock = MockDecisionProvider::new();
    mock.set_failure(Some("Connection timed out".into()));

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = SkillRouter::new(&config, Some(&engine));
    let (cats, _, fallback, rationale, _) = router.classify_intent("Review code");

    assert!(cats.is_empty());
    assert!(fallback);
    assert!(rationale.contains("Decision provider error"));
}

#[test]
fn test_skill_router_route_resolves_skills_end_to_end() {
    let temp = tempfile::tempdir().unwrap();
    let sec_file = temp.path().join("sec_SKILL.md");
    let test_file = temp.path().join("test_SKILL.md");

    fs::write(&sec_file, "security skill content").unwrap();
    fs::write(&test_file, "testing skill content").unwrap();

    let mut hasher = sha2::Sha256::new();
    hasher.update(b"security skill content");
    let sec_sha = format!("{:x}", hasher.finalize());

    let mut hasher = sha2::Sha256::new();
    hasher.update(b"testing skill content");
    let test_sha = format!("{:x}", hasher.finalize());

    let sec_entry = SkillEntry {
        name: "security-review".into(),
        description: "Security auditing and cookie review".into(),
        scope: "global".into(),
        triggers: vec!["security".into()],
        categories: vec!["security".into()],
        sha256: sec_sha,
        harness_paths: std::collections::BTreeMap::from([(
            "opencode".into(),
            sec_file.to_string_lossy().into(),
        )]),
    };

    let test_entry = SkillEntry {
        name: "testing-qa".into(),
        description: "Writing and running tests".into(),
        scope: "global".into(),
        triggers: vec!["test".into()],
        categories: vec!["testing".into()],
        sha256: test_sha,
        harness_paths: std::collections::BTreeMap::from([(
            "opencode".into(),
            test_file.to_string_lossy().into(),
        )]),
    };

    let registry = SkillRegistry {
        skills: vec![sec_entry, test_entry],
        ..Default::default()
    };

    let config = SkillRoutingConfig {
        enabled: true,
        minimum_confidence_pct: 70,
    };

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "security",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.92,
            },
        )
        .with_canned_answer(
            "testing",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.85,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = SkillRouter::new(&config, Some(&engine));

    let (result, status, md) = router.route(
        &registry,
        HarnessKind::Opencode,
        "Audit JWT tokens and write unit tests",
    );

    assert_eq!(status, "paths-injected");
    assert!(!result.fallback_applied);
    assert_eq!(result.candidate_categories, vec!["security", "testing"]);
    assert_eq!(result.resolved_skills.len(), 2);
    assert!(md.contains("security-review"));
    assert!(md.contains("testing-qa"));
}

#[test]
fn test_skill_router_route_below_threshold_falls_back_to_keyword() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("SKILL.md");
    fs::write(&file, "content").unwrap();

    let mut hasher = sha2::Sha256::new();
    hasher.update(b"content");
    let sha = format!("{:x}", hasher.finalize());

    let entry = SkillEntry {
        name: "my-tool".into(),
        description: "A tool".into(),
        scope: "global".into(),
        triggers: vec!["analyze".into()],
        categories: vec!["research".into()],
        sha256: sha,
        harness_paths: std::collections::BTreeMap::from([(
            "opencode".into(),
            file.to_string_lossy().into(),
        )]),
    };

    let registry = SkillRegistry {
        skills: vec![entry],
        ..Default::default()
    };

    let config = SkillRoutingConfig {
        enabled: true,
        minimum_confidence_pct: 80,
    };

    // Research confidence is 0.60, below 80% threshold
    let mock = MockDecisionProvider::new().with_canned_answer(
        "research",
        DecisionAnswer::Boolean {
            value: true,
            confidence: 0.60,
        },
    );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = SkillRouter::new(&config, Some(&engine));

    // Keyword "analyze" matches trigger
    let (result, status, _md) =
        router.route(&registry, HarnessKind::Opencode, "Please analyze the code");

    assert!(result.candidate_categories.is_empty());
    assert_eq!(status, "paths-injected");
    assert_eq!(result.resolved_skills.len(), 1);
    assert_eq!(result.resolved_skills[0].name, "my-tool");
}

#[test]
fn test_skill_router_route_tampered_skill_degrades() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("SKILL.md");
    fs::write(&file, "original content").unwrap();

    let entry = SkillEntry {
        name: "tampered-skill".into(),
        description: "Security auditing".into(),
        scope: "global".into(),
        triggers: vec!["security".into()],
        categories: vec!["security".into()],
        sha256: "mismatched_sha256_hash".into(),
        harness_paths: std::collections::BTreeMap::from([(
            "opencode".into(),
            file.to_string_lossy().into(),
        )]),
    };

    let registry = SkillRegistry {
        skills: vec![entry],
        ..Default::default()
    };

    let config = SkillRoutingConfig {
        enabled: true,
        minimum_confidence_pct: 70,
    };

    let mock = MockDecisionProvider::new().with_canned_answer(
        "security",
        DecisionAnswer::Boolean {
            value: true,
            confidence: 0.95,
        },
    );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = SkillRouter::new(&config, Some(&engine));

    let (result, status, md) = router.route(&registry, HarnessKind::Opencode, "Audit security");

    assert_eq!(status, "fallback-fuzzy");
    assert!(result.resolved_skills.is_empty());
    assert!(md.contains("status=fallback-fuzzy"));
}
