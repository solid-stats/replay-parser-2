use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::source_ref::SourceRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub json_path: Option<String>,
    pub expected_shape: Option<String>,
    pub observed_shape: Option<String>,
    pub parser_action: String,
    pub source_refs: Vec<SourceRef>,
}
