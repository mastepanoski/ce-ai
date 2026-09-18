use super::*;

#[test]
fn test_decision_mode_parsing_and_display() {
    assert_eq!(DecisionMode::parse("off").unwrap(), DecisionMode::Off);
    assert_eq!(DecisionMode::parse("disabled").unwrap(), DecisionMode::Off);
    assert_eq!(DecisionMode::parse("shadow").unwrap(), DecisionMode::Shadow);
    assert_eq!(DecisionMode::parse("active").unwrap(), DecisionMode::Active);
    assert_eq!(DecisionMode::parse("on").unwrap(), DecisionMode::Active);
    assert_eq!(
        DecisionMode::parse("enabled").unwrap(),
        DecisionMode::Active
    );
    assert!(DecisionMode::parse("invalid-mode").is_err());

    assert_eq!(DecisionMode::Off.as_str(), "off");
    assert_eq!(DecisionMode::Shadow.as_str(), "shadow");
    assert_eq!(DecisionMode::Active.as_str(), "active");
    assert_eq!(format!("{}", DecisionMode::Active), "active");
}

#[test]
fn test_decision_question_builders_and_getters() {
    let q_bool = DecisionQuestion::boolean("q1", "Is this task complex?");
    assert_eq!(q_bool.id(), "q1");
    assert_eq!(q_bool.question(), "Is this task complex?");

    let q_choice = DecisionQuestion::choice("q2", "Pick a model class", vec!["fast", "reasoning"]);
    assert_eq!(q_choice.id(), "q2");
    assert_eq!(q_choice.question(), "Pick a model class");

    let q_score = DecisionQuestion::score("q3", "Rate risk level", 0.0, 1.0);
    assert_eq!(q_score.id(), "q3");
    assert_eq!(q_score.question(), "Rate risk level");
}

#[test]
fn test_decision_request_and_context_serialization_round_trip() {
    let ctx = DecisionContext::new("Refactor database schema")
        .with_stage("plan")
        .with_mode("compound")
        .with_meta("author", "dev1");

    let req = DecisionRequest::new(ctx)
        .with_question(DecisionQuestion::boolean(
            "needs_migration",
            "Requires DB migration?",
        ))
        .with_question(DecisionQuestion::choice(
            "risk",
            "Estimated risk",
            vec!["low", "medium", "high"],
        ));

    let serialized = serde_json::to_string(&req).expect("serialization failed");
    let deserialized: DecisionRequest =
        serde_json::from_str(&serialized).expect("deserialization failed");

    assert_eq!(req, deserialized);
    assert_eq!(deserialized.questions.len(), 2);
    assert_eq!(deserialized.context.workflow_stage.as_deref(), Some("plan"));
    assert_eq!(
        deserialized.context.execution_mode.as_deref(),
        Some("compound")
    );
    assert_eq!(
        deserialized
            .context
            .metadata
            .get("author")
            .map(|s| s.as_str()),
        Some("dev1")
    );
}

#[test]
fn test_decision_answer_helpers() {
    let ans_bool = DecisionAnswer::Boolean {
        value: true,
        confidence: 0.95,
    };
    assert_eq!(ans_bool.as_boolean(), Some(true));
    assert_eq!(ans_bool.as_choice(), None);
    assert_eq!(ans_bool.confidence(), 0.95);

    let ans_choice = DecisionAnswer::Choice {
        selected: "reasoning".into(),
        confidence: 0.88,
        probabilities: BTreeMap::new(),
    };
    assert_eq!(ans_choice.as_choice(), Some("reasoning"));
    assert_eq!(ans_choice.as_boolean(), None);
    assert_eq!(ans_choice.confidence(), 0.88);

    let ans_score = DecisionAnswer::Score {
        score: 0.75,
        confidence: 0.90,
    };
    assert_eq!(ans_score.as_score(), Some(0.75));
    assert_eq!(ans_score.confidence(), 0.90);
}

#[test]
fn test_decision_response_helpers_and_serialization() {
    let mut resp = DecisionResponse::new("jev", "jev-v1", 45);
    resp = resp.with_answer(
        "q1",
        DecisionAnswer::Boolean {
            value: true,
            confidence: 0.9,
        },
    );

    assert_eq!(resp.provider, "jev");
    assert_eq!(resp.model, "jev-v1");
    assert_eq!(resp.latency_ms, 45);
    assert!(!resp.fallback_used);

    let ans = resp.get_answer("q1").expect("missing answer");
    assert_eq!(ans.as_boolean(), Some(true));

    let serialized = serde_json::to_string(&resp).expect("serialization failed");
    let deserialized: DecisionResponse =
        serde_json::from_str(&serialized).expect("deserialization failed");
    assert_eq!(resp, deserialized);
}
