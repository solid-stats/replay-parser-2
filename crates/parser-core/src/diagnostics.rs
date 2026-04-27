//! Diagnostic policy helpers.

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
