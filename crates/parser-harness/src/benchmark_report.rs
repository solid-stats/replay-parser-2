//! Serializable benchmark report vocabulary and release-gate validation.
// coverage-exclusion: reviewed Phase 05 benchmark report defensive branches are allowlisted by exact source line.
//!
//! Benchmark reports tie speed claims to workload identity, parity status,
//! deterministic old baseline evidence, throughput, and memory/RSS notes.

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

/// Compact artifact-size evidence for a measured workload.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArtifactSizeEvidence {
    /// Raw replay input bytes processed by the workload.
    pub raw_input_bytes: u64,
    /// Compact parser artifact bytes emitted by the workload.
    pub compact_artifact_bytes: u64,
    /// `compact_artifact_bytes / raw_input_bytes`.
    pub artifact_raw_ratio: f64,
}

impl ArtifactSizeEvidence {
    /// Builds compact artifact-size evidence.
    #[must_use]
    pub const fn new(
        raw_input_bytes: u64,
        compact_artifact_bytes: u64,
        artifact_raw_ratio: f64,
    ) -> Self {
        Self { raw_input_bytes, compact_artifact_bytes, artifact_raw_ratio }
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

/// Benchmark evidence for one selected or whole-list/corpus workload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkEvidence {
    /// Human-readable workload name.
    pub workload_name: String,
    /// Workload tier.
    pub tier: BenchmarkTier,
    /// Workload identity and byte count.
    pub workload: BenchmarkWorkload,
    /// Compact artifact-size evidence for the workload.
    pub artifact_size: ArtifactSizeEvidence,
    /// Parse-only stage evidence.
    pub parse_only: BenchmarkMetric,
    /// Aggregate-only or nearest public equivalent stage evidence.
    pub aggregate_only: BenchmarkMetric,
    /// End-to-end parser evidence.
    pub end_to_end: BenchmarkMetric,
    /// Parity status for the measured workload.
    pub parity_status: ParityStatus,
    /// 10x target status.
    pub ten_x_status: TenXStatus,
    /// Required triage for failed or unknown 10x status.
    pub triage: Option<String>,
}

impl BenchmarkEvidence {
    /// Validates evidence invariants for one workload.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when workload identity,
    /// compact size evidence, or 10x triage is missing.
    pub fn validate(&self) -> Result<(), BenchmarkReportValidationError> {
        validate_non_empty(&self.workload_name, "workload_name")?;

        if self.tier != self.workload.tier {
            return Err(BenchmarkReportValidationError::MismatchedEvidenceTier);
        }
        if !self.workload.has_identity() {
            return Err(BenchmarkReportValidationError::MissingWorkloadIdentity);
        }
        if self.artifact_size.raw_input_bytes == 0 {
            return Err(BenchmarkReportValidationError::MissingArtifactSizeEvidence {
                field: "raw_input_bytes",
            });
        }
        if self.artifact_size.compact_artifact_bytes == 0 {
            return Err(BenchmarkReportValidationError::MissingArtifactSizeEvidence {
                field: "compact_artifact_bytes",
            });
        }

        let expected_ratio = bytes_as_f64(self.artifact_size.compact_artifact_bytes)
            / bytes_as_f64(self.artifact_size.raw_input_bytes);
        if (self.artifact_size.artifact_raw_ratio - expected_ratio).abs() > 0.0001 {
            return Err(BenchmarkReportValidationError::InvalidArtifactRawRatio);
        }

        let triage = self.triage.as_deref().unwrap_or_default();
        let triage_lower = triage.to_ascii_lowercase();
        if self.ten_x_status == TenXStatus::Fail
            && !(triage_lower.contains("bottleneck")
                && triage_lower.contains("parity")
                && triage_lower.contains("artifact"))
        {
            return Err(BenchmarkReportValidationError::FailedTenXRequiresTriage);
        }

        if self.ten_x_status == TenXStatus::Unknown && triage.trim().is_empty() {
            return Err(BenchmarkReportValidationError::UnknownTenXRequiresTriage);
        }

        Ok(())
    }

    const fn any_missing_rss(&self) -> bool {
        self.parse_only.rss_mb.is_none()
            || self.aggregate_only.rss_mb.is_none()
            || self.end_to_end.rss_mb.is_none()
    }
}

/// Parser benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Report schema version.
    pub report_version: String,
    /// Phase that produced this report.
    pub phase: String,
    /// Old baseline profile, normally deterministic `WORKER_COUNT=1`.
    pub old_baseline_profile: String,
    /// Old command used or documented for the run.
    pub old_command: String,
    /// New parser command used or documented for the run.
    pub new_command: String,
    /// Selected replay/sample evidence that is cheap enough for CI.
    pub selected_evidence: BenchmarkEvidence,
    /// Whole-list or corpus evidence when prerequisites are available.
    pub whole_list_or_corpus_evidence: Option<BenchmarkEvidence>,
    /// Concrete reason whole-list/corpus evidence is unavailable.
    pub whole_list_unavailable_reason: Option<String>,
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
        phase: impl Into<String>,
        old_baseline_profile: impl Into<String>,
        old_command: impl Into<String>,
        new_command: impl Into<String>,
        selected_evidence: BenchmarkEvidence,
        whole_list_or_corpus_evidence: Option<BenchmarkEvidence>,
        whole_list_unavailable_reason: Option<String>,
        rss_note: Option<String>,
    ) -> Result<Self, BenchmarkReportValidationError> {
        let report = Self {
            report_version: BENCHMARK_REPORT_VERSION.to_owned(),
            phase: phase.into(),
            old_baseline_profile: old_baseline_profile.into(),
            old_command: old_command.into(),
            new_command: new_command.into(),
            selected_evidence,
            whole_list_or_corpus_evidence,
            whole_list_unavailable_reason,
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
        validate_non_empty(&self.phase, "phase")?;
        validate_non_empty(&self.old_baseline_profile, "old_baseline_profile")?;
        validate_non_empty(&self.old_command, "old_command")?;
        validate_non_empty(&self.new_command, "new_command")?;
        self.selected_evidence.validate()?;

        if let Some(evidence) = &self.whole_list_or_corpus_evidence {
            evidence.validate()?;
        } else if self
            .whole_list_unavailable_reason
            .as_deref()
            .is_none_or(|reason| reason.trim().is_empty())
        {
            return Err(BenchmarkReportValidationError::MissingWholeListEvidence);
        }

        let selected_triage = self.selected_evidence.triage.as_deref().unwrap_or_default();
        let selected_triage_lower = selected_triage.to_ascii_lowercase();
        if !self.old_baseline_profile.contains("WORKER_COUNT=1")
            && (self.selected_evidence.ten_x_status != TenXStatus::Unknown
                || !selected_triage_lower.contains("baseline"))
        {
            return Err(BenchmarkReportValidationError::MissingDeterministicOldBaseline);
        }

        if self.any_missing_rss()
            && self.rss_note.as_deref().is_none_or(|note| note.trim().is_empty())
        {
            return Err(BenchmarkReportValidationError::MissingRssNote);
        }

        Ok(())
    }

    fn any_missing_rss(&self) -> bool {
        self.selected_evidence.any_missing_rss()
            || self
                .whole_list_or_corpus_evidence
                .as_ref()
                .is_some_and(BenchmarkEvidence::any_missing_rss)
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

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "benchmark byte-count ratios are approximate evidence and validated with tolerance"
)]
const fn bytes_as_f64(value: u64) -> f64 {
    value as f64
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
    /// `BenchmarkEvidence` tier and nested workload tier differ.
    #[error("benchmark evidence tier does not match workload tier")]
    MismatchedEvidenceTier,
    /// Compact artifact-size evidence is missing.
    #[error("benchmark report is missing artifact size field `{field}`")]
    MissingArtifactSizeEvidence {
        /// Missing or zero artifact-size field.
        field: &'static str,
    },
    /// Artifact/raw ratio does not match the recorded bytes.
    #[error(
        "benchmark report artifact_raw_ratio does not match compact_artifact_bytes/raw_input_bytes"
    )]
    InvalidArtifactRawRatio,
    /// No whole-list/corpus evidence and no unavailable rationale were provided.
    #[error(
        "benchmark report requires whole-list/corpus evidence or whole_list_unavailable_reason"
    )]
    MissingWholeListEvidence,
    /// Old baseline profile is not the deterministic baseline and lacks accepted unknown triage.
    #[error("benchmark report is missing deterministic WORKER_COUNT=1 old baseline")]
    MissingDeterministicOldBaseline,
    /// Failed 10x status lacks required bottleneck, parity, and artifact triage.
    #[error("benchmark report ten_x_status=fail requires bottleneck, parity, and artifact triage")]
    FailedTenXRequiresTriage,
    /// Unknown 10x status lacks explanatory triage.
    #[error("benchmark report ten_x_status=unknown requires triage")]
    UnknownTenXRequiresTriage,
    /// RSS was omitted without a note.
    #[error("benchmark report is missing rss_note while one or more rss_mb values are absent")]
    MissingRssNote,
}
