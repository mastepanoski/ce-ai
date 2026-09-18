use super::*;
use crate::decisions::mock::MockDecisionProvider;
use crate::decisions::types::DecisionAnswer;
use crate::decisions::DecisionMode;

#[test]
fn test_model_class_parse_and_display() {
    assert_eq!(ModelClass::parse("fast").unwrap(), ModelClass::Fast);
    assert_eq!(ModelClass::parse("Fast").unwrap(), ModelClass::Fast);
    assert_eq!(ModelClass::parse("standard").unwrap(), ModelClass::Standard);
    assert_eq!(
        ModelClass::parse("reasoning").unwrap(),
        ModelClass::Reasoning
    );
    assert!(ModelClass::parse("unknown").is_err());

    assert_eq!(ModelClass::Fast.to_string(), "fast");
    assert_eq!(ModelClass::Standard.to_string(), "standard");
    assert_eq!(ModelClass::Reasoning.to_string(), "reasoning");
}

#[test]
fn test_model_class_catalog_resolve_with_fallback() {
    let mut catalog = ModelClassCatalog::new();
    catalog.set(ModelClass::Fast, "vendor/fast-1");
    catalog.set(ModelClass::Standard, "vendor/standard-1");
    catalog.set(ModelClass::Reasoning, "vendor/reasoning-1");

    // Exact matches
    assert_eq!(
        catalog.resolve_with_fallback(ModelClass::Fast, "default/fallback"),
        ("vendor/fast-1".to_string(), false)
    );
    assert_eq!(
        catalog.resolve_with_fallback(ModelClass::Standard, "default/fallback"),
        ("vendor/standard-1".to_string(), false)
    );
    assert_eq!(
        catalog.resolve_with_fallback(ModelClass::Reasoning, "default/fallback"),
        ("vendor/reasoning-1".to_string(), false)
    );

    // Missing reasoning degrades to standard
    let mut partial = ModelClassCatalog::new();
    partial.set(ModelClass::Fast, "vendor/fast-1");
    partial.set(ModelClass::Standard, "vendor/standard-1");
    assert_eq!(
        partial.resolve_with_fallback(ModelClass::Reasoning, "default/fallback"),
        ("vendor/standard-1".to_string(), true)
    );

    // Missing standard degrades to fast
    let mut fast_only = ModelClassCatalog::new();
    fast_only.set(ModelClass::Fast, "vendor/fast-1");
    assert_eq!(
        fast_only.resolve_with_fallback(ModelClass::Standard, "default/fallback"),
        ("vendor/fast-1".to_string(), true)
    );

    // Missing everything falls back to default_model
    let empty = ModelClassCatalog::new();
    assert_eq!(
        empty.resolve_with_fallback(ModelClass::Fast, "default/fallback"),
        ("default/fallback".to_string(), true)
    );
}

#[test]
fn test_routing_thresholds_defaults_and_serde() {
    let thresholds = RoutingThresholds::default();
    assert_eq!(thresholds.reasoning_threshold_pct, 75);
    assert_eq!(thresholds.standard_threshold_pct, 50);

    let json = serde_json::to_string(&thresholds).unwrap();
    let deserialized: RoutingThresholds = serde_json::from_str(&json).unwrap();
    assert_eq!(thresholds, deserialized);
}

#[test]
fn test_model_routing_config_eq_and_serde() {
    let config = ModelRoutingConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.models.fast, None);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ModelRoutingConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_override_precedence_explicit_override_wins() {
    let mut config = ModelRoutingConfig {
        enabled: true,
        ..Default::default()
    };
    config.models.set(ModelClass::Fast, "model/fast");

    let router = ModelRouter::new(&config, None);
    let res = router.route(
        "fix typo",
        "default/model",
        Some("override/model"),
        Some("slot/model"),
    );

    assert_eq!(res.resolved_model, "override/model");
    assert!(!res.fallback_applied);
    assert_eq!(res.rationale, "Explicit model override applied.");
}

#[test]
fn test_override_precedence_slot_assignment_wins() {
    let mut config = ModelRoutingConfig {
        enabled: true,
        ..Default::default()
    };
    config.models.set(ModelClass::Fast, "model/fast");

    let router = ModelRouter::new(&config, None);
    let res = router.route("fix typo", "default/model", None, Some("slot/model"));

    assert_eq!(res.resolved_model, "slot/model");
    assert!(!res.fallback_applied);
    assert_eq!(res.rationale, "Static agent slot assignment applied.");
}

#[test]
fn test_disabled_routing_falls_back_to_default() {
    let config = ModelRoutingConfig {
        enabled: false,
        ..Default::default()
    };

    let router = ModelRouter::new(&config, None);
    let res = router.route("fix typo", "default/model", None, None);

    assert_eq!(res.resolved_model, "default/model");
    assert!(res.fallback_applied);
    assert!(res.rationale.contains("Adaptive routing is disabled"));
}

#[test]
fn test_routing_classifies_trivial_to_fast() {
    let mut config = ModelRoutingConfig {
        enabled: true,
        ..Default::default()
    };
    config.models.set(ModelClass::Fast, "vendor/fast");
    config.models.set(ModelClass::Standard, "vendor/standard");
    config.models.set(ModelClass::Reasoning, "vendor/reasoning");

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "complexity",
            DecisionAnswer::Choice {
                selected: "trivial".into(),
                confidence: 0.95,
                probabilities: Default::default(),
            },
        )
        .with_canned_answer(
            "risk",
            DecisionAnswer::Choice {
                selected: "low".into(),
                confidence: 0.90,
                probabilities: Default::default(),
            },
        )
        .with_canned_answer(
            "needs_reasoning",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.99,
            },
        )
        .with_canned_answer(
            "needs_large_context",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.90,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = ModelRouter::new(&config, Some(&engine));
    let res = router.route("Update README link", "default/model", None, None);

    assert_eq!(res.recommended_class, ModelClass::Fast);
    assert_eq!(res.resolved_model, "vendor/fast");
    assert!(!res.fallback_applied);
}

#[test]
fn test_routing_classifies_complex_to_reasoning() {
    let mut config = ModelRoutingConfig {
        enabled: true,
        ..Default::default()
    };
    config.models.set(ModelClass::Fast, "vendor/fast");
    config.models.set(ModelClass::Standard, "vendor/standard");
    config.models.set(ModelClass::Reasoning, "vendor/reasoning");

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "complexity",
            DecisionAnswer::Choice {
                selected: "complex".into(),
                confidence: 0.88,
                probabilities: Default::default(),
            },
        )
        .with_canned_answer(
            "risk",
            DecisionAnswer::Choice {
                selected: "high".into(),
                confidence: 0.85,
                probabilities: Default::default(),
            },
        )
        .with_canned_answer(
            "needs_reasoning",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.80,
            },
        )
        .with_canned_answer(
            "needs_large_context",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.70,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = ModelRouter::new(&config, Some(&engine));
    let res = router.route(
        "Refactor multi-threaded consensus engine",
        "default/model",
        None,
        None,
    );

    assert_eq!(res.recommended_class, ModelClass::Reasoning);
    assert_eq!(res.resolved_model, "vendor/reasoning");
    assert!(!res.fallback_applied);
}

#[test]
fn test_routing_classifies_moderate_to_standard() {
    let mut config = ModelRoutingConfig {
        enabled: true,
        ..Default::default()
    };
    config.models.set(ModelClass::Fast, "vendor/fast");
    config.models.set(ModelClass::Standard, "vendor/standard");
    config.models.set(ModelClass::Reasoning, "vendor/reasoning");

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "complexity",
            DecisionAnswer::Choice {
                selected: "moderate".into(),
                confidence: 0.75,
                probabilities: Default::default(),
            },
        )
        .with_canned_answer(
            "risk",
            DecisionAnswer::Choice {
                selected: "medium".into(),
                confidence: 0.60,
                probabilities: Default::default(),
            },
        )
        .with_canned_answer(
            "needs_reasoning",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.50,
            },
        )
        .with_canned_answer(
            "needs_large_context",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.50,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = ModelRouter::new(&config, Some(&engine));
    let res = router.route(
        "Add unit tests for helper function",
        "default/model",
        None,
        None,
    );

    assert_eq!(res.recommended_class, ModelClass::Standard);
    assert_eq!(res.resolved_model, "vendor/standard");
    assert!(!res.fallback_applied);
}

#[test]
fn test_routing_provider_failure_degrades_gracefully() {
    let mut config = ModelRoutingConfig {
        enabled: true,
        ..Default::default()
    };
    config.models.set(ModelClass::Fast, "vendor/fast");

    let mock = MockDecisionProvider::new();
    mock.set_failure(Some("Simulated network timeout".into()));

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let router = ModelRouter::new(&config, Some(&engine));
    let res = router.route("Some task", "default/model", None, None);

    assert_eq!(res.resolved_model, "default/model");
    assert!(res.fallback_applied);
    assert!(res.rationale.contains("Decision provider error"));
}
