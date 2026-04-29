use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    compact::{ObservedParticipantRef, ParseFactSection, ParseSummarySection},
    diagnostic::Diagnostic,
    failure::ParseFailure,
    metadata::ReplayMetadata,
    side_facts::ReplaySideFacts,
    source_ref::ReplaySource,
    version::{ContractVersion, ParserInfo},
};

/// High-level parse outcome recorded in a parser artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseStatus {
    /// The parser completed without semantic data loss.
    Success,
    /// The parser completed with warnings or recoverable gaps.
    Partial,
    /// The parser intentionally skipped this replay.
    Skipped,
    /// The parser failed before producing a usable artifact.
    Failed,
}

/// Versioned parser output consumed by downstream Solid Stats services.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ParseArtifact {
    /// Parser contract version used to shape this artifact.
    pub contract_version: ContractVersion,
    /// Parser binary and build metadata.
    pub parser: ParserInfo,
    /// Replay source identity and checksum metadata.
    pub source: ReplaySource,
    /// Overall parse status.
    pub status: ParseStatus,
    /// Optional production timestamp supplied by non-deterministic adapters.
    pub produced_at: Option<String>,
    /// Structured warnings and errors that did not necessarily fail parsing.
    pub diagnostics: Vec<Diagnostic>,
    /// Normalized replay metadata when it could be extracted.
    pub replay: Option<ReplayMetadata>,
    /// Compact observed participant references from the replay.
    #[serde(default)]
    pub participants: Vec<ObservedParticipantRef>,
    /// Compact facts and auditable contribution references.
    #[serde(default)]
    pub facts: ParseFactSection,
    /// Compact summary projections for review and ingestion sanity checks.
    #[serde(default)]
    pub summaries: ParseSummarySection,
    /// Replay-side commander and outcome facts.
    pub side_facts: ReplaySideFacts,
    /// Structured failure details required when status is `failed`.
    pub failure: Option<ParseFailure>,
    /// Stable extension object reserved for forward-compatible parser metadata.
    pub extensions: BTreeMap<String, Value>,
}

impl ParseArtifact {
    /// Validates the relationship between `status` and `failure`.
    ///
    /// # Errors
    ///
    /// Returns [`ParseArtifactError::MissingFailure`] when a failed artifact has no failure
    /// payload, and [`ParseArtifactError::UnexpectedFailure`] when a non-failed artifact has
    /// failure details.
    pub const fn validate_status_payload(&self) -> Result<(), ParseArtifactError> {
        match (self.status, self.failure.as_ref()) {
            (ParseStatus::Failed, Some(_)) => Ok(()),
            (ParseStatus::Failed, None) => Err(ParseArtifactError::MissingFailure),
            (_, Some(_)) => Err(ParseArtifactError::UnexpectedFailure),
            _ => Ok(()),
        }
    }
}

/// Error returned when a parse artifact envelope violates status invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ParseArtifactError {
    /// Failed artifacts must include structured failure details.
    #[error("failed parse artifacts must include structured failure details")]
    MissingFailure,
    /// Non-failed artifacts must not include structured failure details.
    #[error("non-failed parse artifacts must not include structured failure details")]
    UnexpectedFailure,
}
