use super::*;
use crate::decisions::types::{DecisionContext, DecisionQuestion, DecisionRequest};

#[test]
fn test_jev_config_defaults() {
    let cfg = JevConfig::default();
    assert_eq!(cfg.endpoint, "https://api.typesafe.ai/v1");
    assert_eq!(cfg.model, "jev-latest");
    assert_eq!(cfg.timeout_ms, 1000);
}

#[test]
fn test_jev_missing_api_key_returns_usage_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let non_existent = tmp.path().join("credentials.toml");
    std::env::set_var("CE_AI_CREDENTIALS_PATH", non_existent.to_str().unwrap());
    let saved_typesafe = std::env::var("TYPESAFE_API_KEY").ok();
    let saved_jev = std::env::var("JEV_API_KEY").ok();
    std::env::remove_var("TYPESAFE_API_KEY");
    std::env::remove_var("JEV_API_KEY");

    let provider = JevProvider::new(JevConfig::default(), None);
    let has_key = provider.has_api_key();

    let req = DecisionRequest::new(DecisionContext::new("Test task"));
    let err = provider
        .evaluate(req)
        .expect_err("should error without API key");

    let health = provider.check_health().expect("health check");

    if let Some(v) = saved_typesafe {
        std::env::set_var("TYPESAFE_API_KEY", v);
    }
    if let Some(v) = saved_jev {
        std::env::set_var("JEV_API_KEY", v);
    }
    std::env::remove_var("CE_AI_CREDENTIALS_PATH");

    assert!(!has_key);
    assert!(matches!(err, CeError::Usage(_)));
    assert!(format!("{err}").contains("TYPESAFE_API_KEY"));
    assert!(!health.available);
    assert!(health.message.contains("missing TYPESAFE_API_KEY"));
}

#[test]
fn test_jev_wire_payload_serialization_round_trip() {
    let provider = JevProvider::new(JevConfig::default(), Some("ts-test-key".into()));
    assert!(provider.has_api_key());

    let req = DecisionRequest::new(
        DecisionContext::new("Audit security of auth module")
            .with_stage("verify")
            .with_mode("compound")
            .with_meta("author", "alice"),
    )
    .with_question(DecisionQuestion::boolean(
        "needs_audit",
        "Needs security review?",
    ))
    .with_question(DecisionQuestion::choice(
        "model_class",
        "Select model class",
        vec!["fast", "reasoning"],
    ));

    let wire = provider.build_wire_payload(req);
    assert_eq!(wire.model, "jev-latest");
    assert_eq!(wire.task_description, "Audit security of auth module");
    assert_eq!(wire.workflow_stage.as_deref(), Some("verify"));
    assert_eq!(wire.execution_mode.as_deref(), Some("compound"));
    assert_eq!(wire.questions.len(), 2);

    let serialized = serde_json::to_string(&wire).expect("serialize wire request");
    let deserialized: JevWireRequest =
        serde_json::from_str(&serialized).expect("deserialize wire request");
    assert_eq!(deserialized.model, "jev-latest");
    assert_eq!(deserialized.questions.len(), 2);
}

#[test]
fn test_jev_wire_response_parsing() {
    let provider = JevProvider::new(JevConfig::default(), Some("ts-test-key".into()));

    let sample_json = r#"{
        "model": "jev-latest",
        "estimated_cost_usd": 0.0003,
        "answers": {
            "is_risky": {
                "kind": "boolean",
                "value": true,
                "confidence": 0.94
            },
            "category": {
                "kind": "choice",
                "selected": "security",
                "confidence": 0.89,
                "probabilities": {
                    "security": 0.89,
                    "refactor": 0.11
                }
            }
        }
    }"#;

    let wire_resp: JevWireResponse =
        serde_json::from_str(sample_json).expect("parse sample response");
    assert_eq!(wire_resp.answers.len(), 2);

    let decision_resp = provider.parse_wire_response(wire_resp, 38);
    assert_eq!(decision_resp.provider, "jev");
    assert_eq!(decision_resp.model, "jev-latest");
    assert_eq!(decision_resp.latency_ms, 38);
    assert_eq!(decision_resp.estimated_cost_usd, Some(0.0003));
    assert!(!decision_resp.fallback_used);

    let ans_bool = decision_resp
        .get_answer("is_risky")
        .expect("is_risky answer");
    assert_eq!(ans_bool.as_boolean(), Some(true));
    assert!((ans_bool.confidence() - 0.94).abs() < f64::EPSILON);

    let ans_choice = decision_resp
        .get_answer("category")
        .expect("category answer");
    assert_eq!(ans_choice.as_choice(), Some("security"));
    assert!((ans_choice.confidence() - 0.89).abs() < f64::EPSILON);
}
