//! Selected-input old-vs-new comparison logic.
// coverage-exclusion: reviewed Phase 05 comparison fallback branches are allowlisted by exact source line.

use serde_json::Value;

use crate::report::{
    ComparisonBaseline, ComparisonFinding, ComparisonInput, ComparisonReport, ImpactAssessment,
    ImpactLevel, MismatchCategory, ReportValidationError,
};

/// Compares two selected saved JSON artifacts and returns a structured report.
///
/// # Errors
///
/// Returns [`ComparisonError::InvalidJson`] when either input is not JSON, or
/// [`ComparisonError::InvalidReport`] when report invariants are violated.
pub fn compare_artifacts(
    old_label: impl Into<String>,
    old_json: &[u8],
    new_label: impl Into<String>,
    new_json: &[u8],
) -> Result<ComparisonReport, ComparisonError> {
    let old_label = old_label.into();
    let new_label = new_label.into();
    let old_value = parse_json("old", old_json)?;
    let new_value = parse_json("new", new_json)?;
    compare_values(old_label, &old_value, new_label, &new_value)
}

fn compare_values(
    old_label: String,
    old_value: &Value,
    new_label: String,
    new_value: &Value,
) -> Result<ComparisonReport, ComparisonError> {
    let baseline = baseline_from_old_label(&old_label);
    let baseline_is_drift = baseline.is_current_vs_regenerated_drift();
    let findings = selected_surfaces()
        .iter()
        .map(|surface| compare_surface(surface, old_value, new_value, baseline_is_drift))
        .collect();

    ComparisonReport::new(
        baseline,
        vec![ComparisonInput::new("old", old_label), ComparisonInput::new("new", new_label)],
        findings,
    )
    .map_err(ComparisonError::InvalidReport)
}

fn parse_json(side: &'static str, bytes: &[u8]) -> Result<Value, ComparisonError> {
    serde_json::from_slice(bytes).map_err(|source| ComparisonError::InvalidJson { side, source })
}

fn baseline_from_old_label(old_label: &str) -> ComparisonBaseline {
    if labels_current_vs_regenerated_drift(old_label) {
        ComparisonBaseline {
            old_profile: old_label.to_owned(),
            old_command: format!("saved artifact: {old_label}"),
            worker_count: Some(1),
            source: "current_vs_regenerated_drift".to_owned(),
            diagnostic_only: false,
        }
    } else {
        ComparisonBaseline::saved_artifact(old_label)
    }
}

fn labels_current_vs_regenerated_drift(label: &str) -> bool {
    let label = label.to_ascii_lowercase();
    label.contains("current") && label.contains("regenerated") && label.contains("drift")
}

const fn selected_surfaces() -> [SelectedSurface; 7] {
    [
        SelectedSurface::top_level("status"),
        SelectedSurface::top_level("replay"),
        SelectedSurface::top_level("events"),
        SelectedSurface::projection("legacy.player_game_results"),
        SelectedSurface::projection("legacy.relationships"),
        SelectedSurface::projection("bounty.inputs"),
        SelectedSurface::projection("vehicle_score.inputs"),
    ]
}

fn compare_surface(
    surface: &SelectedSurface,
    old_root: &Value,
    new_root: &Value,
    baseline_is_drift: bool,
) -> ComparisonFinding {
    let old_value = surface.extract(old_root);
    let new_value = surface.extract(new_root);
    let category = classify_values(old_value, new_value, baseline_is_drift);

    ComparisonFinding::new(
        surface.name,
        None,
        category,
        impact_for_surface(surface),
        old_value.cloned().unwrap_or(Value::Null),
        new_value.cloned().unwrap_or(Value::Null),
    )
}

fn classify_values(
    old_value: Option<&Value>,
    new_value: Option<&Value>,
    baseline_is_drift: bool,
) -> MismatchCategory {
    if baseline_is_drift {
        return MismatchCategory::HumanReview;
    }

    match (old_value, new_value) {
        (Some(old), Some(new)) if old == new => MismatchCategory::Compatible,
        (Some(_), Some(_)) => MismatchCategory::HumanReview,
        _ => MismatchCategory::InsufficientData,
    }
}

const fn impact_for_surface(surface: &SelectedSurface) -> ImpactAssessment {
    if surface.is_projection {
        return ImpactAssessment::new(
            ImpactLevel::Yes,
            ImpactLevel::Unknown,
            ImpactLevel::Unknown,
            ImpactLevel::Unknown,
        );
    }

    ImpactAssessment::new(
        ImpactLevel::Yes,
        ImpactLevel::Unknown,
        ImpactLevel::Unknown,
        ImpactLevel::Unknown,
    )
}

#[derive(Debug, Clone, Copy)]
struct SelectedSurface {
    name: &'static str,
    is_projection: bool,
}

impl SelectedSurface {
    const fn top_level(name: &'static str) -> Self {
        Self { name, is_projection: false }
    }

    const fn projection(name: &'static str) -> Self {
        Self { name, is_projection: true }
    }

    fn extract<'a>(&self, root: &'a Value) -> Option<&'a Value> {
        if self.is_projection {
            return root.get("aggregates")?.get("projections")?.get(self.name);
        }

        root.get(self.name)
    }
}

/// Comparison harness failures.
#[derive(Debug, thiserror::Error)]
pub enum ComparisonError {
    /// One side of the comparison was not valid JSON.
    #[error("{side} artifact is not valid JSON: {source}")]
    InvalidJson {
        /// Compared side label.
        side: &'static str,
        /// JSON parser error.
        source: serde_json::Error,
    },
    /// The produced report violated report invariants.
    #[error(transparent)]
    InvalidReport(#[from] ReportValidationError),
}
