use super::*;
use tempfile::tempdir;

#[test]
fn test_decision_type_parse_and_display() {
    assert_eq!(
        DecisionType::parse("model_routing").unwrap(),
        DecisionType::ModelRouting
    );
    assert_eq!(
        DecisionType::parse("routing").unwrap(),
        DecisionType::ModelRouting
    );
    assert_eq!(
        DecisionType::parse("model").unwrap(),
        DecisionType::ModelRouting
    );
    assert_eq!(
        DecisionType::parse("skill_routing").unwrap(),
        DecisionType::SkillRouting
    );
    assert_eq!(
        DecisionType::parse("skills").unwrap(),
        DecisionType::SkillRouting
    );
    assert_eq!(
        DecisionType::parse("risk_classification").unwrap(),
        DecisionType::RiskClassification
    );
    assert_eq!(
        DecisionType::parse("risk").unwrap(),
        DecisionType::RiskClassification
    );
    assert_eq!(
        DecisionType::parse("stage_readiness").unwrap(),
        DecisionType::StageReadiness
    );
    assert_eq!(
        DecisionType::parse("readiness").unwrap(),
        DecisionType::StageReadiness
    );

    assert!(DecisionType::parse("invalid_type").is_err());

    assert_eq!(DecisionType::ModelRouting.as_str(), "model_routing");
    assert_eq!(DecisionType::ModelRouting.to_string(), "model_routing");
}

#[test]
fn test_decision_event_builder_and_serde() {
    let event = DecisionEvent::new(
        DecisionType::ModelRouting,
        "jev",
        "typesafe-classifier-v1",
        45,
        "standard",
    )
    .with_workflow(Some("feature-auth"))
    .with_stage(Some("work"))
    .with_confidence(Some(0.92))
    .with_fallback(false)
    .with_shadow(true)
    .with_estimated_cost(Some(0.0001))
    .with_metadata("complexity", "moderate");

    let json = serde_json::to_string(&event).expect("serialize");
    let parsed: DecisionEvent = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.id, event.id);
    assert_eq!(parsed.workflow_id.as_deref(), Some("feature-auth"));
    assert_eq!(parsed.stage.as_deref(), Some("work"));
    assert_eq!(parsed.decision_type, DecisionType::ModelRouting);
    assert_eq!(parsed.outcome, "standard");
    assert_eq!(parsed.confidence, Some(0.92));
    assert!(parsed.shadow_mode);
    assert!(!parsed.fallback_used);
    assert_eq!(
        parsed.metadata.get("complexity").map(|s| s.as_str()),
        Some("moderate")
    );
}

#[test]
fn test_sanitize_task_summary() {
    assert_eq!(
        sanitize_task_summary("Implement OAuth2 authentication flow\nwith PKCE support"),
        "Implement OAuth2 authentication flow"
    );
    let long_prompt = "A".repeat(100);
    let sanitized = sanitize_task_summary(&long_prompt);
    assert_eq!(sanitized.len(), 60);
    assert!(sanitized.ends_with("..."));

    assert_eq!(sanitize_task_summary(""), "unspecified_task");
}

#[test]
fn test_ledger_atomic_append_and_read() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path();

    // Empty ledger read
    let initial = read_decision_events(config_dir).unwrap();
    assert!(initial.is_empty());

    // Write two events
    let e1 = DecisionEvent::new(
        DecisionType::ModelRouting,
        "jev",
        "typesafe-classifier-v1",
        30,
        "fast",
    )
    .with_workflow(Some("wf-1"));

    let e2 = DecisionEvent::new(
        DecisionType::RiskClassification,
        "mock",
        "mock-rules",
        15,
        "allow",
    )
    .with_workflow(Some("wf-1"));

    log_decision_event(config_dir, &e1).unwrap();
    log_decision_event(config_dir, &e2).unwrap();

    let records = read_decision_events(config_dir).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id, e1.id);
    assert_eq!(records[0].outcome, "fast");
    assert_eq!(records[1].id, e2.id);
    assert_eq!(records[1].outcome, "allow");

    // Manually inject a blank line and a corrupt line into decisions.jsonl
    let ledger_path = decision_ledger_path(config_dir);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&ledger_path)
        .unwrap();
    writeln!(file).unwrap();
    writeln!(file, "{{corrupted json").unwrap();

    let recovered = read_decision_events(config_dir).unwrap();
    assert_eq!(recovered.len(), 2);
}

#[test]
fn test_analytics_filter_and_stats() {
    let e1 = DecisionEvent::new(DecisionType::ModelRouting, "jev", "v1", 20, "fast")
        .with_workflow(Some("wf-1"))
        .with_confidence(Some(0.95));
    let e2 = DecisionEvent::new(DecisionType::ModelRouting, "jev", "v1", 40, "standard")
        .with_workflow(Some("wf-1"))
        .with_confidence(Some(0.85));
    let e3 = DecisionEvent::new(DecisionType::ModelRouting, "jev", "v1", 100, "reasoning")
        .with_workflow(Some("wf-2"))
        .with_confidence(Some(0.65))
        .with_fallback(true);
    let e4 = DecisionEvent::new(DecisionType::RiskClassification, "jev", "v1", 10, "allow")
        .with_workflow(Some("wf-1"))
        .with_confidence(Some(0.99));

    let analytics = DecisionAnalytics::new(vec![e1, e2, e3, e4]);

    // Filter by workflow
    let wf1_events = analytics.filter(Some("wf-1"), None);
    assert_eq!(wf1_events.len(), 3);

    // Filter by type
    let routing_events = analytics.filter(None, Some(DecisionType::ModelRouting));
    assert_eq!(routing_events.len(), 3);

    // Filter by workflow + type
    let wf1_routing = analytics.filter(Some("wf-1"), Some(DecisionType::ModelRouting));
    assert_eq!(wf1_routing.len(), 2);

    // Compute stats
    let stats = analytics.compute_stats(&routing_events);
    assert_eq!(stats.total_runs, 3);
    assert_eq!(stats.fallback_count, 1);
    assert!((stats.fallback_rate_pct - 33.33).abs() < 0.1);
    assert_eq!(stats.latency.min_ms, 20);
    assert_eq!(stats.latency.median_ms, 40);
    assert_eq!(stats.latency.max_ms, 100);
    assert_eq!(stats.confidence.low_confidence_count, 1); // 0.65 is < 0.70
    assert_eq!(stats.outcome_distribution.get("fast"), Some(&1));
    assert_eq!(stats.outcome_distribution.get("standard"), Some(&1));
    assert_eq!(stats.outcome_distribution.get("reasoning"), Some(&1));
}

#[test]
fn test_routing_comparison_benchmark_and_observed() {
    let e1 = DecisionEvent::new(DecisionType::ModelRouting, "jev", "v1", 30, "fast");
    let e2 = DecisionEvent::new(DecisionType::ModelRouting, "jev", "v1", 35, "fast");
    let e3 = DecisionEvent::new(DecisionType::ModelRouting, "jev", "v1", 40, "standard");
    let e4 = DecisionEvent::new(DecisionType::ModelRouting, "jev", "v1", 45, "reasoning");

    let analytics = DecisionAnalytics::new(vec![e1, e2, e3, e4]);
    let events = analytics.filter(None, Some(DecisionType::ModelRouting));

    // 1. Benchmark comparison (empty usage records)
    let comparison = analytics.compare_routing(&events, &[]);
    assert_eq!(comparison.total_runs, 4);
    assert_eq!(comparison.fast_pct, 50.0);
    assert_eq!(comparison.standard_pct, 25.0);
    assert_eq!(comparison.reasoning_pct, 25.0);
    assert_eq!(comparison.fallback_pct, 0.0);
    assert!(!comparison.static_routing_cost.is_observed);
    assert!(!comparison.adaptive_routing_cost.is_observed);
    // Static = 4 * 0.12 = 0.48
    assert!((comparison.static_routing_cost.amount_usd - 0.48).abs() < 0.001);
    // Adaptive = 2 * 0.015 + 1 * 0.12 + 1 * 0.60 = 0.03 + 0.12 + 0.60 = 0.75
    assert!((comparison.adaptive_routing_cost.amount_usd - 0.75).abs() < 0.001);

    // 2. Observed comparison with usage records
    let usage = vec![crate::capture::ledger::UsageRecord {
        author: "alice".to_string(),
        timestamp: "2026-09-22T10:00:00Z".to_string(),
        harness: "opencode".to_string(),
        session_id: "s-1".to_string(),
        cwd_basename: "repo".to_string(),
        model: "claude-3-7-sonnet".to_string(),
        input_tokens: 500_000,
        output_tokens: 500_000,
        cache_read: 0,
        cache_write: 0,
        reasoning_tokens: 0,
    }];
    let observed_comp = analytics.compare_routing(&events, &usage);
    assert!(observed_comp.static_routing_cost.is_observed);
    assert!(observed_comp.adaptive_routing_cost.is_observed);
    assert_eq!(observed_comp.static_routing_cost.label(), "[observed]");
    assert_eq!(comparison.static_routing_cost.label(), "[estimated]");
}

#[test]
fn test_model_router_telemetry_and_shadow_mode() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path();

    let mock = crate::decisions::mock::MockDecisionProvider::new()
        .with_canned_answer(
            "complexity",
            crate::decisions::types::DecisionAnswer::Choice {
                selected: "trivial".into(),
                confidence: 0.95,
                probabilities: Default::default(),
            },
        )
        .with_canned_answer(
            "needs_reasoning",
            crate::decisions::types::DecisionAnswer::Boolean {
                value: false,
                confidence: 0.99,
            },
        )
        .with_canned_answer(
            "needs_large_context",
            crate::decisions::types::DecisionAnswer::Boolean {
                value: false,
                confidence: 0.99,
            },
        )
        .with_canned_answer(
            "risk",
            crate::decisions::types::DecisionAnswer::Choice {
                selected: "low".into(),
                confidence: 0.95,
                probabilities: Default::default(),
            },
        );

    let config = crate::decisions::ModelRoutingConfig {
        enabled: true,
        models: crate::decisions::ModelClassCatalog {
            fast: Some("provider/fast-model".into()),
            standard: Some("provider/standard-model".into()),
            reasoning: Some("provider/reasoning-model".into()),
        },
        thresholds: crate::decisions::RoutingThresholds::default(),
    };

    // 1. Active mode: routes to fast model and logs telemetry
    let active_engine = crate::decisions::DecisionEngine::with_mock(
        mock.clone(),
        crate::decisions::DecisionMode::Active,
    );
    let active_router = crate::decisions::ModelRouter::new(&config, Some(&active_engine))
        .with_config_dir(config_dir)
        .with_workflow(Some("feature-test"))
        .with_stage(Some("work"));

    let res = active_router.route("Simple task", "default/model", None, None);
    assert_eq!(res.recommended_class, crate::decisions::ModelClass::Fast);
    assert_eq!(res.resolved_model, "provider/fast-model");

    let events = read_decision_events(config_dir).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].decision_type, DecisionType::ModelRouting);
    assert_eq!(events[0].outcome, "fast");
    assert!(!events[0].shadow_mode);
    assert_eq!(events[0].workflow_id.as_deref(), Some("feature-test"));

    // 2. Shadow mode: suggests fast model, but default model remains authoritative
    let shadow_engine =
        crate::decisions::DecisionEngine::with_mock(mock, crate::decisions::DecisionMode::Shadow);
    let shadow_router = crate::decisions::ModelRouter::new(&config, Some(&shadow_engine))
        .with_config_dir(config_dir)
        .with_workflow(Some("feature-test"))
        .with_stage(Some("work"));

    let res_shadow = shadow_router.route("Simple task", "default/model", None, None);
    assert_eq!(
        res_shadow.recommended_class,
        crate::decisions::ModelClass::Fast
    );
    assert_eq!(res_shadow.resolved_model, "default/model");
    assert!(res_shadow.rationale.contains("Shadow mode active"));

    let events2 = read_decision_events(config_dir).unwrap();
    assert_eq!(events2.len(), 2);
    assert_eq!(events2[1].decision_type, DecisionType::ModelRouting);
    assert_eq!(events2[1].outcome, "fast");
    assert!(events2[1].shadow_mode);
}

#[test]
fn test_risk_evaluator_telemetry_and_shadow_mode() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path();

    let mock = crate::decisions::mock::MockDecisionProvider::new()
        .with_canned_answer(
            "writes_filesystem",
            crate::decisions::types::DecisionAnswer::Boolean {
                value: true,
                confidence: 0.95,
            },
        )
        .with_canned_answer(
            "overall_risk",
            crate::decisions::types::DecisionAnswer::Choice {
                selected: "destructive".into(),
                confidence: 0.95,
                probabilities: Default::default(),
            },
        );

    let config = crate::decisions::RiskConfig {
        enabled: true,
        thresholds: crate::decisions::RiskThresholds::default(),
        fallback: crate::decisions::RiskFallbackPolicy::RequireConfirmation,
    };

    let shadow_engine =
        crate::decisions::DecisionEngine::with_mock(mock, crate::decisions::DecisionMode::Shadow);
    let evaluator = crate::decisions::RiskEvaluator::new(&config, Some(&shadow_engine))
        .with_config_dir(config_dir)
        .with_workflow(Some("feature-test"));

    let result = evaluator.evaluate("run_command", "python deploy.py", None);
    assert_eq!(result.policy, crate::decisions::ExecutionPolicy::Allow);
    assert!(result.reason.contains("Shadow mode active"));

    let events = read_decision_events(config_dir).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].decision_type, DecisionType::RiskClassification);
    assert_eq!(events[0].outcome, "deny");
    assert!(events[0].shadow_mode);
}
