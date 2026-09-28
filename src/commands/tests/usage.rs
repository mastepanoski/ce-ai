use super::*;
use crate::capture::ledger::UsageRecord;

fn record(timestamp: &str, session_id: &str) -> UsageRecord {
    UsageRecord {
        author: "audit".into(),
        timestamp: timestamp.into(),
        harness: "claude".into(),
        session_id: session_id.into(),
        cwd_basename: "repo".into(),
        model: "test".into(),
        input_tokens: 1,
        output_tokens: 2,
        cache_read: 0,
        cache_write: 0,
        reasoning_tokens: 0,
    }
}

#[test]
fn report_filters_records_by_inclusive_rfc3339_interval() {
    let records = vec![
        record("2020-01-01T00:00:00Z", "old"),
        record("2025-06-01T00:00:00Z", "inside"),
        record("2030-01-01T00:00:00Z", "new"),
    ];

    let filtered = filter_records(
        records,
        Some("2025-01-01T00:00:00Z"),
        Some("2026-01-01T00:00:00Z"),
    )
    .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].session_id, "inside");
}

#[test]
fn report_rejects_invalid_or_inverted_date_filters() {
    assert!(matches!(
        filter_records(vec![], Some("not-a-timestamp"), None),
        Err(CeError::Usage(_))
    ));
    assert!(matches!(
        filter_records(
            vec![],
            Some("2026-01-01T00:00:00Z"),
            Some("2025-01-01T00:00:00Z")
        ),
        Err(CeError::Usage(_))
    ));
}

#[test]
fn report_rejects_unsupported_grouping_instead_of_ignoring_it() {
    assert!(matches!(
        validate_group_by(Some("harness")),
        Err(CeError::Usage(_))
    ));
    assert!(validate_group_by(Some("record")).is_ok());
}
