//! Parse artifact construction.
// coverage-exclusion: reviewed Phase 05 defensive artifact construction branches are allowlisted by exact source line.

use std::collections::BTreeMap;

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    compact::{CombatFact, ObservedParticipantRef, ParseFactSection, ParseSummarySection},
    failure::{ErrorCode, ErrorCodeError, ParseFailure, ParseStage, Retryability},
    identity::ObservedEntity,
    presence::{FieldPresence, UnknownReason},
    side_facts::ReplaySideFacts,
    source_ref::{
        ReplaySource, RuleId, RuleIdError, SourceChecksum, SourceRef, SourceRefs, SourceRefsError,
    },
    version::ContractVersion,
};

use crate::{
    aggregates::derive_aggregate_section,
    diagnostics::{DiagnosticAccumulator, DiagnosticPolicy},
    entities::normalize_entities,
    events::normalize_combat_events,
    input::ParserInput,
    metadata::normalize_metadata,
    raw::RawReplay,
    raw_compact::{CompactDecodeError, RawOcapRoot, decode_compact_root},
    side_facts::normalize_side_facts,
};

/// Parses replay bytes into a deterministic artifact shell.
#[must_use]
pub fn parse_replay(input: ParserInput<'_>) -> ParseArtifact {
    let diagnostic_limit = DiagnosticPolicy::from(input.options).limit();

    match decode_compact_root(input.bytes) {
        Ok(root) => success_artifact(input.parser, input.source, &root, diagnostic_limit),
        Err(CompactDecodeError::RootNotObject { source_cause }) => failed_artifact(
            input.parser,
            input.source,
            &FailureSpec::RootNotObject { source_cause },
        ),
        Err(CompactDecodeError::JsonDecode { source_cause }) => {
            failed_artifact(input.parser, input.source, &FailureSpec::JsonDecode { source_cause })
        }
    }
}

fn success_artifact(
    parser: parser_contract::version::ParserInfo,
    source: ReplaySource,
    root: &RawOcapRoot<'_>,
    diagnostic_limit: usize,
) -> ParseArtifact {
    let raw = RawReplay::new(root);
    let context = SourceContext::new(&source);
    let mut diagnostics = DiagnosticAccumulator::new(diagnostic_limit);
    let replay = normalize_metadata(&raw, &context, &mut diagnostics);
    let entities = normalize_entities(&raw, &context, &mut diagnostics);
    let events = normalize_combat_events(&raw, &entities, &context, &mut diagnostics);
    let side_facts = normalize_side_facts(&raw, &entities, &context, &mut diagnostics);
    let aggregates = derive_aggregate_section(&replay, &entities, &events, &context);
    let diagnostic_report = diagnostics.finish(&context);
    let participants = compact_participants(&entities);
    let facts = ParseFactSection {
        combat: events
            .iter()
            .filter_map(|event| {
                let combat = event.combat.as_ref()?;
                Some(CombatFact {
                    fact_id: event.event_id.clone(),
                    semantic: combat.semantic,
                    frame: event.frame.clone(),
                    event_index: event.event_index.clone(),
                    killer: combat.killer.clone(),
                    victim: combat.victim.clone(),
                    victim_kind: combat.victim_kind,
                    vehicle_context: combat.vehicle_context.clone(),
                    bounty: combat.bounty.clone(),
                    legacy_counter_effects: combat.legacy_counter_effects.clone(),
                    source_refs: event.source_refs.clone(),
                    rule_id: event.rule_id.clone(),
                })
            })
            .collect(),
        aggregate_contributions: aggregates.contributions,
    };
    let summaries = ParseSummarySection { projections: aggregates.projections };

    ParseArtifact {
        contract_version: ContractVersion::current(),
        parser,
        source,
        status: diagnostic_report.status_for_successful_parse,
        produced_at: None,
        diagnostics: diagnostic_report.diagnostics,
        replay: Some(replay),
        participants,
        facts,
        summaries,
        side_facts,
        failure: None,
        extensions: BTreeMap::new(),
    }
}

fn failed_artifact(
    parser: parser_contract::version::ParserInfo,
    source: ReplaySource,
    spec: &FailureSpec,
) -> ParseArtifact {
    let failure = parse_failure(&source, spec).ok();

    ParseArtifact {
        contract_version: ContractVersion::current(),
        parser,
        source,
        status: ParseStatus::Failed,
        produced_at: None,
        diagnostics: Vec::new(),
        replay: None,
        participants: Vec::new(),
        facts: ParseFactSection::default(),
        summaries: ParseSummarySection::default(),
        side_facts: ReplaySideFacts::default(),
        failure,
        extensions: BTreeMap::new(),
    }
}

fn compact_participants(entities: &[ObservedEntity]) -> Vec<ObservedParticipantRef> {
    entities
        .iter()
        .map(|entity| ObservedParticipantRef {
            source_entity_id: entity.source_entity_id,
            observed_name: participant_name(entity),
            side: entity.identity.side.clone(),
            group: participant_group(entity),
            role: participant_role(entity),
            steam_id: entity.identity.steam_id.clone(),
            source_refs: entity.source_refs.clone(),
        })
        .collect()
}

fn participant_name(entity: &ObservedEntity) -> FieldPresence<String> {
    match &entity.identity.nickname {
        FieldPresence::Present { .. } | FieldPresence::Inferred { .. } => {
            entity.identity.nickname.clone()
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => entity.observed_name.clone(),
    }
}

fn participant_group(entity: &ObservedEntity) -> FieldPresence<String> {
    match &entity.identity.squad {
        FieldPresence::Present { .. } | FieldPresence::Inferred { .. } => {
            entity.identity.squad.clone()
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => entity.identity.group.clone(),
    }
}

fn participant_role(entity: &ObservedEntity) -> FieldPresence<String> {
    match &entity.identity.role {
        FieldPresence::Present { .. } | FieldPresence::Inferred { .. } => {
            entity.identity.role.clone()
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => entity.identity.description.clone(),
    }
}

fn parse_failure(
    source: &ReplaySource,
    spec: &FailureSpec,
) -> Result<ParseFailure, FailureBuildError> {
    let failure_source_ref = failure_source_ref(source, spec.rule_id()?);

    Ok(ParseFailure {
        job_id: FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: None },
        replay_id: replay_id_presence(source),
        source_file: FieldPresence::Present { value: source.source_file.clone(), source: None },
        checksum: source.checksum.clone(),
        stage: spec.stage(),
        error_code: spec.error_code()?,
        message: spec.message().to_string(),
        retryability: Retryability::NotRetryable,
        source_cause: FieldPresence::Present {
            value: spec.source_cause().to_string(),
            source: None,
        },
        source_refs: SourceRefs::new(vec![failure_source_ref])?,
    })
}

fn replay_id_presence(source: &ReplaySource) -> FieldPresence<String> {
    source.replay_id.as_ref().map_or(
        FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: None },
        |replay_id| FieldPresence::Present { value: replay_id.clone(), source: None },
    )
}

fn failure_source_ref(source: &ReplaySource, rule_id: RuleId) -> SourceRef {
    SourceRef {
        replay_id: source.replay_id.clone(),
        source_file: Some(source.source_file.clone()),
        checksum: present_checksum(source),
        frame: None,
        event_index: None,
        entity_id: None,
        json_path: Some("$".to_string()),
        rule_id: Some(rule_id),
    }
}

fn present_checksum(source: &ReplaySource) -> Option<SourceChecksum> {
    match &source.checksum {
        FieldPresence::Present { value, source: _ } => Some(value.clone()),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

/// Source coordinates copied from caller-provided replay source metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceContext {
    replay_id: Option<String>,
    source_file: String,
    checksum: Option<SourceChecksum>,
}

impl SourceContext {
    /// Creates source context from the artifact replay source.
    #[must_use]
    pub fn new(source: &ReplaySource) -> Self {
        Self {
            replay_id: source.replay_id.clone(),
            source_file: source.source_file.clone(),
            checksum: present_checksum(source),
        }
    }

    /// Builds a source reference at the supplied JSON path and rule identifier.
    #[must_use]
    pub fn source_ref(&self, json_path: &str, rule_id: Option<RuleId>) -> SourceRef {
        SourceRef {
            replay_id: self.replay_id.clone(),
            source_file: Some(self.source_file.clone()),
            checksum: self.checksum.clone(),
            frame: None,
            event_index: None,
            entity_id: None,
            json_path: Some(json_path.to_string()),
            rule_id,
        }
    }

    /// Builds a source reference for an event tuple with stable event coordinates.
    #[must_use]
    pub fn event_source_ref(
        &self,
        json_path: &str,
        frame: Option<u64>,
        event_index: Option<u64>,
        entity_id: Option<i64>,
        rule_id: Option<RuleId>,
    ) -> SourceRef {
        SourceRef {
            replay_id: self.replay_id.clone(),
            source_file: Some(self.source_file.clone()),
            checksum: self.checksum.clone(),
            frame,
            event_index,
            entity_id,
            json_path: Some(json_path.to_string()),
            rule_id,
        }
    }
}

enum FailureSpec {
    JsonDecode { source_cause: String },
    RootNotObject { source_cause: String },
}

impl FailureSpec {
    const fn stage(&self) -> ParseStage {
        match self {
            Self::JsonDecode { .. } => ParseStage::JsonDecode,
            Self::RootNotObject { .. } => ParseStage::Schema,
        }
    }

    fn error_code(&self) -> Result<ErrorCode, ErrorCodeError> {
        match self {
            Self::JsonDecode { .. } => ErrorCode::new("json.decode"),
            Self::RootNotObject { .. } => ErrorCode::new("schema.root_object"),
        }
    }

    fn rule_id(&self) -> Result<RuleId, RuleIdError> {
        match self {
            Self::JsonDecode { .. } => RuleId::new("failure.json.decode"),
            Self::RootNotObject { .. } => RuleId::new("failure.schema.root_object"),
        }
    }

    const fn message(&self) -> &'static str {
        match self {
            Self::JsonDecode { .. } => "Replay JSON could not be decoded",
            Self::RootNotObject { .. } => "OCAP replay root must be a JSON object",
        }
    }

    fn source_cause(&self) -> &str {
        match self {
            Self::JsonDecode { source_cause } | Self::RootNotObject { source_cause } => {
                source_cause
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum FailureBuildError {
    #[error("invalid parser-core failure error code")]
    ErrorCode(#[from] ErrorCodeError),
    #[error("invalid parser-core failure rule ID")]
    RuleId(#[from] RuleIdError),
    #[error("parser-core failure source refs must be non-empty")]
    SourceRefs(#[from] SourceRefsError),
}
