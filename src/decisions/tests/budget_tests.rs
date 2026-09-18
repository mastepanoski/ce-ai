use super::*;

#[test]
fn test_budget_allows_execution_within_limits() {
    let config = BudgetConfig {
        max_monthly_cents: 500,
        max_session_requests: 10,
        timeout_ms: 500,
        max_consecutive_failures: 3,
        cooloff_secs: 60,
    };

    let mut tracker = BudgetTracker::in_memory(config);
    assert!(tracker.can_execute().is_ok());

    tracker.record_success(Some(0.10));
    assert_eq!(tracker.session_requests(), 1);
    assert_eq!(tracker.total_monthly_requests(), 1);
    assert_eq!(tracker.accumulated_spend_cents(), 10);
    assert!((tracker.accumulated_spend_usd() - 0.10).abs() < f64::EPSILON);
    assert!(tracker.can_execute().is_ok());
}

#[test]
fn test_budget_exceeded_trips_fallback() {
    let config = BudgetConfig {
        max_monthly_cents: 100,
        max_session_requests: 100,
        timeout_ms: 500,
        max_consecutive_failures: 3,
        cooloff_secs: 60,
    };

    let mut tracker = BudgetTracker::in_memory(config);
    tracker.record_success(Some(0.95));
    assert!(tracker.can_execute().is_ok());

    tracker.record_success(Some(0.10)); // Total 1.05 > 1.00
    let err = tracker.can_execute().expect_err("should exceed budget");
    assert!(matches!(err, FallbackReason::BudgetExceeded { .. }));
    assert!(format!("{err}").contains("monthly budget limit reached"));
}

#[test]
fn test_session_limit_exceeded_trips_fallback() {
    let config = BudgetConfig {
        max_monthly_cents: 1000,
        max_session_requests: 2,
        timeout_ms: 500,
        max_consecutive_failures: 3,
        cooloff_secs: 60,
    };

    let mut tracker = BudgetTracker::in_memory(config);
    tracker.record_success(Some(0.01));
    tracker.record_success(Some(0.01));

    let err = tracker
        .can_execute()
        .expect_err("should exceed session limit");
    assert!(matches!(err, FallbackReason::SessionLimitExceeded { .. }));
    assert!(format!("{err}").contains("session request limit reached"));
}

#[test]
fn test_circuit_breaker_trips_on_consecutive_failures() {
    let config = BudgetConfig {
        max_monthly_cents: 1000,
        max_session_requests: 100,
        timeout_ms: 500,
        max_consecutive_failures: 3,
        cooloff_secs: 30,
    };

    let mut tracker = BudgetTracker::in_memory(config);
    tracker.record_failure();
    assert_eq!(tracker.consecutive_failures(), 1);
    assert!(tracker.can_execute().is_ok());

    tracker.record_failure();
    assert_eq!(tracker.consecutive_failures(), 2);
    assert!(tracker.can_execute().is_ok());

    tracker.record_failure(); // 3 consecutive failures
    assert_eq!(tracker.consecutive_failures(), 3);

    let err = tracker.can_execute().expect_err("circuit should be open");
    assert!(matches!(err, FallbackReason::CircuitOpen { .. }));
    assert!(format!("{err}").contains("circuit breaker open"));

    // Success resets consecutive failures
    tracker.record_success(None);
    assert_eq!(tracker.consecutive_failures(), 0);
    assert!(tracker.can_execute().is_ok());
}

#[test]
fn test_monthly_rollover_resets_spend() {
    let config = BudgetConfig::default();
    let mut tracker = BudgetTracker::in_memory(config);

    // Simulate an old month
    tracker.ledger.month = "2025-01".into();
    tracker.ledger.accumulated_spend_cents = 490;
    tracker.ledger.total_requests = 42;

    assert!(tracker.can_execute().is_ok());
    assert_ne!(tracker.ledger.month, "2025-01");
    assert_eq!(tracker.accumulated_spend_cents(), 0);
    assert_eq!(tracker.total_monthly_requests(), 0);
}
