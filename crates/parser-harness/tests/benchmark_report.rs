//! Benchmark report validation tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_harness::benchmark_report::{
    ArtifactSizeEvidence, BenchmarkEvidence, BenchmarkMetric, BenchmarkReport,
    BenchmarkReportValidationError, BenchmarkTier, BenchmarkWorkload, ParityStatus, TenXStatus,
};

#[test]
fn benchmark_report_should_accept_selected_and_whole_list_evidence() {
    // Arrange + Act
    let report = valid_report(
        selected_evidence(TenXStatus::Pass, None),
        Some(whole_list_evidence(TenXStatus::Pass, None)),
        None,
    )
    .expect("selected plus whole-list benchmark report should pass");

    // Assert
    assert_eq!(report.selected_evidence.ten_x_status, TenXStatus::Pass);
    assert!(report.whole_list_or_corpus_evidence.is_some());
}

#[test]
fn benchmark_report_should_accept_selected_with_unavailable_reason() {
    // Arrange + Act
    let report = valid_report(
        selected_evidence(TenXStatus::Unknown, Some("baseline not run yet".to_owned())),
        None,
        Some("RUN_PHASE5_FULL_CORPUS not enabled".to_owned()),
    )
    .expect("selected evidence with concrete whole-list unavailable reason should pass");

    // Assert
    assert_eq!(
        report.whole_list_unavailable_reason.as_deref(),
        Some("RUN_PHASE5_FULL_CORPUS not enabled")
    );
}

#[test]
fn benchmark_report_should_reject_missing_artifact_size() {
    // Arrange
    let mut evidence = selected_evidence(TenXStatus::Pass, None);
    evidence.artifact_size.compact_artifact_bytes = 0;

    // Act
    let error = valid_report(evidence, None, Some("RUN_PHASE5_FULL_CORPUS not enabled".to_owned()))
        .expect_err("missing compact artifact bytes should fail validation");

    // Assert
    assert_eq!(
        error,
        BenchmarkReportValidationError::MissingArtifactSizeEvidence {
            field: "compact_artifact_bytes"
        }
    );
}

#[test]
fn benchmark_report_should_reject_invalid_artifact_raw_ratio() {
    // Arrange
    let mut evidence = selected_evidence(TenXStatus::Pass, None);
    evidence.artifact_size.artifact_raw_ratio = 9.9;

    // Act
    let error = valid_report(evidence, None, Some("RUN_PHASE5_FULL_CORPUS not enabled".to_owned()))
        .expect_err("invalid artifact/raw ratio should fail validation");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::InvalidArtifactRawRatio);
}

#[test]
fn benchmark_report_should_reject_missing_whole_list_evidence() {
    // Arrange + Act
    let error = valid_report(selected_evidence(TenXStatus::Pass, None), None, None)
        .expect_err("missing whole-list evidence and reason should fail validation");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::MissingWholeListEvidence);
}

#[test]
fn benchmark_report_should_reject_failed_ten_x_without_artifact_triage() {
    // Arrange + Act
    let error = valid_report(
        selected_evidence(
            TenXStatus::Fail,
            Some("bottleneck: too slow; parity: human review".to_owned()),
        ),
        None,
        Some("RUN_PHASE5_FULL_CORPUS not enabled".to_owned()),
    )
    .expect_err("failed 10x report should require artifact triage");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::FailedTenXRequiresTriage);
}

#[test]
fn benchmark_report_should_reject_missing_rss_note_when_rss_is_absent() {
    // Arrange
    let mut evidence = selected_evidence(TenXStatus::Pass, None);
    evidence.parse_only = metric_without_rss();

    // Act
    let error = BenchmarkReport::new(
        "05.1",
        "deterministic WORKER_COUNT=1",
        "WORKER_COUNT=1 pnpm run parse",
        "replay-parser-2 parse",
        evidence,
        None,
        Some("RUN_PHASE5_FULL_CORPUS not enabled".to_owned()),
        None,
    )
    .expect_err("missing rss_note should fail when rss_mb is absent");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::MissingRssNote);
}

fn valid_report(
    selected_evidence: BenchmarkEvidence,
    whole_list_or_corpus_evidence: Option<BenchmarkEvidence>,
    whole_list_unavailable_reason: Option<String>,
) -> Result<BenchmarkReport, BenchmarkReportValidationError> {
    BenchmarkReport::new(
        "05.1",
        "deterministic WORKER_COUNT=1",
        "HOME=/tmp/sg-baseline WORKER_COUNT=1 pnpm run parse",
        "cargo run --release -p parser-cli --bin replay-parser-2 -- parse",
        selected_evidence,
        whole_list_or_corpus_evidence,
        whole_list_unavailable_reason,
        None,
    )
}

fn selected_evidence(ten_x_status: TenXStatus, triage: Option<String>) -> BenchmarkEvidence {
    BenchmarkEvidence {
        workload_name: "selected compact replay".to_owned(),
        tier: BenchmarkTier::SmallCi,
        workload: workload(BenchmarkTier::SmallCi),
        artifact_size: ArtifactSizeEvidence::new(1024, 256, 0.25),
        parse_only: metric_with_rss(),
        aggregate_only: metric_with_rss(),
        end_to_end: metric_with_rss(),
        parity_status: ParityStatus::Passed,
        ten_x_status,
        triage,
    }
}

fn whole_list_evidence(ten_x_status: TenXStatus, triage: Option<String>) -> BenchmarkEvidence {
    BenchmarkEvidence {
        workload_name: "whole replay list".to_owned(),
        tier: BenchmarkTier::ManualFullCorpus,
        workload: BenchmarkWorkload {
            tier: BenchmarkTier::ManualFullCorpus,
            fixtures: Vec::new(),
            corpus_selector: Some("~/sg_stats/lists/replaysList.json".to_owned()),
            total_bytes: 4096,
        },
        artifact_size: ArtifactSizeEvidence::new(4096, 512, 0.125),
        parse_only: metric_with_rss(),
        aggregate_only: metric_with_rss(),
        end_to_end: metric_with_rss(),
        parity_status: ParityStatus::Passed,
        ten_x_status,
        triage,
    }
}

fn workload(tier: BenchmarkTier) -> BenchmarkWorkload {
    BenchmarkWorkload {
        tier,
        fixtures: vec!["crates/parser-core/tests/fixtures/aggregate-combat.ocap.json".to_owned()],
        corpus_selector: None,
        total_bytes: 1024,
    }
}

const fn metric_with_rss() -> BenchmarkMetric {
    BenchmarkMetric::new(10.0, Some(100.0), Some(12.5), Some(1_000.0), Some(64.0))
}

const fn metric_without_rss() -> BenchmarkMetric {
    BenchmarkMetric::new(10.0, Some(100.0), Some(12.5), Some(1_000.0), None)
}
