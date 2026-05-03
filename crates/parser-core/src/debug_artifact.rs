//! Deterministic debug-side artifact construction.
// coverage-exclusion: reviewed v1.0 debug-sidecar defensive serialization regions are allowlisted by exact source line.

use parser_contract::{
    diagnostic::Diagnostic,
    events::NormalizedEvent,
    identity::ObservedEntity,
    metadata::ReplayMetadata,
    side_facts::ReplaySideFacts,
    source_ref::ReplaySource,
    version::{ContractVersion, ParserInfo},
};
use serde::{Deserialize, Serialize};

use crate::{
    artifact::SourceContext,
    diagnostics::{DiagnosticAccumulator, DiagnosticPolicy},
    entities::normalize_entities,
    events::normalize_combat_events,
    input::ParserInput,
    metadata::normalize_metadata,
    raw::RawReplay,
    raw_compact::decode_compact_root,
    side_facts::normalize_side_facts,
};

/// Full deterministic parser-side artifact used for audits and debugging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugParseArtifact {
    /// Parser contract version used by normalized payload types.
    pub contract_version: ContractVersion,
    /// Parser binary and build metadata.
    pub parser: ParserInfo,
    /// Replay source identity and checksum metadata.
    pub source: ReplaySource,
    /// Normalized replay metadata when the replay root decoded.
    pub replay: Option<ReplayMetadata>,
    /// Full observed entity records with provenance and compatibility hints.
    pub entities: Vec<ObservedEntity>,
    /// Full normalized event records with source references and rule IDs.
    pub events: Vec<NormalizedEvent>,
    /// Replay-side commander and outcome facts with source-backed evidence.
    pub side_facts: ReplaySideFacts,
    /// Structured diagnostics emitted during normalization.
    pub diagnostics: Vec<Diagnostic>,
}

/// Parses replay bytes into the full deterministic parser-side debug artifact.
#[must_use]
pub fn parse_replay_debug(input: ParserInput<'_>) -> DebugParseArtifact {
    let diagnostic_limit = DiagnosticPolicy::from(input.options).limit();

    match decode_compact_root(input.bytes) {
        Ok(root) => {
            let raw = RawReplay::new(&root);
            let context = SourceContext::new(&input.source);
            let mut diagnostics = DiagnosticAccumulator::new(diagnostic_limit);
            let replay = normalize_metadata(&raw, &context, &mut diagnostics);
            let entities = normalize_entities(&raw, &context, &mut diagnostics);
            let events = normalize_combat_events(&raw, &entities, &context, &mut diagnostics);
            let side_facts = normalize_side_facts(&raw, &entities, &context, &mut diagnostics);
            let diagnostic_report = diagnostics.finish(&context);

            DebugParseArtifact {
                contract_version: ContractVersion::current(),
                parser: input.parser,
                source: input.source,
                replay: Some(replay),
                entities,
                events,
                side_facts,
                diagnostics: diagnostic_report.diagnostics,
            }
        }
        Err(_) => DebugParseArtifact {
            contract_version: ContractVersion::current(),
            parser: input.parser,
            source: input.source,
            replay: None,
            entities: Vec::new(),
            events: Vec::new(),
            side_facts: ReplaySideFacts::default(),
            diagnostics: Vec::new(),
        },
    }
}
