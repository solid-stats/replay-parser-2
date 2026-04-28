//! Comparison report vocabulary tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_harness::report::{
    ComparisonBaseline, ComparisonFinding, ComparisonInput, ComparisonReport, ImpactAssessment,
    ImpactLevel, MismatchCategory, ReportValidationError,
};
use serde_json::{Value, json};

#[test]
fn comparison_report_mismatch_categories_should_serialize_as_phase_1_snake_case_values() {
    // Arrange
    let cases = [
        (MismatchCategory::Compatible, "compatible"),
        (MismatchCategory::IntentionalChange, "intentional_change"),
        (MismatchCategory::OldBugPreserved, "old_bug_preserved"),
        (MismatchCategory::OldBugFixed, "old_bug_fixed"),
        (MismatchCategory::NewBug, "new_bug"),
        (MismatchCategory::InsufficientData, "insufficient_data"),
        (MismatchCategory::HumanReview, "human_review"),
    ];

    // Act + Assert
    for (category, expected) in cases {
        let serialized = serde_json::to_value(category)
            .expect("mismatch category should serialize to JSON value");
        assert_eq!(serialized, Value::String(expected.to_owned()));
    }
}

#[test]
fn comparison_report_impact_assessment_should_require_every_downstream_dimension() {
    // Arrange + Act
    let error = ImpactAssessment::try_new(
        Some(ImpactLevel::Yes),
        Some(ImpactLevel::No),
        None,
        Some(ImpactLevel::Unknown),
    )
    .expect_err("missing server_2_recalculation should fail validation");

    // Assert
    assert_eq!(
        error,
        ReportValidationError::MissingImpactDimension { dimension: "server_2_recalculation" }
    );
}

#[test]
fn comparison_report_current_vs_regenerated_drift_should_remain_human_review() {
    // Arrange
    let baseline = ComparisonBaseline {
        old_profile: "current-results-vs-worker-count-1".to_owned(),
        old_command: "pnpm run parse".to_owned(),
        worker_count: Some(1),
        source: "current_vs_regenerated_drift".to_owned(),
        diagnostic_only: false,
    };
    let finding = ComparisonFinding::new(
        "results.digest",
        None,
        MismatchCategory::Compatible,
        ImpactAssessment::unknown(),
        json!("old"),
        json!("new"),
    );

    // Act
    let error = ComparisonReport::new(
        baseline,
        vec![ComparisonInput::new("old", "~/sg_stats/results")],
        vec![finding],
    )
    .expect_err("unexplained drift cannot be classified before human review");

    // Assert
    assert_eq!(
        error,
        ReportValidationError::BaselineDriftMustRemainHumanReview {
            surface: "results.digest".to_owned(),
            category: MismatchCategory::Compatible,
        }
    );
}
