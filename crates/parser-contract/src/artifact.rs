use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    diagnostic::MinimalDiagnosticRow,
    failure::ParseFailure,
    metadata::ReplayMetadata,
    minimal::{MinimalDestroyedVehicleRow, MinimalKillRow, MinimalPlayerRow, MinimalWeaponRow},
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub produced_at: Option<String>,
    /// Structured warnings and errors that did not necessarily fail parsing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<MinimalDiagnosticRow>,
    /// Normalized replay metadata when it could be extracted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay: Option<ReplayMetadata>,
    /// Minimal observed player rows from the replay.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<MinimalPlayerRow>,
    /// Minimal deterministic weapon dictionary rows referenced by event tables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weapons: Vec<MinimalWeaponRow>,
    /// Minimal player death rows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kills: Vec<MinimalKillRow>,
    /// Minimal vehicle and static weapon destruction rows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destroyed_vehicles: Vec<MinimalDestroyedVehicleRow>,
    /// Replay-side commander and outcome facts.
    #[serde(default, skip_serializing_if = "ReplaySideFacts::is_default")]
    pub side_facts: ReplaySideFacts,
    /// Structured failure details required when status is `failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<ParseFailure>,
    /// Stable extension object reserved for forward-compatible parser metadata.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
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
