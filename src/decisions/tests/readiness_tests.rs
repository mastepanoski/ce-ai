use super::*;
use crate::decisions::mock::MockDecisionProvider;
use crate::decisions::types::DecisionAnswer;
use crate::decisions::DecisionMode;

#[test]
fn test_readiness_config_defaults_and_serde_and_eq() {
    let config = ReadinessConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.thresholds.ready_pct, 80);
    assert_eq!(config.thresholds.warning_pct, 60);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ReadinessConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_readiness_status_indicators() {
    assert_eq!(ReadinessStatus::Ready.indicator(), "✓ Ready");
    assert_eq!(ReadinessStatus::Warning.indicator(), "△ Needs Attention");
    assert_eq!(ReadinessStatus::NotReady.indicator(), "⚠ Incomplete");
}

#[test]
fn test_readiness_evaluator_disabled_returns_ready_fallback() {
    let config = ReadinessConfig {
        enabled: false,
        ..Default::default()
    };
    let evaluator = ReadinessEvaluator::new(&config, None);
    let result = evaluator.evaluate_odd("test-feat", "Add auth", "- [x] Done", None);
    assert_eq!(result.status, ReadinessStatus::Ready);
    assert_eq!(result.composite_score, 1.0);
    assert!(result.fallback_applied);
    assert_eq!(result.latency_ms, 0);

    let stage_res =
        evaluator.evaluate_stage("test-feat", 4, "WorkTdd", "Implementation active", None);
    assert_eq!(stage_res.status, ReadinessStatus::Ready);
    assert!(stage_res.fallback_applied);
}

#[test]
fn test_readiness_evaluator_odd_ready_status() {
    let config = ReadinessConfig {
        enabled: true,
        thresholds: ReadinessThresholds {
            ready_pct: 80,
            warning_pct: 60,
        },
    };

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "dod_satisfied",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.95,
            },
        )
        .with_canned_answer(
            "guardrails_respected",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.90,
            },
        )
        .with_canned_answer(
            "graduation_recommended",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.85,
            },
        )
        .with_canned_answer(
            "ready_to_close",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.92,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let evaluator = ReadinessEvaluator::new(&config, Some(&engine));
    let result = evaluator.evaluate_odd(
        "auth-token-refresh",
        "Implement token rotation",
        "- [x] rotate tokens\n- [x] unit tests",
        Some("3 files changed, 45 insertions(+)"),
    );

    assert_eq!(result.status, ReadinessStatus::Ready);
    assert!(result.composite_score >= 0.80);
    assert!(!result.graduation_suggested);
    assert!(!result.fallback_applied);
    assert_eq!(result.workflow_mode, "organic");
    assert_eq!(result.dimensions.len(), 4);
}

#[test]
fn test_readiness_evaluator_odd_warning_and_graduation_recommendation() {
    let config = ReadinessConfig {
        enabled: true,
        thresholds: ReadinessThresholds {
            ready_pct: 80,
            warning_pct: 60,
        },
    };

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "dod_satisfied",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.70, // 60%..79% range
            },
        )
        .with_canned_answer(
            "guardrails_respected",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.65,
            },
        )
        .with_canned_answer(
            "graduation_recommended",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.88, // high confidence graduation trigger
            },
        )
        .with_canned_answer(
            "ready_to_close",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.68,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let evaluator = ReadinessEvaluator::new(&config, Some(&engine));
    let result = evaluator.evaluate_odd(
        "large-subsystem-refactor",
        "Refactor auth, database and network layers",
        "- [x] partial refactor",
        Some("28 files changed, 1450 insertions(+)"),
    );

    assert_eq!(result.status, ReadinessStatus::Warning);
    assert!(result.composite_score >= 0.60 && result.composite_score < 0.80);
    assert!(result.graduation_suggested);
    assert!(result
        .advisory_notes
        .iter()
        .any(|n| n.contains("ce-ai graduate")));
}

#[test]
fn test_readiness_evaluator_odd_not_ready_status() {
    let config = ReadinessConfig {
        enabled: true,
        thresholds: ReadinessThresholds {
            ready_pct: 80,
            warning_pct: 60,
        },
    };

    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "dod_satisfied",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.80, // DoD incomplete
            },
        )
        .with_canned_answer(
            "guardrails_respected",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.75, // guardrails breached
            },
        )
        .with_canned_answer(
            "graduation_recommended",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.50,
            },
        )
        .with_canned_answer(
            "ready_to_close",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 0.85,
            },
        );

    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let evaluator = ReadinessEvaluator::new(&config, Some(&engine));
    let result = evaluator.evaluate_odd(
        "unimplemented-feature",
        "Build payment integration",
        "- [ ] stripe integration\n- [ ] webhook handling",
        None,
    );

    assert_eq!(result.status, ReadinessStatus::NotReady);
    assert!(result.composite_score < 0.60);
    assert!(!result.fallback_applied);
}

#[test]
fn test_readiness_evaluator_ce_stage_transition() {
    let config = ReadinessConfig {
        enabled: true,
        thresholds: ReadinessThresholds {
            ready_pct: 80,
            warning_pct: 60,
        },
    };

    let mock_ready = MockDecisionProvider::new()
        .with_canned_answer(
            "requirements_clear",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.95,
            },
        )
        .with_canned_answer(
            "implementation_complete",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.90,
            },
        )
        .with_canned_answer(
            "tests_sufficient",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.88,
            },
        )
        .with_canned_answer(
            "docs_complete",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.85,
            },
        );

    let engine_ready = DecisionEngine::with_mock(mock_ready, DecisionMode::Active);
    let evaluator_ready = ReadinessEvaluator::new(&config, Some(&engine_ready));
    let res = evaluator_ready.evaluate_stage(
        "feat-xyz",
        4,
        "WorkTdd",
        "Implementation and tests completed",
        Some("5 files changed"),
    );

    assert_eq!(res.status, ReadinessStatus::Ready);
    assert!(res.composite_score >= 0.80);
    assert_eq!(res.workflow_mode, "compound");
    assert_eq!(res.dimensions.len(), 4);
    assert!(res.dimensions.iter().all(|d| d.passed));
}

#[test]
fn test_readiness_evaluator_unconfigured_fallback() {
    let config = ReadinessConfig {
        enabled: true,
        ..Default::default()
    };
    // Engine is None
    let evaluator = ReadinessEvaluator::new(&config, None);
    let res = evaluator.evaluate_odd("feat", "task", "- [ ] incomplete", None);
    assert_eq!(res.status, ReadinessStatus::Ready);
    assert!(res.fallback_applied);
    assert_eq!(res.composite_score, 1.0);
}
