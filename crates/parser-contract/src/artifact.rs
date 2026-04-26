use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    aggregates::AggregateSection,
    diagnostic::Diagnostic,
    events::NormalizedEvent,
    failure::ParseFailure,
    identity::ObservedEntity,
    metadata::ReplayMetadata,
    source_ref::ReplaySource,
    version::{ContractVersion, ParserInfo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseStatus {
    Success,
    Partial,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ParseArtifact {
    pub contract_version: ContractVersion,
    pub parser: ParserInfo,
    pub source: ReplaySource,
    pub status: ParseStatus,
    pub produced_at: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub replay: Option<ReplayMetadata>,
    pub entities: Vec<ObservedEntity>,
    pub events: Vec<NormalizedEvent>,
    pub aggregates: AggregateSection,
    pub failure: Option<ParseFailure>,
    pub extensions: BTreeMap<String, Value>,
}

impl ParseArtifact {
    pub fn validate_status_payload(&self) -> Result<(), ParseArtifactError> {
        match (self.status, self.failure.as_ref()) {
            (ParseStatus::Failed, Some(_)) => Ok(()),
            (ParseStatus::Failed, None) => Err(ParseArtifactError::MissingFailure),
            (_, Some(_)) => Err(ParseArtifactError::UnexpectedFailure),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseArtifactError {
    #[error("failed parse artifacts must include structured failure details")]
    MissingFailure,
    #[error("non-failed parse artifacts must not include structured failure details")]
    UnexpectedFailure,
}
