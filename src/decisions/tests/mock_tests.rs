use super::*;
use crate::decisions::types::{DecisionAnswer, DecisionContext, DecisionQuestion, DecisionRequest};

#[test]
fn test_mock_provider_evaluates_default_answers() {
    let mock = MockDecisionProvider::new();
    let req = DecisionRequest::new(DecisionContext::new("Test task"))
        .with_question(DecisionQuestion::boolean("q_bool", "Is ok?"))
        .with_question(DecisionQuestion::choice(
            "q_choice",
            "Pick one",
            vec!["opt_a", "opt_b"],
        ))
        .with_question(DecisionQuestion::score("q_score", "Score", 0.0, 10.0));

    let resp = mock.evaluate(req).expect("evaluation failed");
    assert_eq!(resp.provider, "mock");
    assert_eq!(resp.answers.len(), 3);

    let ans_bool = resp.get_answer("q_bool").unwrap();
    assert_eq!(ans_bool.as_boolean(), Some(false));

    let ans_choice = resp.get_answer("q_choice").unwrap();
    assert_eq!(ans_choice.as_choice(), Some("opt_a"));

    let ans_score = resp.get_answer("q_score").unwrap();
    assert_eq!(ans_score.as_score(), Some(0.0));
}

#[test]
fn test_mock_provider_with_canned_answers() {
    let mock = MockDecisionProvider::new().with_canned_answer(
        "q_custom",
        DecisionAnswer::Boolean {
            value: true,
            confidence: 0.99,
        },
    );

    let req = DecisionRequest::new(DecisionContext::new("Test custom"))
        .with_question(DecisionQuestion::boolean("q_custom", "Custom?"));

    let resp = mock.evaluate(req).expect("evaluation failed");
    let ans = resp.get_answer("q_custom").unwrap();
    assert_eq!(ans.as_boolean(), Some(true));
    assert_eq!(ans.confidence(), 0.99);
}

#[test]
fn test_mock_provider_simulated_failure_and_health() {
    let mock = MockDecisionProvider::new();
    let health = mock.check_health().expect("health check failed");
    assert!(health.available);

    mock.set_failure(Some("Connection refused".into()));
    let req = DecisionRequest::new(DecisionContext::new("Test fail"));
    assert!(mock.evaluate(req).is_err());

    let health_failing = mock
        .check_health()
        .expect("health check should return status");
    assert!(!health_failing.available);
    assert!(health_failing.message.contains("Connection refused"));
}
