//! Benchmark report validation tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_harness::benchmark_report::{
    ALL_RAW_CORPUS_SELECTOR, AllowlistApprovalStatus, BenchmarkAllowlist, BenchmarkReport,
    BenchmarkReportValidationError, DEFAULT_ARTIFACT_SIZE_LIMIT_BYTES, GateStatus, ParityStatus,
    SELECTED_LARGE_REPLAY_SELECTION_POLICY, SelectedLargeReplay,
};

use parser_harness::benchmark_report::AllRawCorpus;

const PARSER_PIPELINE_BENCH: &str = include_str!("../benches/parser_pipeline.rs");

#[test]
fn benchmark_report_should_accept_phase_05_2_acceptance_evidence() {
    // Arrange + Act
    let report = valid_report(selected_large_replay(), all_raw_corpus(), None)
        .expect("Phase 05.2 benchmark report should pass");

    // Assert
    assert_eq!(report.phase, "05.2");
    assert_eq!(report.artifact_size_limit_bytes, 100_000);
    assert_eq!(report.artifact_size_limit_bytes, DEFAULT_ARTIFACT_SIZE_LIMIT_BYTES);
    assert_eq!(report.selected_large_replay.x3_status, GateStatus::Pass);
    assert_eq!(report.all_raw_corpus.x10_status, GateStatus::Pass);
    assert_eq!(report.all_raw_corpus.size_gate_status, GateStatus::Pass);
}

#[test]
fn benchmark_report_should_reject_wrong_artifact_size_limit() {
    // Arrange
    let mut report = valid_report(selected_large_replay(), all_raw_corpus(), None)
        .expect("baseline report should be valid");
    report.artifact_size_limit_bytes = 100_001;

    // Act
    let error = report.validate().expect_err("artifact_size_limit_bytes must be exactly 100_000");

    // Assert
    assert_eq!(
        error,
        BenchmarkReportValidationError::InvalidArtifactSizeLimit {
            expected: 100_000,
            actual: 100_001
        }
    );
}

#[test]
fn benchmark_report_should_reject_selected_x3_pass_below_speedup_gate() {
    // Arrange
    let mut selected = selected_large_replay();
    selected.speedup = Some(2.99);

    // Act
    let error = valid_report(selected, all_raw_corpus(), None)
        .expect_err("selected x3 pass should require speedup >= 3.0");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::SelectedX3PassRequiresSpeedupAndParity);
}

#[test]
fn benchmark_report_should_reject_selected_x3_pass_without_parity() {
    // Arrange
    let mut selected = selected_large_replay();
    selected.parity_status = ParityStatus::Failed;

    // Act
    let error = valid_report(selected, all_raw_corpus(), None)
        .expect_err("selected x3 pass should require passed parity");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::SelectedX3PassRequiresSpeedupAndParity);
}

#[test]
fn benchmark_report_should_reject_selected_artifact_size_above_100000() {
    // Arrange
    let mut selected = selected_large_replay();
    selected.artifact_bytes = 100_001;
    selected.artifact_raw_ratio = 100_001.0 / 2_000_000.0;

    // Act
    let error = valid_report(selected, all_raw_corpus(), None)
        .expect_err("selected artifact_size_status pass should require artifact_bytes <= 100_000");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::SelectedArtifactSizePassRequiresLimit);
}

#[test]
fn benchmark_report_should_reject_all_raw_x10_pass_below_speedup_gate() {
    // Arrange
    let mut all_raw = all_raw_corpus();
    all_raw.speedup = Some(9.99);

    // Act
    let error = valid_report(selected_large_replay(), all_raw, None)
        .expect_err("all-raw x10 pass should require speedup >= 10.0");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::AllRawX10PassRequiresSpeedup);
}

#[test]
fn benchmark_report_should_reject_median_ratio_above_0_05() {
    // Arrange
    let mut all_raw = all_raw_corpus();
    all_raw.median_artifact_raw_ratio = Some(0.0501);

    // Act
    let error = valid_report(selected_large_replay(), all_raw, None)
        .expect_err("all-raw size gate should reject median above 0.05");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::AllRawSizeGatePassRequiresRatioAndMaxBytes);
}

#[test]
fn benchmark_report_should_reject_p95_ratio_above_0_10() {
    // Arrange
    let mut all_raw = all_raw_corpus();
    all_raw.p95_artifact_raw_ratio = Some(0.1001);

    // Act
    let error = valid_report(selected_large_replay(), all_raw, None)
        .expect_err("all-raw size gate should reject p95 above 0.10");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::AllRawSizeGatePassRequiresRatioAndMaxBytes);
}

#[test]
fn benchmark_report_should_reject_all_raw_max_artifact_size_above_100000() {
    // Arrange
    let mut all_raw = all_raw_corpus();
    all_raw.max_artifact_bytes = 100_001;

    // Act
    let error = valid_report(selected_large_replay(), all_raw, None)
        .expect_err("all-raw size gate should reject max artifact bytes above 100000");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::AllRawSizeGatePassRequiresRatioAndMaxBytes);
}

#[test]
fn benchmark_report_should_reject_all_raw_oversized_artifact_count() {
    // Arrange
    let mut all_raw = all_raw_corpus();
    all_raw.oversized_artifact_count = 1;

    // Act
    let error = valid_report(selected_large_replay(), all_raw, None)
        .expect_err("all-raw size gate should reject oversized artifacts");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::AllRawSizeGatePassRequiresRatioAndMaxBytes);
}

#[test]
fn benchmark_report_should_reject_zero_failure_pass_with_failed_or_skipped_files() {
    // Arrange
    let mut all_raw = all_raw_corpus();
    all_raw.success_count = 1;
    all_raw.failed_count = 1;
    all_raw.skipped_count = 1;

    // Act
    let error = valid_report(selected_large_replay(), all_raw, None)
        .expect_err("zero-failure pass should require no failures/skips without allowlist");

    // Assert
    assert_eq!(error, BenchmarkReportValidationError::ZeroFailurePassRequiresNoFailuresOrAllowlist);
}

#[test]
fn benchmark_report_should_accept_zero_failure_pass_with_user_accepted_allowlist() {
    // Arrange
    let mut all_raw = all_raw_corpus();
    all_raw.success_count = 1;
    all_raw.failed_count = 1;
    all_raw.skipped_count = 1;
    let allowlist = BenchmarkAllowlist {
        path: ".planning/generated/phase-05/benchmarks/all-raw-allowlist.json".to_owned(),
        approval_status: AllowlistApprovalStatus::AcceptedByUser,
    };

    // Act + Assert
    let report = valid_report(selected_large_replay(), all_raw, Some(allowlist))
        .expect("accepted allowlist should permit zero_failure_status pass");

    // Assert
    assert_eq!(
        report.allowlist.as_ref().expect("allowlist should be preserved").approval_status,
        AllowlistApprovalStatus::AcceptedByUser
    );
}

#[test]
fn benchmark_report_should_reject_failed_or_unknown_status_without_complete_triage() {
    // Arrange
    let mut selected = selected_large_replay();
    selected.x3_status = GateStatus::Unknown;
    selected.parity_status = ParityStatus::NotRun;
    selected.triage = Some("bottleneck and parity and artifact mentioned".to_owned());

    // Act
    let error = valid_report(selected, all_raw_corpus(), None)
        .expect_err("unknown selected status should require failure triage");

    // Assert
    assert_eq!(
        error,
        BenchmarkReportValidationError::StatusRequiresTriage { scope: "selected_large_replay" }
    );
}

#[test]
fn benchmark_report_should_measure_selective_decode_not_full_value_decode() {
    assert!(PARSER_PIPELINE_BENCH.contains("decode_compact_root"));
    assert!(!PARSER_PIPELINE_BENCH.contains("from_slice::<Value>"));
}

fn valid_report(
    selected_large_replay: SelectedLargeReplay,
    all_raw_corpus: AllRawCorpus,
    allowlist: Option<BenchmarkAllowlist>,
) -> Result<BenchmarkReport, BenchmarkReportValidationError> {
    BenchmarkReport::new(
        "deterministic WORKER_COUNT=1",
        selected_large_replay,
        all_raw_corpus,
        allowlist,
        "RSS is not captured in the portable smoke path.",
    )
}

fn selected_large_replay() -> SelectedLargeReplay {
    SelectedLargeReplay {
        selection_policy: SELECTED_LARGE_REPLAY_SELECTION_POLICY.to_owned(),
        path: "/home/afgan0r/sg_stats/raw_replays/selected.ocap.json".to_owned(),
        raw_bytes: 2_000_000,
        sha256: "6666666666666666666666666666666666666666666666666666666666666666".to_owned(),
        old_wall_time_ms: Some(300.0),
        new_wall_time_ms: Some(90.0),
        speedup: Some(3.333_333),
        x3_status: GateStatus::Pass,
        parity_status: ParityStatus::Passed,
        artifact_bytes: 100_000,
        artifact_raw_ratio: 100_000.0 / 2_000_000.0,
        artifact_size_status: GateStatus::Pass,
        triage: None,
    }
}

fn all_raw_corpus() -> AllRawCorpus {
    AllRawCorpus {
        selector: ALL_RAW_CORPUS_SELECTOR.to_owned(),
        attempted_count: 3,
        success_count: 3,
        failed_count: 0,
        skipped_count: 0,
        raw_bytes: 6_000_000,
        artifact_bytes: 180_000,
        old_wall_time_ms: Some(10_000.0),
        new_wall_time_ms: Some(900.0),
        speedup: Some(11.111_111),
        x10_status: GateStatus::Pass,
        median_artifact_raw_ratio: Some(0.03),
        p95_artifact_raw_ratio: Some(0.09),
        max_artifact_bytes: 90_000,
        oversized_artifact_count: 0,
        size_gate_status: GateStatus::Pass,
        zero_failure_status: GateStatus::Pass,
        triage: None,
    }
}
