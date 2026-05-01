//! Fault report gate tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_harness::fault_report::{
    FaultCase, FaultOutcome, FaultReport, FaultReportValidationError, FaultRisk,
};

#[test]
fn fault_report_gate_should_accept_caught_cases_for_required_targets() {
    // Arrange
    let cases = required_target_cases(FaultOutcome::Caught, FaultRisk::High);

    // Act
    let report = FaultReport::new("deterministic-fault-injection", "2026-04-28", cases)
        .expect("caught required-target report should validate");

    // Assert
    assert_eq!(report.summary.total_cases, 3);
    assert_eq!(report.summary.by_outcome["caught"], 3);
}

#[test]
fn fault_report_gate_should_reject_high_risk_missed_cases_without_rationale() {
    // Arrange
    let cases = required_target_cases(FaultOutcome::Missed, FaultRisk::High);

    // Act
    let error = FaultReport::new("deterministic-fault-injection", "2026-04-28", cases)
        .expect_err("high-risk missed fault should block validation");

    // Assert
    assert_eq!(
        error,
        FaultReportValidationError::HighRiskMissed {
            id: "events.same-side-teamkill".to_owned(),
            target: "parser-core::events".to_owned(),
        }
    );
}

#[test]
fn fault_report_gate_should_accept_high_risk_missed_case_with_non_applicable_rationale() {
    // Arrange
    let mut cases = required_target_cases(FaultOutcome::Caught, FaultRisk::High);
    cases[0] = FaultCase::new(
        "events.generated-equivalent",
        "parser-core::events",
        "generated equivalent mutation is not applicable to observable parser behavior",
        FaultRisk::High,
        FaultOutcome::Missed,
        vec!["manual review".to_owned()],
    )
    .with_non_applicable_reason("mutation only changes generated monomorphization");

    // Act
    let report = FaultReport::new("cargo-mutants", "2026-04-28", cases)
        .expect("accepted non-applicable missed fault should validate");

    // Assert
    assert_eq!(report.summary.high_risk_missed, 1);
}

#[test]
fn fault_report_gate_should_reject_reports_without_required_target_coverage() {
    // Arrange
    let cases = vec![FaultCase::new(
        "events.same-side-teamkill",
        "parser-core::events",
        "same-side non-suicide kill should not be an enemy kill",
        FaultRisk::High,
        FaultOutcome::Caught,
        vec!["cargo test -p parser-core fault_injection_regressions".to_owned()],
    )];

    // Act
    let error = FaultReport::new("deterministic-fault-injection", "2026-04-28", cases)
        .expect_err("missing aggregate and minimal artifact target coverage should fail");

    // Assert
    assert_eq!(
        error,
        FaultReportValidationError::MissingRequiredTarget { target: "parser-core::aggregates" }
    );
}

#[test]
fn fault_report_gate_should_reject_timeout_and_unviable_cases_without_evidence() {
    // Arrange
    let mut cases = required_target_cases(FaultOutcome::Caught, FaultRisk::High);
    cases[0] = FaultCase::new(
        "events.timeout",
        "parser-core::events",
        "mutation timed out before reaching an assertion",
        FaultRisk::Medium,
        FaultOutcome::Timeout,
        Vec::new(),
    );

    // Act
    let error = FaultReport::new("cargo-mutants", "2026-04-28", cases)
        .expect_err("timeout without evidence should fail validation");

    // Assert
    assert_eq!(
        error,
        FaultReportValidationError::OutcomeRequiresEvidence {
            id: "events.timeout".to_owned(),
            outcome: FaultOutcome::Timeout,
        }
    );
}

fn required_target_cases(outcome: FaultOutcome, risk: FaultRisk) -> Vec<FaultCase> {
    vec![
        FaultCase::new(
            "events.same-side-teamkill",
            "parser-core::events",
            "same-side non-suicide kill should not be an enemy kill",
            risk,
            outcome,
            vec!["cargo test -p parser-core fault_injection_regressions".to_owned()],
        ),
        FaultCase::new(
            "aggregates.replay-local-counters",
            "parser-core::aggregates",
            "aggregate counters should preserve vehicleKills and killsFromVehicle",
            risk,
            outcome,
            vec!["cargo test -p parser-core fault_injection_regressions".to_owned()],
        ),
        FaultCase::new(
            "minimal-artifact.debug-only-fields",
            "parser-core::minimal_artifact",
            "default artifacts should omit debug-only keys while debug sidecar keeps provenance",
            risk,
            outcome,
            vec!["cargo test -p parser-core debug_artifact".to_owned()],
        ),
    ]
}
