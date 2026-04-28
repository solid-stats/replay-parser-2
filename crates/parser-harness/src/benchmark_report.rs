//! Serializable benchmark report vocabulary and release-gate validation.
//!
//! Benchmark reports must tie speed claims to workload identity, parity status,
//! the deterministic old baseline profile, throughput, and memory/RSS evidence
//! or an explicit note explaining why RSS was not practical.

use serde::{Deserialize, Serialize};

/// Current benchmark report schema version.
pub const BENCHMARK_REPORT_VERSION: &str = "1";

/// Benchmark workload tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkTier {
    /// Small sample intended for CI and local smoke checks.
    SmallCi,
    /// Curated representative sample with broader replay shape coverage.
    CuratedRepresentative,
    /// Optional/manual full historical corpus run.
    ManualFullCorpus,
}

/// Status of the roughly 10x target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenXStatus {
    /// The measured workload met or exceeded the 10x target.
    Pass,
    /// The measured workload did not meet the 10x target.
    Fail,
    /// The target could not be evaluated for this run.
    Unknown,
}

/// Parity status for the measured workload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParityStatus {
    /// Parity checks passed for the measured workload.
    Passed,
    /// Parity checks failed for the measured workload.
    Failed,
    /// Parity evidence requires human review before classification.
    HumanReview,
    /// Parity was not run for this report.
    NotRun,
}

/// Timing and throughput metrics for one benchmark dimension.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkMetric {
    /// Wall-clock time in milliseconds.
    pub wall_time_ms: f64,
    /// Files processed per second when available.
    pub files_per_sec: Option<f64>,
    /// Megabytes processed per second when available.
    pub mb_per_sec: Option<f64>,
    /// Events processed per second when available.
    pub events_per_sec: Option<f64>,
    /// Resident set size in MiB when practical to measure.
    pub rss_mb: Option<f64>,
}

impl BenchmarkMetric {
    /// Builds a metric with optional throughput fields.
    #[must_use]
    pub const fn new(
        wall_time_ms: f64,
        files_per_sec: Option<f64>,
        mb_per_sec: Option<f64>,
        events_per_sec: Option<f64>,
        rss_mb: Option<f64>,
    ) -> Self {
        Self { wall_time_ms, files_per_sec, mb_per_sec, events_per_sec, rss_mb }
    }
}

/// Workload identity for a benchmark report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkWorkload {
    /// Workload tier.
    pub tier: BenchmarkTier,
    /// Exact fixture paths used by the run.
    pub fixtures: Vec<String>,
    /// Corpus selector used when fixtures are not enumerated.
    pub corpus_selector: Option<String>,
    /// Total input bytes processed.
    pub total_bytes: u64,
}

impl BenchmarkWorkload {
    fn has_identity(&self) -> bool {
        !self.fixtures.is_empty()
            || self.corpus_selector.as_deref().is_some_and(|selector| !selector.trim().is_empty())
    }
}

/// Parser benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Report schema version.
    pub report_version: String,
    /// Old baseline profile, normally deterministic `WORKER_COUNT=1`.
    pub old_baseline_profile: String,
    /// Old command used or documented for the run.
    pub old_command: String,
    /// New parser command used or documented for the run.
    pub new_command: String,
    /// Workload identity.
    pub workload: BenchmarkWorkload,
    /// Parse-only stage evidence.
    pub parse_only: BenchmarkMetric,
    /// Aggregate-only or nearest public equivalent stage evidence.
    pub aggregate_only: BenchmarkMetric,
    /// End-to-end parser evidence.
    pub end_to_end: BenchmarkMetric,
    /// Parity status for the measured workload.
    pub parity_status: Option<ParityStatus>,
    /// 10x target status.
    pub ten_x_status: TenXStatus,
    /// Required triage for failed or unknown 10x status when baseline evidence is missing.
    pub triage: Option<String>,
    /// Explanation when memory/RSS is not practical for one or more metrics.
    pub rss_note: Option<String>,
}

impl BenchmarkReport {
    /// Builds and validates a benchmark report.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when required release-gate
    /// evidence is missing or an under-10x report lacks triage.
    #[allow(clippy::too_many_arguments, reason = "report schema mirrors persisted JSON fields")]
    pub fn new(
        old_baseline_profile: impl Into<String>,
        old_command: impl Into<String>,
        new_command: impl Into<String>,
        workload: BenchmarkWorkload,
        parse_only: BenchmarkMetric,
        aggregate_only: BenchmarkMetric,
        end_to_end: BenchmarkMetric,
        parity_status: Option<ParityStatus>,
        ten_x_status: TenXStatus,
        triage: Option<String>,
        rss_note: Option<String>,
    ) -> Result<Self, BenchmarkReportValidationError> {
        let report = Self {
            report_version: BENCHMARK_REPORT_VERSION.to_owned(),
            old_baseline_profile: old_baseline_profile.into(),
            old_command: old_command.into(),
            new_command: new_command.into(),
            workload,
            parse_only,
            aggregate_only,
            end_to_end,
            parity_status,
            ten_x_status,
            triage,
            rss_note,
        };
        report.validate()?;
        Ok(report)
    }

    /// Validates benchmark report invariants.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when a required field is
    /// missing or inconsistent with Phase 05 benchmark decisions.
    pub fn validate(&self) -> Result<(), BenchmarkReportValidationError> {
        validate_non_empty(&self.report_version, "report_version")?;
        validate_non_empty(&self.old_baseline_profile, "old_baseline_profile")?;
        validate_non_empty(&self.old_command, "old_command")?;
        validate_non_empty(&self.new_command, "new_command")?;

        if !self.workload.has_identity() {
            return Err(BenchmarkReportValidationError::MissingWorkloadIdentity);
        }
        if self.parity_status.is_none() {
            return Err(BenchmarkReportValidationError::MissingParityStatus);
        }

        let triage = self.triage.as_deref().unwrap_or_default();
        let triage_lower = triage.to_ascii_lowercase();
        if !self.old_baseline_profile.contains("WORKER_COUNT=1")
            && (self.ten_x_status != TenXStatus::Unknown || !triage_lower.contains("baseline"))
        {
            return Err(BenchmarkReportValidationError::MissingDeterministicOldBaseline);
        }

        if self.ten_x_status == TenXStatus::Fail
            && !(triage_lower.contains("bottleneck") && triage_lower.contains("parity"))
        {
            return Err(BenchmarkReportValidationError::FailedTenXRequiresTriage);
        }

        if self.ten_x_status == TenXStatus::Unknown && triage.trim().is_empty() {
            return Err(BenchmarkReportValidationError::UnknownTenXRequiresTriage);
        }

        if self.any_missing_rss()
            && self.rss_note.as_deref().is_none_or(|note| note.trim().is_empty())
        {
            return Err(BenchmarkReportValidationError::MissingRssNote);
        }

        Ok(())
    }

    const fn any_missing_rss(&self) -> bool {
        self.parse_only.rss_mb.is_none()
            || self.aggregate_only.rss_mb.is_none()
            || self.end_to_end.rss_mb.is_none()
    }
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), BenchmarkReportValidationError> {
    if value.trim().is_empty() {
        return Err(BenchmarkReportValidationError::EmptyField { field });
    }

    Ok(())
}

/// Benchmark report validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BenchmarkReportValidationError {
    /// A required string field was empty.
    #[error("benchmark report field `{field}` is empty")]
    EmptyField {
        /// Empty field name.
        field: &'static str,
    },
    /// Neither fixture list nor corpus selector identified the workload.
    #[error("benchmark report is missing workload identity")]
    MissingWorkloadIdentity,
    /// The report did not provide a parity status.
    #[error("benchmark report is missing parity_status")]
    MissingParityStatus,
    /// Old baseline profile is not the deterministic baseline and lacks accepted unknown triage.
    #[error("benchmark report is missing deterministic WORKER_COUNT=1 old baseline")]
    MissingDeterministicOldBaseline,
    /// Failed 10x status lacks required bottleneck and parity triage.
    #[error("benchmark report ten_x_status=fail requires bottleneck and parity triage")]
    FailedTenXRequiresTriage,
    /// Unknown 10x status lacks explanatory triage.
    #[error("benchmark report ten_x_status=unknown requires triage")]
    UnknownTenXRequiresTriage,
    /// RSS was omitted without a note.
    #[error("benchmark report is missing rss_note while one or more rss_mb values are absent")]
    MissingRssNote,
}
