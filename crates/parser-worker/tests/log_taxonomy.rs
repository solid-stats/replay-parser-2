//! Worker log taxonomy behavior tests.

use std::collections::BTreeSet;

use parser_worker::logging::{LOG_EVENTS, OUTCOMES};

#[test]
fn log_events_should_be_unique_low_cardinality_snake_case() {
    let mut seen = BTreeSet::new();

    for event in LOG_EVENTS {
        assert!(seen.insert(event), "duplicate log event: {event}");
        assert!(is_lowercase_snake_case(event), "event is not lowercase snake_case: {event}");
        assert_no_dynamic_identifier(event);
    }
}

#[test]
fn log_outcomes_should_be_unique_low_cardinality_snake_case() {
    let mut seen = BTreeSet::new();

    for outcome in OUTCOMES {
        assert!(seen.insert(outcome), "duplicate log outcome: {outcome}");
        assert!(is_lowercase_snake_case(outcome), "outcome is not lowercase snake_case: {outcome}");
        assert_no_dynamic_identifier(outcome);
    }
}

fn is_lowercase_snake_case(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('_')
        && !value.ends_with('_')
        && !value.contains("__")
        && value.bytes().all(|byte| byte == b'_' || byte.is_ascii_lowercase())
}

fn assert_no_dynamic_identifier(value: &str) {
    for forbidden in ["job-", "replay-", "raw/", "artifacts/", "sha256", "checksum"] {
        assert!(!value.contains(forbidden), "taxonomy value contains dynamic fragment: {value}");
    }
}
