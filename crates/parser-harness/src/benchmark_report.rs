//! Serializable Phase 05.2 benchmark report vocabulary and release-gate validation. coverage-exclusion: reviewed Phase 05 benchmark-report defensive validator branches are allowlisted by exact source line.
//!
//! Benchmark reports tie performance observations to selected large-replay
//! evidence, all-raw corpus coverage, deterministic old baseline evidence, and
//! the hard default artifact size limit.

use serde::{Deserialize, Serialize};

/// Current benchmark report schema version.
pub const BENCHMARK_REPORT_VERSION: &str = "2";

/// Phase 05.2 hard maximum size for each successful default parser artifact.
pub const DEFAULT_ARTIFACT_SIZE_LIMIT_BYTES: u64 = 100_000;

/// Deterministic large replay selection policy recorded in reports.
pub const SELECTED_LARGE_REPLAY_SELECTION_POLICY: &str =
    "largest .json by byte size under ~/sg_stats/raw_replays; tie-break lexicographic path";

/// All-raw corpus selector recorded in reports.
pub const ALL_RAW_CORPUS_SELECTOR: &str =
    "~/sg_stats/raw_replays/**/*.json sorted lexicographically";

/// Pass/fail/unknown status for a benchmark gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    /// The measured gate passed.
    Pass,
    /// The measured gate failed.
    Fail,
    /// The gate could not be evaluated for this report.
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

/// User approval state for a corpus failure allowlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllowlistApprovalStatus {
    /// The user explicitly approved the allowlist.
    AcceptedByUser,
    /// An allowlist exists but has not been approved.
    PendingUserApproval,
    /// The allowlist was rejected.
    Rejected,
}

/// Optional failure/skip allowlist for all-raw corpus acceptance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkAllowlist {
    /// Path to the allowlist evidence file.
    pub path: String,
    /// User approval state.
    pub approval_status: AllowlistApprovalStatus,
}

impl BenchmarkAllowlist {
    fn validate(&self) -> Result<(), BenchmarkReportValidationError> {
        validate_non_empty(&self.path, "allowlist.path")
    }
}

/// Benchmark evidence for the deterministic selected large replay performance sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedLargeReplay {
    /// Stable selection policy used to choose the replay.
    pub selection_policy: String,
    /// Selected replay path.
    pub path: String,
    /// Raw replay size in bytes.
    pub raw_bytes: u64,
    /// Selected replay SHA-256.
    pub sha256: String,
    /// Old parser wall-clock time in milliseconds.
    pub old_wall_time_ms: Option<f64>,
    /// New parser wall-clock time in milliseconds.
    pub new_wall_time_ms: Option<f64>,
    /// Old/new speedup when both timings are available.
    pub speedup: Option<f64>,
    /// Historical x3 target status for the selected replay.
    pub x3_status: GateStatus,
    /// Old-vs-new parity status.
    pub parity_status: ParityStatus,
    /// Default artifact size in bytes.
    pub artifact_bytes: u64,
    /// `artifact_bytes / raw_bytes`.
    pub artifact_raw_ratio: f64,
    /// Hard max-artifact-size status.
    pub artifact_size_status: GateStatus,
    /// Required triage for failed or unknown selected-replay gates.
    pub triage: Option<String>,
}

impl SelectedLargeReplay {
    /// Validates selected large-replay evidence.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when identity, ratio, x3, or
    /// hard artifact-size evidence is inconsistent.
    pub fn validate(&self, limit_bytes: u64) -> Result<(), BenchmarkReportValidationError> {
        validate_non_empty(&self.selection_policy, "selected_large_replay.selection_policy")?;
        validate_non_empty(&self.path, "selected_large_replay.path")?;
        validate_non_empty(&self.sha256, "selected_large_replay.sha256")?;

        if self.raw_bytes == 0 {
            return Err(BenchmarkReportValidationError::MissingByteEvidence {
                field: "selected_large_replay.raw_bytes",
            });
        }

        if self.artifact_bytes == 0 {
            return Err(BenchmarkReportValidationError::MissingByteEvidence {
                field: "selected_large_replay.artifact_bytes",
            });
        }

        let expected_ratio = bytes_as_f64(self.artifact_bytes) / bytes_as_f64(self.raw_bytes);
        if (self.artifact_raw_ratio - expected_ratio).abs() > 0.0001 {
            return Err(BenchmarkReportValidationError::InvalidSelectedArtifactRawRatio);
        }

        if self.x3_status == GateStatus::Pass
            && !(self.speedup.is_some_and(|speedup| speedup >= 3.0)
                && self.parity_status == ParityStatus::Passed)
        {
            return Err(BenchmarkReportValidationError::SelectedX3PassRequiresSpeedupAndParity);
        }

        if self.artifact_size_status == GateStatus::Pass && self.artifact_bytes > limit_bytes {
            return Err(BenchmarkReportValidationError::SelectedArtifactSizePassRequiresLimit);
        }

        if (self.x3_status != GateStatus::Pass
            || self.artifact_size_status != GateStatus::Pass
            || self.parity_status != ParityStatus::Passed)
            && !triage_mentions_required_terms(self.triage.as_deref())
        {
            return Err(BenchmarkReportValidationError::StatusRequiresTriage {
                scope: "selected_large_replay",
            });
        }

        Ok(())
    }
}

/// Benchmark evidence for the all-raw corpus performance and size gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllRawCorpus {
    /// Corpus selector.
    pub selector: String,
    /// Number of raw replay files attempted.
    pub attempted_count: u64,
    /// Number of successful default artifacts.
    pub success_count: u64,
    /// Number of failed parse attempts.
    pub failed_count: u64,
    /// Number of skipped artifacts.
    pub skipped_count: u64,
    /// Total raw bytes attempted.
    pub raw_bytes: u64,
    /// Total successful default artifact bytes.
    pub artifact_bytes: u64,
    /// Old parser corpus wall-clock time in milliseconds.
    pub old_wall_time_ms: Option<f64>,
    /// New parser corpus wall-clock time in milliseconds.
    pub new_wall_time_ms: Option<f64>,
    /// Old/new speedup when both timings are available.
    pub speedup: Option<f64>,
    /// Historical x10 target status for the all-raw corpus.
    pub x10_status: GateStatus,
    /// Median artifact/raw ratio across successful artifacts.
    pub median_artifact_raw_ratio: Option<f64>,
    /// p95 artifact/raw ratio across successful artifacts.
    pub p95_artifact_raw_ratio: Option<f64>,
    /// Maximum successful default artifact bytes.
    pub max_artifact_bytes: u64,
    /// Number of successful default artifacts above the hard byte limit.
    pub oversized_artifact_count: u64,
    /// Hard max-artifact-size gate status.
    pub size_gate_status: GateStatus,
    /// Failure compatibility status. This passes for zero failures or for a
    /// user-approved allowlist of known malformed/non-JSON raw files.
    pub zero_failure_status: GateStatus,
    /// Required triage for failed or unknown all-raw gates.
    pub triage: Option<String>,
}

impl AllRawCorpus {
    /// Validates all-raw corpus evidence.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when count accounting, x10,
    /// failure compatibility, or size-gate status is inconsistent.
    pub fn validate(
        &self,
        limit_bytes: u64,
        allowlist: Option<&BenchmarkAllowlist>,
    ) -> Result<(), BenchmarkReportValidationError> {
        validate_non_empty(&self.selector, "all_raw_corpus.selector")?;

        if self.attempted_count != self.success_count + self.failed_count + self.skipped_count {
            return Err(BenchmarkReportValidationError::InvalidAllRawCounts);
        }

        if self.x10_status == GateStatus::Pass
            && !(self.attempted_count > 0 && self.speedup.is_some_and(|speedup| speedup >= 10.0))
        {
            return Err(BenchmarkReportValidationError::AllRawX10PassRequiresSpeedup);
        }

        if self.size_gate_status == GateStatus::Pass
            && !(self.success_count > 0
                && self.max_artifact_bytes <= limit_bytes
                && self.oversized_artifact_count == 0)
        {
            return Err(BenchmarkReportValidationError::AllRawSizeGatePassRequiresMaxBytes);
        }

        if self.zero_failure_status == GateStatus::Pass
            && !(self.failed_count == 0 && self.skipped_count == 0
                || allowlist.is_some_and(|entry| {
                    entry.approval_status == AllowlistApprovalStatus::AcceptedByUser
                }))
        {
            return Err(
                BenchmarkReportValidationError::ZeroFailurePassRequiresNoFailuresOrAllowlist,
            );
        }

        if (self.x10_status != GateStatus::Pass
            || self.size_gate_status != GateStatus::Pass
            || self.zero_failure_status != GateStatus::Pass)
            && !triage_mentions_required_terms(self.triage.as_deref())
        {
            return Err(BenchmarkReportValidationError::StatusRequiresTriage {
                scope: "all_raw_corpus",
            });
        }

        Ok(())
    }
}

/// Parser benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Report schema version.
    pub report_version: String,
    /// Phase that produced this report.
    pub phase: String,
    /// Old baseline profile, always documenting deterministic `WORKER_COUNT=1`.
    pub old_baseline_profile: String,
    /// Hard default artifact size limit in bytes.
    pub artifact_size_limit_bytes: u64,
    /// Deterministic selected large-replay evidence.
    pub selected_large_replay: SelectedLargeReplay,
    /// All-raw corpus evidence.
    pub all_raw_corpus: AllRawCorpus,
    /// Optional failure/skip allowlist.
    pub allowlist: Option<BenchmarkAllowlist>,
    /// Explanation when memory/RSS is not practical to capture.
    pub rss_note: String,
}

impl BenchmarkReport {
    /// Builds and validates a Phase 05.2 benchmark report.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when required release-gate
    /// evidence is missing or inconsistent.
    pub fn new(
        old_baseline_profile: impl Into<String>,
        selected_large_replay: SelectedLargeReplay,
        all_raw_corpus: AllRawCorpus,
        allowlist: Option<BenchmarkAllowlist>,
        rss_note: impl Into<String>,
    ) -> Result<Self, BenchmarkReportValidationError> {
        let report = Self {
            report_version: BENCHMARK_REPORT_VERSION.to_owned(),
            phase: "05.2".to_owned(),
            old_baseline_profile: old_baseline_profile.into(),
            artifact_size_limit_bytes: DEFAULT_ARTIFACT_SIZE_LIMIT_BYTES,
            selected_large_replay,
            all_raw_corpus,
            allowlist,
            rss_note: rss_note.into(),
        };
        report.validate()?;
        Ok(report)
    }

    /// Validates benchmark report invariants.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when a required field is
    /// missing or inconsistent with Phase 05.2 benchmark decisions.
    pub fn validate(&self) -> Result<(), BenchmarkReportValidationError> {
        validate_non_empty(&self.report_version, "report_version")?;
        validate_non_empty(&self.phase, "phase")?;
        validate_non_empty(&self.old_baseline_profile, "old_baseline_profile")?;
        validate_non_empty(&self.rss_note, "rss_note")?;

        if self.phase != "05.2" {
            return Err(BenchmarkReportValidationError::InvalidPhase);
        }

        if self.artifact_size_limit_bytes != DEFAULT_ARTIFACT_SIZE_LIMIT_BYTES {
            return Err(BenchmarkReportValidationError::InvalidArtifactSizeLimit {
                expected: DEFAULT_ARTIFACT_SIZE_LIMIT_BYTES,
                actual: self.artifact_size_limit_bytes,
            });
        }

        if !self.old_baseline_profile.contains("WORKER_COUNT=1") {
            return Err(BenchmarkReportValidationError::MissingDeterministicOldBaseline);
        }

        if let Some(allowlist) = &self.allowlist {
            allowlist.validate()?;
        }

        self.selected_large_replay.validate(self.artifact_size_limit_bytes)?;
        self.all_raw_corpus.validate(self.artifact_size_limit_bytes, self.allowlist.as_ref())?;

        Ok(())
    }

    /// Validates that this structurally valid report is Phase 05.2 acceptance evidence.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkReportValidationError`] when structural validation fails or when the
    /// current accepted Phase 05.2 benchmark gates are not passing.
    pub fn validate_acceptance(&self) -> Result<(), BenchmarkReportValidationError> {
        self.validate()?;

        if self.selected_large_replay.artifact_size_status != GateStatus::Pass
            || self.all_raw_corpus.old_wall_time_ms.is_none()
            || self.all_raw_corpus.new_wall_time_ms.is_none()
            || self.all_raw_corpus.speedup.is_none()
            || self.all_raw_corpus.size_gate_status != GateStatus::Pass
            || self.all_raw_corpus.zero_failure_status != GateStatus::Pass
        {
            return Err(BenchmarkReportValidationError::AcceptanceGatesNotPassed);
        }

        Ok(())
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

fn triage_mentions_required_terms(triage: Option<&str>) -> bool {
    let Some(triage) = triage else {
        return false;
    };
    let triage = triage.to_ascii_lowercase();
    ["bottleneck", "parity", "artifact", "failure"]
        .into_iter()
        .all(|required| triage.contains(required))
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
    /// A required byte evidence field was zero.
    #[error("benchmark report is missing byte evidence field `{field}`")]
    MissingByteEvidence {
        /// Missing or zero byte field.
        field: &'static str,
    },
    /// Phase is not the Phase 05.2 acceptance report phase.
    #[error("benchmark report phase must be 05.2")]
    InvalidPhase,
    /// Artifact-size limit is not the hard default limit.
    #[error("benchmark report artifact_size_limit_bytes must be {expected} bytes, got {actual}")]
    InvalidArtifactSizeLimit {
        /// Required limit.
        expected: u64,
        /// Reported limit.
        actual: u64,
    },
    /// Old baseline profile is not the deterministic baseline.
    #[error("benchmark report is missing deterministic WORKER_COUNT=1 old baseline")]
    MissingDeterministicOldBaseline,
    /// Selected artifact/raw ratio does not match the recorded bytes.
    #[error("selected_large_replay artifact_raw_ratio does not match artifact_bytes/raw_bytes")]
    InvalidSelectedArtifactRawRatio,
    /// Selected x3 status cannot pass without speedup >= 3.0 and passed parity.
    #[error("selected x3_status=pass requires speedup >= 3.0 and parity_status=passed")]
    SelectedX3PassRequiresSpeedupAndParity,
    /// Selected hard artifact-size status cannot pass above the byte limit.
    #[error(
        "selected artifact_size_status=pass requires artifact_bytes <= artifact_size_limit_bytes"
    )]
    SelectedArtifactSizePassRequiresLimit,
    /// All-raw attempted/success/failed/skipped counts are inconsistent.
    #[error(
        "all_raw_corpus attempted_count must equal success_count + failed_count + skipped_count"
    )]
    InvalidAllRawCounts,
    /// All-raw x10 status cannot pass without speedup >= 10.0.
    #[error("all_raw_corpus x10_status=pass requires speedup >= 10.0")]
    AllRawX10PassRequiresSpeedup,
    /// All-raw size gate cannot pass without hard max/oversized limits.
    #[error(
        "all_raw_corpus size_gate_status=pass requires max_artifact_bytes <= limit and oversized_artifact_count == 0"
    )]
    AllRawSizeGatePassRequiresMaxBytes,
    /// Failure compatibility cannot pass without no failures/skips or an accepted allowlist.
    #[error(
        "all_raw_corpus zero_failure_status=pass requires failed_count == 0 and skipped_count == 0, unless allowlist.approval_status=accepted_by_user"
    )]
    ZeroFailurePassRequiresNoFailuresOrAllowlist,
    /// Failed or unknown status lacks required triage terms.
    #[error(
        "{scope} failed or unknown statuses require triage mentioning bottleneck, parity, artifact, and failure"
    )]
    StatusRequiresTriage {
        /// Report scope needing triage.
        scope: &'static str,
    },
    /// Structurally valid report does not satisfy accepted Phase 05.2 benchmark gates.
    #[error(
        "benchmark acceptance requires selected artifact-size pass, all-raw timing evidence, all-raw max-size pass, and accepted all-raw failure compatibility"
    )]
    AcceptanceGatesNotPassed,
}
