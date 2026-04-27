//! Diagnostic policy and accumulator helpers.

use parser_contract::diagnostic::Diagnostic;

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

/// Capped accumulator for parser diagnostics emitted during normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticAccumulator {
    diagnostics: Vec<Diagnostic>,
    limit: usize,
    has_data_loss: bool,
}

impl DiagnosticAccumulator {
    /// Creates an empty accumulator with the provided diagnostic limit.
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self { diagnostics: Vec::new(), limit, has_data_loss: false }
    }

    /// Pushes a diagnostic if the accumulator has not reached its limit.
    pub fn push(&mut self, diagnostic: Diagnostic, indicates_data_loss: bool) {
        self.has_data_loss |= indicates_data_loss;

        if self.diagnostics.len() < self.limit {
            self.diagnostics.push(diagnostic);
        }
    }

    /// Returns true when any recorded issue caused data loss or an explicit drift unknown.
    #[must_use]
    pub const fn has_data_loss(&self) -> bool {
        self.has_data_loss
    }

    /// Consumes the accumulator and returns emitted diagnostics in insertion order.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
