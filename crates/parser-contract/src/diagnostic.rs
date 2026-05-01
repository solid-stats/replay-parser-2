use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::source_ref::SourceRefs;

/// Severity of a structured parser diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Informational diagnostic.
    Info,
    /// Warning diagnostic that did not necessarily fail parsing.
    Warning,
    /// Error diagnostic for failed or partially failed parsing.
    Error,
}

/// Structured warning or error emitted during parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// Human-readable diagnostic message.
    pub message: String,
    /// JSON path to the replay source field, when available.
    pub json_path: Option<String>,
    /// Expected source shape, when the diagnostic concerns schema drift.
    pub expected_shape: Option<String>,
    /// Observed source shape, when the diagnostic concerns schema drift.
    pub observed_shape: Option<String>,
    /// Parser action taken in response to the issue.
    pub parser_action: String,
    /// Source evidence backing the diagnostic.
    pub source_refs: SourceRefs,
}

/// Minimal diagnostic row emitted by default parser artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalDiagnosticRow {
    /// Stable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Parser action taken in response to the issue.
    pub parser_action: String,
}

impl From<Diagnostic> for MinimalDiagnosticRow {
    fn from(diagnostic: Diagnostic) -> Self {
        Self {
            code: diagnostic.code,
            severity: diagnostic.severity,
            message: diagnostic.message,
            parser_action: diagnostic.parser_action,
        }
    }
}
