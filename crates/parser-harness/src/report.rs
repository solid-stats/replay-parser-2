//! Serializable old-vs-new comparison report vocabulary.
//!
//! Full-corpus generated reports belong under [`GENERATED_REPORT_ROOT`], not in
//! committed fixture directories. Ordinary v1 parity intentionally excludes
//! annual/yearly nomination outputs. Reports describe parser artifact,
//! `server-2`, and `web` impact dimensions for review; they do not mutate
//! adjacent application state.

use std::{
    collections::BTreeMap,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current comparison report schema version.
pub const COMPARISON_REPORT_VERSION: &str = "1";

/// Ignored generated-output root for bulky Phase 5 parity reports.
pub const GENERATED_REPORT_ROOT: &str = ".planning/generated/phase-05/";

/// Required Phase 1 mismatch categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MismatchCategory {
    /// Outputs match exactly or differ only by an approved non-semantic representation.
    Compatible,
    /// New behavior intentionally differs from legacy behavior due to an accepted decision.
    IntentionalChange,
    /// Legacy behavior appears wrong but is preserved after human review.
    OldBugPreserved,
    /// Legacy behavior appears wrong and is corrected after human review.
    OldBugFixed,
    /// New parser or harness behavior conflicts with accepted reference behavior.
    NewBug,
    /// Available evidence cannot prove whether old or new behavior is correct.
    InsufficientData,
    /// Product, domain, cross-app, or unexplained drift judgment is still required.
    HumanReview,
}

impl MismatchCategory {
    /// Returns the stable `snake_case` string used in report JSON.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::IntentionalChange => "intentional_change",
            Self::OldBugPreserved => "old_bug_preserved",
            Self::OldBugFixed => "old_bug_fixed",
            Self::NewBug => "new_bug",
            Self::InsufficientData => "insufficient_data",
            Self::HumanReview => "human_review",
        }
    }
}

impl Display for MismatchCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Impact value for a downstream report dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactLevel {
    /// The difference affects this dimension.
    Yes,
    /// The difference does not affect this dimension.
    No,
    /// Impact cannot be determined from the available evidence.
    Unknown,
}

/// Downstream impact dimensions required by Phase 1 D-14 and Phase 5 D-07.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactAssessment {
    /// Whether normalized parser artifacts or parser evidence differ.
    pub parser_artifact: ImpactLevel,
    /// Whether `server-2` persistence could store different parser-derived data.
    pub server_2_persistence: ImpactLevel,
    /// Whether `server-2` recalculation of derived stats could change.
    pub server_2_recalculation: ImpactLevel,
    /// Whether `web` could show different public stats through `server-2`.
    pub ui_visible_public_stats: ImpactLevel,
}

impl ImpactAssessment {
    /// Builds a complete impact assessment.
    #[must_use]
    pub const fn new(
        parser_artifact: ImpactLevel,
        server_2_persistence: ImpactLevel,
        server_2_recalculation: ImpactLevel,
        ui_visible_public_stats: ImpactLevel,
    ) -> Self {
        Self {
            parser_artifact,
            server_2_persistence,
            server_2_recalculation,
            ui_visible_public_stats,
        }
    }

    /// Builds an impact assessment and rejects omitted dimensions.
    ///
    /// # Errors
    ///
    /// Returns [`ReportValidationError::MissingImpactDimension`] when any
    /// required dimension is absent.
    pub fn try_new(
        parser_artifact: Option<ImpactLevel>,
        server_2_persistence: Option<ImpactLevel>,
        server_2_recalculation: Option<ImpactLevel>,
        ui_visible_public_stats: Option<ImpactLevel>,
    ) -> Result<Self, ReportValidationError> {
        Ok(Self::new(
            parser_artifact.ok_or(ReportValidationError::MissingImpactDimension {
                dimension: "parser_artifact",
            })?,
            server_2_persistence.ok_or(ReportValidationError::MissingImpactDimension {
                dimension: "server_2_persistence",
            })?,
            server_2_recalculation.ok_or(ReportValidationError::MissingImpactDimension {
                dimension: "server_2_recalculation",
            })?,
            ui_visible_public_stats.ok_or(ReportValidationError::MissingImpactDimension {
                dimension: "ui_visible_public_stats",
            })?,
        ))
    }

    /// Returns a conservative unknown impact assessment.
    #[must_use]
    pub const fn unknown() -> Self {
        Self::new(
            ImpactLevel::Unknown,
            ImpactLevel::Unknown,
            ImpactLevel::Unknown,
            ImpactLevel::Unknown,
        )
    }
}

/// Baseline evidence used for old-side comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonBaseline {
    /// Human-readable old-parser profile name.
    pub old_profile: String,
    /// Old-parser command or saved-artifact source used as evidence.
    pub old_command: String,
    /// Worker count for regenerated old-parser output when known.
    pub worker_count: Option<u16>,
    /// Baseline source label, such as deterministic old output or current/regenerated drift.
    pub source: String,
    /// Whether this baseline is diagnostic-only rather than primary semantic evidence.
    pub diagnostic_only: bool,
}

impl ComparisonBaseline {
    /// Builds baseline metadata for selected saved artifacts.
    #[must_use]
    pub fn saved_artifact(old_label: impl Into<String>) -> Self {
        let old_label = old_label.into();
        Self {
            old_profile: old_label.clone(),
            old_command: format!("saved artifact: {old_label}"),
            worker_count: None,
            source: "selected_saved_artifact".to_owned(),
            diagnostic_only: false,
        }
    }

    /// Returns true for unexplained current-vs-regenerated old-baseline drift.
    #[must_use]
    pub fn is_current_vs_regenerated_drift(&self) -> bool {
        let source = self.source.to_ascii_lowercase();
        source.contains("current") && source.contains("regenerated") && source.contains("drift")
    }
}

/// Input document recorded in a comparison report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonInput {
    /// Stable input label used by report findings.
    pub label: String,
    /// Input source description or path.
    pub source: String,
}

impl ComparisonInput {
    /// Builds a comparison input descriptor.
    #[must_use]
    pub fn new(label: impl Into<String>, source: impl Into<String>) -> Self {
        Self { label: label.into(), source: source.into() }
    }
}

/// A single surface or field-level comparison finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonFinding {
    /// Compared surface, such as `status`, `replay`, or an aggregate projection key.
    pub surface: String,
    /// Optional nested field within the surface.
    pub field: Option<String>,
    /// Mismatch category assigned to this finding.
    pub category: MismatchCategory,
    /// Required downstream impact assessment.
    pub impact: ImpactAssessment,
    /// Old-side JSON value observed for this surface.
    pub old_value: Value,
    /// New-side JSON value observed for this surface.
    pub new_value: Value,
    /// Source references or evidence paths relevant to this finding.
    pub source_refs: Vec<String>,
    /// Human-readable notes for reviewers.
    pub notes: Vec<String>,
}

impl ComparisonFinding {
    /// Builds a comparison finding with an explicit impact assessment.
    #[must_use]
    pub fn new(
        surface: impl Into<String>,
        field: Option<String>,
        category: MismatchCategory,
        impact: ImpactAssessment,
        old_value: Value,
        new_value: Value,
    ) -> Self {
        Self {
            surface: surface.into(),
            field,
            category,
            impact,
            old_value,
            new_value,
            source_refs: Vec::new(),
            notes: Vec::new(),
        }
    }
}

/// Aggregate report summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonSummary {
    /// Number of findings in the report.
    pub total_findings: usize,
    /// Finding counts keyed by mismatch category string.
    pub by_category: BTreeMap<String, usize>,
}

impl ComparisonSummary {
    /// Builds summary counts from report findings.
    #[must_use]
    pub fn from_findings(findings: &[ComparisonFinding]) -> Self {
        let mut by_category = BTreeMap::new();
        for finding in findings {
            let count = by_category.entry(finding.category.as_str().to_owned()).or_insert(0);
            *count += 1;
        }

        Self { total_findings: findings.len(), by_category }
    }
}

/// Full old-vs-new comparison report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// Report schema version.
    pub report_version: String,
    /// Old-side baseline metadata.
    pub baseline: ComparisonBaseline,
    /// Input documents used for the comparison.
    pub inputs: Vec<ComparisonInput>,
    /// Surface and field-level findings.
    pub findings: Vec<ComparisonFinding>,
    /// Aggregate finding counts.
    pub summary: ComparisonSummary,
}

impl ComparisonReport {
    /// Builds and validates a comparison report.
    ///
    /// # Errors
    ///
    /// Returns [`ReportValidationError::BaselineDriftMustRemainHumanReview`] when
    /// current-vs-regenerated old-baseline drift is classified as anything other
    /// than [`MismatchCategory::HumanReview`].
    pub fn new(
        baseline: ComparisonBaseline,
        inputs: Vec<ComparisonInput>,
        findings: Vec<ComparisonFinding>,
    ) -> Result<Self, ReportValidationError> {
        let report = Self {
            report_version: COMPARISON_REPORT_VERSION.to_owned(),
            baseline,
            summary: ComparisonSummary::from_findings(&findings),
            inputs,
            findings,
        };
        report.validate()?;
        Ok(report)
    }

    /// Validates report invariants required by the mismatch taxonomy.
    ///
    /// # Errors
    ///
    /// Returns [`ReportValidationError::BaselineDriftMustRemainHumanReview`] when
    /// unexplained current-vs-regenerated drift is not held for human review.
    pub fn validate(&self) -> Result<(), ReportValidationError> {
        if self.baseline.is_current_vs_regenerated_drift() {
            for finding in &self.findings {
                if finding.category != MismatchCategory::HumanReview {
                    return Err(ReportValidationError::BaselineDriftMustRemainHumanReview {
                        surface: finding.surface.clone(),
                        category: finding.category,
                    });
                }
            }
        }

        Ok(())
    }
}

/// Validation failures for comparison report construction.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReportValidationError {
    /// A required impact dimension was not provided.
    #[error("impact dimension is missing: {dimension}")]
    MissingImpactDimension {
        /// Missing impact dimension field name.
        dimension: &'static str,
    },
    /// Unexplained current-vs-regenerated old-baseline drift was classified too eagerly.
    #[error(
        "current-vs-regenerated drift finding `{surface}` must be human_review, got {category}"
    )]
    BaselineDriftMustRemainHumanReview {
        /// Surface whose finding violated the drift gate.
        surface: String,
        /// Category assigned before required human review.
        category: MismatchCategory,
    },
}
