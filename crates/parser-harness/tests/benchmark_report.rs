//! Benchmark report validation tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_harness::benchmark_report::{
    BenchmarkMetric, BenchmarkReport, BenchmarkReportValidationError, BenchmarkTier,
    BenchmarkWorkload, ParityStatus, TenXStatus,
};

#[test]
fn benchmark_report_should_accept_valid_pass_with_deterministic_old_baseline() {
    // Arrange + Act
    let report = valid_report(TenXStatus::Pass, Some(ParityStatus::Passed), None)
        .expect("valid benchmark report should pass");

    // Assert
    assert_eq!(report.ten_x_status, TenXStatus::Pass);
    assert_eq!(report.parity_status, Some(ParityStatus::Passed));
}

#[test]
fn benchmark_report_should_reject_fail_without_bottleneck_and_parity_triage() {
    // Arrange + Act
    let error = valid_report(TenXStatus::Fail, Some(ParityStatus::Passed), Some("too slow".into()))
        .expect_err("failed 10x report should require bottleneck and parity triage");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::FailedTenXRequiresTriage);
}

#[test]
fn benchmark_report_should_reject_missing_parity_status() {
    // Arrange + Act
    let error = valid_report(TenXStatus::Pass, None, None)
        .expect_err("missing parity_status should fail validation");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::MissingParityStatus);
}

#[test]
fn benchmark_report_should_reject_missing_workload_identity() {
    // Arrange
    let workload = BenchmarkWorkload {
        tier: BenchmarkTier::SmallCi,
        fixtures: Vec::new(),
        corpus_selector: None,
        total_bytes: 0,
    };

    // Act
    let error = BenchmarkReport::new(
        "deterministic WORKER_COUNT=1",
        "WORKER_COUNT=1 pnpm run parse",
        "replay-parser-2 parse",
        workload,
        metric_with_rss(),
        metric_with_rss(),
        metric_with_rss(),
        Some(ParityStatus::Passed),
        TenXStatus::Pass,
        None,
        None,
    )
    .expect_err("missing workload identity should fail validation");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::MissingWorkloadIdentity);
}

#[test]
fn benchmark_report_should_reject_missing_rss_note_when_rss_is_absent() {
    // Arrange + Act
    let error = BenchmarkReport::new(
        "deterministic WORKER_COUNT=1",
        "WORKER_COUNT=1 pnpm run parse",
        "replay-parser-2 parse",
        workload(),
        metric_without_rss(),
        metric_with_rss(),
        metric_with_rss(),
        Some(ParityStatus::Passed),
        TenXStatus::Pass,
        None,
        None,
    )
    .expect_err("missing rss_note should fail when rss_mb is absent");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::MissingRssNote);
}

fn valid_report(
    ten_x_status: TenXStatus,
    parity_status: Option<ParityStatus>,
    triage: Option<String>,
) -> Result<BenchmarkReport, BenchmarkReportValidationError> {
    BenchmarkReport::new(
        "deterministic WORKER_COUNT=1",
        "HOME=/tmp/sg-baseline WORKER_COUNT=1 pnpm run parse",
        "cargo run --release -p parser-cli --bin replay-parser-2 -- parse",
        workload(),
        metric_with_rss(),
        metric_with_rss(),
        metric_with_rss(),
        parity_status,
        ten_x_status,
        triage,
        None,
    )
}

fn workload() -> BenchmarkWorkload {
    BenchmarkWorkload {
        tier: BenchmarkTier::SmallCi,
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
