//! Serializable mutation and deterministic fault-injection report vocabulary.
// coverage-exclusion: reviewed Phase 05 fault report defensive branches are allowlisted by exact source line.
//!
//! Reports are release-gate evidence for fixed or accepted fault escapes.

use std::{
    collections::BTreeMap,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};

/// Current fault report schema version.
pub const FAULT_REPORT_VERSION: &str = "1";

const REQUIRED_TARGETS: [&str; 3] =
    ["parser-core::events", "parser-core::aggregates", "parser-core::minimal_artifact"];

/// Fault execution outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultOutcome {
    /// The test suite caught the injected fault.
    Caught,
    /// The injected fault survived the suite.
    Missed,
    /// The fault run exceeded the configured timeout.
    Timeout,
    /// The fault could not be applied or executed meaningfully.
    Unviable,
}

impl FaultOutcome {
    /// Returns the stable `snake_case` report value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Caught => "caught",
            Self::Missed => "missed",
            Self::Timeout => "timeout",
            Self::Unviable => "unviable",
        }
    }
}

impl Display for FaultOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Fault business risk classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultRisk {
    /// Low parser correctness or downstream risk.
    Low,
    /// Medium parser correctness or downstream risk.
    Medium,
    /// High parser correctness or downstream risk.
    High,
}

/// One mutation or deterministic fault case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultCase {
    /// Stable case identifier.
    pub id: String,
    /// Targeted parser module or behavior.
    pub target: String,
    /// Human-readable fault description.
    pub description: String,
    /// Risk level if this fault escaped.
    pub risk: FaultRisk,
    /// Observed outcome.
    pub outcome: FaultOutcome,
    /// Evidence such as test command, mutant path, or failure assertion.
    pub evidence: Vec<String>,
    /// Rationale for accepting a missed high-risk case as non-applicable.
    pub accepted_non_applicable_reason: Option<String>,
}

impl FaultCase {
    /// Builds a fault case.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        target: impl Into<String>,
        description: impl Into<String>,
        risk: FaultRisk,
        outcome: FaultOutcome,
        evidence: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            description: description.into(),
            risk,
            outcome,
            evidence,
            accepted_non_applicable_reason: None,
        }
    }

    /// Adds an accepted non-applicable reason.
    #[must_use]
    pub fn with_non_applicable_reason(mut self, reason: impl Into<String>) -> Self {
        self.accepted_non_applicable_reason = Some(reason.into());
        self
    }

    fn validate(&self) -> Result<(), FaultReportValidationError> {
        validate_non_empty(&self.id, "id", &self.id)?;
        validate_non_empty(&self.target, "target", &self.id)?;
        validate_non_empty(&self.description, "description", &self.id)?;

        if matches!(self.outcome, FaultOutcome::Timeout | FaultOutcome::Unviable)
            && self.evidence.is_empty()
        {
            return Err(FaultReportValidationError::OutcomeRequiresEvidence {
                id: self.id.clone(),
                outcome: self.outcome,
            });
        }

        if self.risk == FaultRisk::High
            && self.outcome == FaultOutcome::Missed
            && self
                .accepted_non_applicable_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
        {
            return Err(FaultReportValidationError::HighRiskMissed {
                id: self.id.clone(),
                target: self.target.clone(),
            });
        }

        Ok(())
    }
}

/// Aggregate fault report summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultSummary {
    /// Total number of cases.
    pub total_cases: usize,
    /// Case counts keyed by [`FaultOutcome`] stable string.
    pub by_outcome: BTreeMap<String, usize>,
    /// Number of high-risk missed cases, including accepted non-applicable cases.
    pub high_risk_missed: usize,
}

impl FaultSummary {
    /// Builds summary counts from cases.
    #[must_use]
    pub fn from_cases(cases: &[FaultCase]) -> Self {
        let mut by_outcome = BTreeMap::new();
        let mut high_risk_missed = 0;

        for case in cases {
            let count = by_outcome.entry(case.outcome.as_str().to_owned()).or_insert(0);
            *count += 1;
            if case.risk == FaultRisk::High && case.outcome == FaultOutcome::Missed {
                high_risk_missed += 1;
            }
        }

        Self { total_cases: cases.len(), by_outcome, high_risk_missed }
    }
}

/// Mutation or equivalent deterministic fault-injection report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultReport {
    /// Report schema version.
    pub report_version: String,
    /// Tool or fallback generator name.
    pub tool: String,
    /// Deterministic report generation timestamp or label.
    pub generated_at: String,
    /// Fault cases included in the report.
    pub cases: Vec<FaultCase>,
    /// Aggregate case counts.
    pub summary: FaultSummary,
}

impl FaultReport {
    /// Builds and validates a fault report.
    ///
    /// # Errors
    ///
    /// Returns [`FaultReportValidationError`] when the report misses required
    /// parser-core targets, lacks evidence for timeout/unviable cases, or
    /// contains unaccepted high-risk missed cases.
    pub fn new(
        tool: impl Into<String>,
        generated_at: impl Into<String>,
        cases: Vec<FaultCase>,
    ) -> Result<Self, FaultReportValidationError> {
        let report = Self {
            report_version: FAULT_REPORT_VERSION.to_owned(),
            tool: tool.into(),
            generated_at: generated_at.into(),
            summary: FaultSummary::from_cases(&cases),
            cases,
        };
        report.validate()?;
        Ok(report)
    }

    /// Validates report invariants required by Phase 05 Plan 04.
    ///
    /// # Errors
    ///
    /// Returns [`FaultReportValidationError`] when the report is not acceptable
    /// as release-gate evidence.
    pub fn validate(&self) -> Result<(), FaultReportValidationError> {
        validate_non_empty(&self.report_version, "report_version", "report")?;
        validate_non_empty(&self.tool, "tool", "report")?;
        validate_non_empty(&self.generated_at, "generated_at", "report")?;

        for required_target in REQUIRED_TARGETS {
            if !self.cases.iter().any(|case| case.target.contains(required_target)) {
                return Err(FaultReportValidationError::MissingRequiredTarget {
                    target: required_target,
                });
            }
        }

        for case in &self.cases {
            case.validate()?;
        }

        Ok(())
    }
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
    id: &str,
) -> Result<(), FaultReportValidationError> {
    if value.trim().is_empty() {
        return Err(FaultReportValidationError::EmptyField { id: id.to_owned(), field });
    }

    Ok(())
}

/// Fault report validation failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FaultReportValidationError {
    /// A required metadata field was empty.
    #[error("fault report item `{id}` has empty {field}")]
    EmptyField {
        /// Case id or report marker.
        id: String,
        /// Empty field name.
        field: &'static str,
    },
    /// A high-risk missed fault was not accepted as non-applicable.
    #[error("high-risk missed fault `{id}` for `{target}` requires non-applicable rationale")]
    HighRiskMissed {
        /// Fault case id.
        id: String,
        /// Fault target.
        target: String,
    },
    /// A required parser-core target is absent from the report.
    #[error("fault report is missing required target `{target}`")]
    MissingRequiredTarget {
        /// Required target prefix.
        target: &'static str,
    },
    /// Timeout or unviable cases must carry evidence.
    #[error("fault `{id}` outcome `{outcome}` requires evidence")]
    OutcomeRequiresEvidence {
        /// Fault case id.
        id: String,
        /// Outcome requiring evidence.
        outcome: FaultOutcome,
    },
}
