use super::*;

#[test]
fn test_engine_mode_off_returns_fallback_immediately() {
    let mock = MockDecisionProvider::new();
    let engine = DecisionEngine::with_mock(mock, DecisionMode::Off);

    assert!(!engine.is_enabled());
    assert_eq!(engine.mode(), DecisionMode::Off);

    let req = DecisionRequest::new(DecisionContext::new("Test task"))
        .with_question(DecisionQuestion::boolean("q1", "Check?"));

    let resp = engine.evaluate(req).expect("off evaluation should succeed");
    assert!(resp.fallback_used);
    assert_eq!(resp.model, "disabled");
}

#[test]
fn test_engine_mode_shadow_evaluates_and_flags_fallback() {
    let mock = MockDecisionProvider::new().with_canned_answer(
        "q1",
        DecisionAnswer::Boolean {
            value: true,
            confidence: 0.95,
        },
    );
    let engine = DecisionEngine::with_mock(mock, DecisionMode::Shadow);

    assert!(engine.is_enabled());
    assert_eq!(engine.mode(), DecisionMode::Shadow);

    let req = DecisionRequest::new(DecisionContext::new("Shadow task"))
        .with_question(DecisionQuestion::boolean("q1", "Check?"));

    let resp = engine
        .evaluate(req)
        .expect("shadow evaluation should succeed");
    assert!(resp.fallback_used);
    let ans = resp.get_answer("q1").unwrap();
    assert_eq!(ans.as_boolean(), Some(true));
}

#[test]
fn test_engine_mode_active_evaluates_provider() {
    let mock = MockDecisionProvider::new().with_canned_answer(
        "q1",
        DecisionAnswer::Boolean {
            value: true,
            confidence: 0.98,
        },
    );
    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);

    assert!(engine.is_enabled());

    let req = DecisionRequest::new(DecisionContext::new("Active task"))
        .with_question(DecisionQuestion::boolean("q1", "Check?"));

    let resp = engine
        .evaluate(req)
        .expect("active evaluation should succeed");
    assert!(!resp.fallback_used);
    let ans = resp.get_answer("q1").unwrap();
    assert_eq!(ans.as_boolean(), Some(true));
}

#[test]
fn test_engine_unconfigured_provider_handling() {
    let engine = DecisionEngine::new(None, DecisionMode::Active);
    assert!(!engine.is_enabled());

    let req = DecisionRequest::new(DecisionContext::new("Unconfigured"));
    let resp = engine
        .evaluate(req)
        .expect("unconfigured should return fallback");
    assert!(resp.fallback_used);
    assert_eq!(resp.model, "unconfigured");

    let health = engine.check_health().expect("health check");
    assert!(!health.available);
}
