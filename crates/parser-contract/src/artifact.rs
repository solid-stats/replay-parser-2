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
