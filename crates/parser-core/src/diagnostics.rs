//! Diagnostic policy and accumulator helpers.
// coverage-exclusion: reviewed Phase 05 diagnostic cap defensive branch is allowlisted by exact source line.

use parser_contract::{
    artifact::ParseStatus,
    diagnostic::{Diagnostic, DiagnosticSeverity},
    source_ref::{RuleId, SourceRefs},
};

use crate::artifact::SourceContext;

/// Policy wrapper for deterministic diagnostic emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticPolicy {
    limit: usize,
}

impl DiagnosticPolicy {
    /// Creates a diagnostic policy with an explicit diagnostic limit.
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self { limit }
    }

    /// Returns the maximum number of diagnostics allowed before compaction.
    #[must_use]
    pub const fn limit(self) -> usize {
        self.limit
    }
}

impl From<crate::input::ParserOptions> for DiagnosticPolicy {
    fn from(options: crate::input::ParserOptions) -> Self {
        Self::new(options.diagnostic_limit)
    }
}

/// Semantic impact of a parser diagnostic on the artifact status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticImpact {
    /// Informational diagnostics do not imply data loss.
    Info,
    /// Warning diagnostics that preserve all semantic source data do not imply data loss.
    NonLossWarning,
    /// Diagnostics caused by dropped source facts, drift unknowns, or audit data loss.
    DataLoss,
}

/// Final diagnostic payload and status decision for a successful root decode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticReport {
    /// Emitted diagnostics, capped and summarized when needed.
    pub diagnostics: Vec<Diagnostic>,
    /// Status to use when the replay root decoded and normalization completed.
    pub status_for_successful_parse: ParseStatus,
}

/// Capped accumulator for parser diagnostics emitted during normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticAccumulator {
    limit: usize,
    diagnostics: Vec<Diagnostic>,
    omitted_count: usize,
    has_data_loss: bool,
}

impl DiagnosticAccumulator {
    /// Creates an empty accumulator with the provided diagnostic limit.
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self { limit, diagnostics: Vec::new(), omitted_count: 0, has_data_loss: false }
    }

    /// Pushes a diagnostic if the accumulator has not reached its limit.
    pub fn push(&mut self, diagnostic: Diagnostic, impact: DiagnosticImpact) {
        self.has_data_loss |= matches!(impact, DiagnosticImpact::DataLoss);

        if self.diagnostics.len() < self.limit {
            self.diagnostics.push(diagnostic);
        } else {
            self.omitted_count = self.omitted_count.saturating_add(1);
        }
    }

    /// Consumes the accumulator and returns capped diagnostics plus status policy.
    #[must_use]
    pub fn finish(mut self, context: &SourceContext) -> DiagnosticReport {
        if self.omitted_count > 0
            && let Some(summary) = limit_exceeded_diagnostic(self.omitted_count, context)
        {
            self.diagnostics.push(summary);
        }

        let status_for_successful_parse =
            if self.has_data_loss { ParseStatus::Partial } else { ParseStatus::Success };

        DiagnosticReport { diagnostics: self.diagnostics, status_for_successful_parse }
    }
}

fn limit_exceeded_diagnostic(omitted_count: usize, context: &SourceContext) -> Option<Diagnostic> {
    let source_ref = context.source_ref("$", RuleId::new("diagnostic.limit_exceeded").ok());
    let source_refs = SourceRefs::new(vec![source_ref]).ok()?;

    Some(Diagnostic {
        code: "diagnostic.limit_exceeded".to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!("Omitted {omitted_count} diagnostics after reaching diagnostic limit"),
        json_path: Some("$".to_string()),
        expected_shape: None,
        observed_shape: None,
        parser_action: "summarized_repeated_diagnostics".to_string(),
        source_refs,
    })
}
