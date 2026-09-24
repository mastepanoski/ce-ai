use super::*;
use crate::decisions::types::{
    DecisionAnswer, DecisionContext, DecisionQuestion, DecisionRequest, SystemOneWireAnswer,
};

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
    ))
    .with_question(DecisionQuestion::score(
        "risk_level",
        "Rate risk score",
        0.0,
        1.0,
    ));

    let wire = provider.build_wire_payload(req);
    assert_eq!(wire.model, "jev-latest");
    assert_eq!(wire.questions.len(), 3);

    // Validate state contents
    let state_obj = wire.state.as_object().expect("state object");
    assert_eq!(
        state_obj.get("task").and_then(|v| v.as_str()),
        Some("Audit security of auth module")
    );
    assert_eq!(
        state_obj.get("stage").and_then(|v| v.as_str()),
        Some("verify")
    );
    assert_eq!(
        state_obj.get("mode").and_then(|v| v.as_str()),
        Some("compound")
    );

    // Validate questions
    let q_bool = wire.questions.get("needs_audit").unwrap();
    assert_eq!(q_bool.question_type, "noul");
    assert_eq!(
        q_bool.instructions.as_deref(),
        Some("Needs security review?")
    );

    let q_choice = wire.questions.get("model_class").unwrap();
    assert_eq!(q_choice.question_type, "choice");
    let criteria = q_choice.criteria.as_ref().unwrap().as_object().unwrap();
    assert!(criteria.contains_key("fast"));
    assert!(criteria.contains_key("reasoning"));

    let q_score = wire.questions.get("risk_level").unwrap();
    assert_eq!(q_score.question_type, "score");

    let serialized = serde_json::to_string(&wire).expect("serialize wire request");
    let deserialized: SystemOneWireRequest =
        serde_json::from_str(&serialized).expect("deserialize wire request");
    assert_eq!(deserialized.model, "jev-latest");
    assert_eq!(deserialized.questions.len(), 3);
}

#[test]
fn test_jev_wire_response_parsing_from_usage() {
    let provider = JevProvider::new(JevConfig::default(), Some("ts-test-key".into()));

    let sample_json = r#"{
        "model": "jev-latest",
        "latency_ms": 38,
        "usage": {
            "estimated_cost_usd": 0.0003
        },
        "answers": {
            "is_risky": {
                "type": "noul",
                "noul": 0.94,
                "confidence": 0.94
            },
            "category": {
                "type": "choice",
                "choice": "security",
                "confidence": 0.89,
                "probabilities": {
                    "security": 0.89,
                    "refactor": 0.11
                }
            },
            "severity": {
                "type": "score",
                "score": 4.5,
                "confidence": 0.95
            }
        }
    }"#;

    let wire_resp: SystemOneWireResponse =
        serde_json::from_str(sample_json).expect("parse sample response");
    assert_eq!(wire_resp.answers.len(), 3);

    let decision_resp = provider.parse_wire_response(wire_resp, 42);
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

    let ans_score = decision_resp
        .get_answer("severity")
        .expect("severity answer");
    assert_eq!(ans_score.as_score(), Some(4.5));
    assert!((ans_score.confidence() - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_jev_wire_response_parsing_top_level_cost() {
    let provider = JevProvider::new(JevConfig::default(), Some("ts-test-key".into()));
    let mut answers = std::collections::BTreeMap::new();
    answers.insert(
        "q1".into(),
        SystemOneWireAnswer {
            answer_type: Some("noul".into()),
            noul: Some(0.2),
            confidence: Some(0.6),
            ..Default::default()
        },
    );

    let wire_resp = SystemOneWireResponse {
        model: Some("jev-preview".into()),
        answers,
        latency_ms: None,
        usage: None,
        estimated_cost_usd: Some(0.00015),
    };

    let resp = provider.parse_wire_response(wire_resp, 50);
    assert_eq!(resp.model, "jev-preview");
    assert_eq!(resp.latency_ms, 50);
    assert_eq!(resp.estimated_cost_usd, Some(0.00015));
    let ans = resp.get_answer("q1").unwrap();
    match ans {
        DecisionAnswer::Boolean { value, confidence } => {
            assert!(!*value);
            assert_eq!(*confidence, 0.6);
        }
        _ => panic!("expected boolean answer"),
    }
}
