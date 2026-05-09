//! Replay-side commander and outcome fact normalization.
// coverage-exclusion: reviewed Phase 05 defensive side-fact branches are allowlisted by exact source line.

use parser_contract::{
    diagnostic::{Diagnostic, DiagnosticSeverity},
    events::EventActorRef,
    identity::{EntitySide, ObservedEntity},
    presence::{Confidence, FieldPresence, UnknownReason},
    side_facts::{
        CommanderFactKind, CommanderSideFact, OutcomeFact, OutcomeStatus, ReplaySideFacts,
    },
    source_ref::{RuleId, SourceRefs},
};

use crate::{
    artifact::SourceContext,
    diagnostics::{DiagnosticAccumulator, DiagnosticImpact},
    legacy_player::is_legacy_player_entity,
    raw::{
        MissionMessageEventObservation, RawReplay, RawStringCandidate, mission_message_events,
        string_candidates,
    },
};

const OUTCOME_EXPLICIT_FIELD_RULE_ID: &str = "side_facts.outcome.explicit_field";
const OUTCOME_MISSION_MESSAGE_RULE_ID: &str = "side_facts.outcome.mission_message";
const OUTCOME_UNKNOWN_RULE_ID: &str = "side_facts.outcome.unknown";
const OUTCOME_UNRECOGNIZED_CODE: &str = "side_facts.outcome_unrecognized";
const OUTCOME_CONFLICT_CODE: &str = "side_facts.outcome_conflict";
const COMMANDER_KEYWORD_RULE_ID: &str = "side_facts.commander.keyword_candidate";
const COMMANDER_MISSION_MESSAGE_RULE_ID: &str = "side_facts.commander.mission_message";
const WINNING_COMMANDERS_LABEL: &str = "Победа КС:";
const LOSING_COMMANDERS_LABEL: &str = "Поражение КС:";

/// Normalizes replay-level commander and winner/outcome facts.
#[must_use]
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the plan requires normalize_side_facts to accept a borrowed RawReplay"
)]
pub fn normalize_side_facts(
    raw: &RawReplay<'_>,
    entities: &[ObservedEntity],
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> ReplaySideFacts {
    let mission_messages = mission_message_events(*raw);
    let commander_outcomes = commander_outcomes_from_mission_messages(&mission_messages);
    let commanders = commanders_from_outcomes(entities, &commander_outcomes, context)
        .into_iter()
        .chain(commander_candidates(entities))
        .fold(Vec::new(), push_unique_commander);

    ReplaySideFacts {
        commanders,
        outcome: normalize_outcome(*raw, entities, &commander_outcomes, context, diagnostics),
    }
}

/// Normalizes compact server-facing commander/outcome facts from explicit mission messages.
#[must_use]
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the plan requires normalize_side_facts to accept a borrowed RawReplay"
)]
pub fn normalize_mission_message_side_facts(
    raw: &RawReplay<'_>,
    entities: &[ObservedEntity],
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> ReplaySideFacts {
    let mission_messages = mission_message_events(*raw);
    let commander_outcomes = commander_outcomes_from_mission_messages(&mission_messages);
    if commander_outcomes.is_empty() {
        return ReplaySideFacts::default();
    }

    ReplaySideFacts {
        commanders: commanders_from_outcomes(entities, &commander_outcomes, context)
            .into_iter()
            .fold(Vec::new(), push_unique_commander),
        outcome: mission_message_outcome(&commander_outcomes, entities, context, diagnostics)
            .unwrap_or_else(unknown_outcome),
    }
}

fn normalize_outcome(
    raw: RawReplay<'_>,
    entities: &[ObservedEntity],
    commander_outcomes: &[CommanderOutcomeMessage],
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> OutcomeFact {
    if !commander_outcomes.is_empty() {
        return mission_message_outcome(commander_outcomes, entities, context, diagnostics)
            .unwrap_or_else(unknown_outcome);
    }

    let candidates = string_candidates(raw, &["winner", "winningSide", "outcome"]);

    let recognized = candidates
        .iter()
        .filter_map(|candidate| {
            accepted_winner_side(&candidate.value).map(|side| (candidate, side))
        })
        .collect::<Vec<_>>();

    if recognized_sides_conflict(&recognized) {
        push_conflicting_outcome_diagnostic(&recognized, context, diagnostics);
        return unknown_outcome();
    }

    if let Some((candidate, winner_side)) = recognized.first()
        && let Some(outcome) = known_outcome(candidate, *winner_side, context)
    {
        return outcome;
    }

    for candidate in candidates {
        push_unrecognized_outcome_diagnostic(&candidate, context, diagnostics);
    }

    unknown_outcome()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommanderOutcomeMessage {
    winning_commanders: Vec<String>,
    losing_commanders: Vec<String>,
    message: String,
    event_index: usize,
    frame: Option<u64>,
    json_path: String,
}

fn commander_outcomes_from_mission_messages(
    mission_messages: &[MissionMessageEventObservation],
) -> Vec<CommanderOutcomeMessage> {
    mission_messages.iter().filter_map(commander_outcome_from_message).collect()
}

fn commander_outcome_from_message(
    event: &MissionMessageEventObservation,
) -> Option<CommanderOutcomeMessage> {
    let winning_start = event.message.find(WINNING_COMMANDERS_LABEL)?;
    let after_winning = winning_start + WINNING_COMMANDERS_LABEL.len();
    let losing_relative_start = event.message[after_winning..].find(LOSING_COMMANDERS_LABEL)?;
    let losing_start = after_winning + losing_relative_start;
    let winning_text = event.message[after_winning..losing_start].trim().trim_end_matches('.');
    let losing_text = event.message[(losing_start + LOSING_COMMANDERS_LABEL.len())..].trim();

    Some(CommanderOutcomeMessage {
        winning_commanders: split_commander_names(winning_text),
        losing_commanders: split_commander_names(losing_text),
        message: event.message.clone(),
        event_index: event.event_index,
        frame: event.frame,
        json_path: event.json_path.clone(),
    })
}

fn split_commander_names(value: &str) -> Vec<String> {
    value.split(',').map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned).collect()
}

fn mission_message_outcome(
    commander_outcomes: &[CommanderOutcomeMessage],
    entities: &[ObservedEntity],
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Option<OutcomeFact> {
    for outcome in commander_outcomes {
        let winner_sides = unique_sides(
            outcome
                .winning_commanders
                .iter()
                .filter_map(|name| entity_by_observed_name(entities, name))
                .filter_map(entity_side),
        );

        if winner_sides.len() > 1 {
            push_conflicting_mission_outcome_diagnostic(
                outcome,
                &winner_sides,
                context,
                diagnostics,
            );
            return None;
        }

        if winner_sides.is_empty() {
            continue;
        }

        let winner_side = *winner_sides.first()?;
        let rule_id = rule_id(OUTCOME_MISSION_MESSAGE_RULE_ID)?;
        let source_ref = context.event_source_ref(
            &outcome.json_path,
            outcome.frame,
            event_index_u64(outcome.event_index),
            None,
            Some(rule_id.clone()),
        );
        let source_refs = SourceRefs::new(vec![source_ref.clone()]).ok()?;

        return Some(OutcomeFact {
            status: OutcomeStatus::Inferred,
            winner_side: FieldPresence::Inferred {
                value: winner_side,
                reason: "mission_message names winning KS and matching entity side".to_owned(),
                confidence: Confidence::new(0.9).ok(),
                source: Some(source_ref.clone()),
                rule_id: rule_id.clone(),
            },
            source_label: FieldPresence::Present {
                value: outcome.message.clone(),
                source: Some(source_ref),
            },
            confidence: Confidence::new(0.9).ok(),
            rule_id,
            source_refs: Some(source_refs),
        });
    }

    None
}

fn push_conflicting_mission_outcome_diagnostic(
    outcome: &CommanderOutcomeMessage,
    winner_sides: &[EntitySide],
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) {
    let Some(rule_id) = rule_id(OUTCOME_CONFLICT_CODE) else {
        return;
    };
    let source_ref = context.event_source_ref(
        &outcome.json_path,
        outcome.frame,
        event_index_u64(outcome.event_index),
        None,
        Some(rule_id),
    );
    let Some(source_refs) = SourceRefs::new(vec![source_ref]).ok() else {
        return;
    };
    let observed_shape =
        winner_sides.iter().filter_map(|side| side_name(*side)).collect::<Vec<_>>().join(", ");

    diagnostics.push(
        Diagnostic {
            code: OUTCOME_CONFLICT_CODE.to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Mission-message KS winners resolve to conflicting sides".to_string(),
            json_path: Some(outcome.json_path.clone()),
            expected_shape: Some("all Победа КС names resolve to the same side".to_string()),
            observed_shape: Some(observed_shape),
            parser_action: "set_outcome_unknown".to_string(),
            source_refs,
        },
        DiagnosticImpact::DataLoss,
    );
}

fn unique_sides(sides: impl Iterator<Item = EntitySide>) -> Vec<EntitySide> {
    sides.fold(Vec::new(), |mut result, side| {
        if !result.contains(&side) {
            result.push(side);
        }
        result
    })
}

fn known_outcome(
    candidate: &RawStringCandidate,
    winner_side: EntitySide,
    context: &SourceContext,
) -> Option<OutcomeFact> {
    let rule_id = rule_id(OUTCOME_EXPLICIT_FIELD_RULE_ID)?;
    let source_ref = context.source_ref(&candidate.json_path, Some(rule_id.clone()));
    let source_refs = SourceRefs::new(vec![source_ref.clone()]).ok()?;

    Some(OutcomeFact {
        status: OutcomeStatus::Known,
        winner_side: FieldPresence::Present {
            value: winner_side,
            source: Some(source_ref.clone()),
        },
        source_label: FieldPresence::Present {
            value: candidate.value.clone(),
            source: Some(source_ref),
        },
        confidence: Confidence::new(1.0).ok(),
        rule_id,
        source_refs: Some(source_refs),
    })
}

fn unknown_outcome() -> OutcomeFact {
    OutcomeFact {
        status: OutcomeStatus::Unknown,
        winner_side: FieldPresence::Unknown { reason: UnknownReason::MissingWinner, source: None },
        source_label: FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: None,
        },
        confidence: None,
        rule_id: RuleId(OUTCOME_UNKNOWN_RULE_ID.to_string()),
        source_refs: None,
    }
}

fn accepted_winner_side(value: &str) -> Option<EntitySide> {
    match value.trim().to_ascii_lowercase().as_str() {
        "west" | "blufor" => Some(EntitySide::West),
        "east" | "opfor" => Some(EntitySide::East),
        "guer" | "independent" | "resistance" => Some(EntitySide::Guer),
        "civ" | "civilian" => Some(EntitySide::Civ),
        _ => None,
    }
}

fn recognized_sides_conflict(recognized: &[(&RawStringCandidate, EntitySide)]) -> bool {
    let Some((_, first_side)) = recognized.first() else {
        return false;
    };

    recognized.iter().any(|(_, side)| side != first_side)
}

fn push_conflicting_outcome_diagnostic(
    recognized: &[(&RawStringCandidate, EntitySide)],
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) {
    let source_refs = recognized
        .iter()
        .map(|(candidate, _)| {
            context.source_ref(&candidate.json_path, rule_id(OUTCOME_CONFLICT_CODE))
        })
        .collect::<Vec<_>>();
    let Some(source_refs) = SourceRefs::new(source_refs).ok() else {
        return;
    };
    let observed_shape = recognized
        .iter()
        .map(|(candidate, side)| format!("{}={:?}", candidate.key, side))
        .collect::<Vec<_>>()
        .join(", ");

    diagnostics.push(
        Diagnostic {
            code: OUTCOME_CONFLICT_CODE.to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Replay outcome fields contain conflicting recognized winner sides"
                .to_string(),
            json_path: Some("$".to_string()),
            expected_shape: Some("winner, winningSide, and outcome agree when present".to_string()),
            observed_shape: Some(observed_shape),
            parser_action: "set_outcome_unknown".to_string(),
            source_refs,
        },
        DiagnosticImpact::DataLoss,
    );
}

#[allow(
    clippy::expect_used,
    reason = "a single constructed outcome source reference is non-empty by construction"
)]
fn push_unrecognized_outcome_diagnostic(
    candidate: &RawStringCandidate,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) {
    let source_ref = context.source_ref(&candidate.json_path, rule_id(OUTCOME_UNRECOGNIZED_CODE));
    let source_refs = SourceRefs::new(vec![source_ref])
        .expect("unrecognized outcome diagnostic source refs include one source ref");

    diagnostics.push(
        Diagnostic {
            code: OUTCOME_UNRECOGNIZED_CODE.to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!("Replay outcome field {} has unrecognized side value", candidate.key),
            json_path: Some(candidate.json_path.clone()),
            expected_shape: Some(
                "WEST, west, BLUFOR, EAST, east, OPFOR, GUER, guer, INDEPENDENT, CIV, or civilian"
                    .to_string(),
            ),
            observed_shape: Some(candidate.value.clone()),
            parser_action: "set_outcome_unknown".to_string(),
            source_refs,
        },
        DiagnosticImpact::NonLossWarning,
    );
}

fn commanders_from_outcomes(
    entities: &[ObservedEntity],
    commander_outcomes: &[CommanderOutcomeMessage],
    context: &SourceContext,
) -> Vec<CommanderSideFact> {
    commander_outcomes
        .iter()
        .flat_map(|outcome| {
            outcome
                .winning_commanders
                .iter()
                .chain(outcome.losing_commanders.iter())
                .filter_map(|name| mission_message_commander_fact(entities, name, outcome, context))
        })
        .collect()
}

fn mission_message_commander_fact(
    entities: &[ObservedEntity],
    name: &str,
    outcome: &CommanderOutcomeMessage,
    context: &SourceContext,
) -> Option<CommanderSideFact> {
    let entity = entity_by_observed_name(entities, name);
    let rule_id = rule_id(COMMANDER_MISSION_MESSAGE_RULE_ID)?;
    let message_source = context.event_source_ref(
        &outcome.json_path,
        outcome.frame,
        event_index_u64(outcome.event_index),
        entity.map(|value| value.source_entity_id),
        Some(rule_id.clone()),
    );
    let source_refs = SourceRefs::new(vec![message_source.clone()]).ok()?;

    entity.map(|entity| CommanderSideFact {
        side: entity.identity.side.clone(),
        side_name: side_name_presence(&entity.identity.side),
        commander: FieldPresence::Present {
            value: actor_ref(entity),
            source: Some(message_source),
        },
        kind: CommanderFactKind::Observed,
        confidence: Confidence::new(0.9).ok(),
        rule_id,
        source_refs,
    })
}

fn push_unique_commander(
    mut commanders: Vec<CommanderSideFact>,
    commander: CommanderSideFact,
) -> Vec<CommanderSideFact> {
    let key = commander_key(&commander);
    if !commanders.iter().any(|existing| commander_key(existing) == key) {
        commanders.push(commander);
    }
    commanders
}

fn commander_key(commander: &CommanderSideFact) -> String {
    let side = entity_side_from_presence(&commander.side).and_then(side_name).unwrap_or("unknown");
    let actor = present_actor(&commander.commander);
    let entity_id = actor
        .and_then(|actor| match &actor.source_entity_id {
            FieldPresence::Present { value, .. } => Some(value.to_string()),
            FieldPresence::ExplicitNull { .. }
            | FieldPresence::Unknown { .. }
            | FieldPresence::Inferred { .. }
            | FieldPresence::NotApplicable { .. } => None,
        })
        .unwrap_or_default();
    let observed_name = actor
        .and_then(|actor| present_text(&actor.observed_name))
        .unwrap_or_default()
        .to_lowercase();
    format!("{side}:{entity_id}:{observed_name}")
}

fn event_index_u64(event_index: usize) -> Option<u64> {
    u64::try_from(event_index).ok()
}

fn entity_by_observed_name<'a>(
    entities: &'a [ObservedEntity],
    name: &str,
) -> Option<&'a ObservedEntity> {
    let trimmed_name = name.trim();
    let lower_name = trimmed_name.to_lowercase();
    entities.iter().filter(|entity| is_legacy_player_entity(entity)).find(|entity| {
        [&entity.observed_name, &entity.identity.nickname]
            .iter()
            .filter_map(|field| present_text(field))
            .any(|candidate| candidate.trim().to_lowercase() == lower_name)
    })
}

fn entity_side(entity: &ObservedEntity) -> Option<EntitySide> {
    entity_side_from_presence(&entity.identity.side)
}

fn entity_side_from_presence(side: &FieldPresence<EntitySide>) -> Option<EntitySide> {
    match side {
        FieldPresence::Present { value, .. } | FieldPresence::Inferred { value, .. } => {
            (*value != EntitySide::Unknown).then_some(*value)
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn present_actor(field: &FieldPresence<EventActorRef>) -> Option<&EventActorRef> {
    match field {
        FieldPresence::Present { value, source: _ } | FieldPresence::Inferred { value, .. } => {
            Some(value)
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn commander_candidates(entities: &[ObservedEntity]) -> Vec<CommanderSideFact> {
    entities
        .iter()
        .filter(|entity| is_legacy_player_entity(entity))
        .filter(|entity| has_commander_keyword(entity))
        .filter_map(commander_candidate)
        .collect()
}

fn commander_candidate(entity: &ObservedEntity) -> Option<CommanderSideFact> {
    let source_refs = entity.source_refs.clone();
    let source = source_refs.as_slice().first().cloned();

    Some(CommanderSideFact {
        side: entity.identity.side.clone(),
        side_name: side_name_presence(&entity.identity.side),
        commander: FieldPresence::Present { value: actor_ref(entity), source },
        kind: CommanderFactKind::Candidate,
        confidence: Confidence::new(0.6).ok(),
        rule_id: rule_id(COMMANDER_KEYWORD_RULE_ID)?,
        source_refs,
    })
}

fn has_commander_keyword(entity: &ObservedEntity) -> bool {
    [
        &entity.identity.description,
        &entity.identity.role,
        &entity.identity.group,
        &entity.observed_name,
    ]
    .iter()
    .filter_map(|field| present_text(field))
    .any(contains_commander_keyword)
}

fn contains_commander_keyword(value: &str) -> bool {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .any(|token| {
            token.eq_ignore_ascii_case("ks")
                || token.eq_ignore_ascii_case("commander")
                || token.eq_ignore_ascii_case("командир")
        })
}

const fn present_text(field: &FieldPresence<String>) -> Option<&str> {
    match field {
        FieldPresence::Present { value, source: _ } | FieldPresence::Inferred { value, .. } => {
            Some(value.as_str())
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn side_name_presence(side: &FieldPresence<EntitySide>) -> FieldPresence<String> {
    match side {
        FieldPresence::Present { value, source } => side_name(*value).map_or_else(
            || FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: source.clone(),
            },
            |side_name| FieldPresence::Present {
                value: side_name.to_string(),
                source: source.clone(),
            },
        ),
        FieldPresence::ExplicitNull { source, .. }
        | FieldPresence::Unknown { source, .. }
        | FieldPresence::Inferred { source, .. } => FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: source.clone(),
        },
        FieldPresence::NotApplicable { .. } => {
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: None }
        }
    }
}

const fn side_name(side: EntitySide) -> Option<&'static str> {
    match side {
        EntitySide::East => Some("east"),
        EntitySide::West => Some("west"),
        EntitySide::Guer => Some("guer"),
        EntitySide::Civ => Some("civ"),
        EntitySide::Unknown => None,
    }
}

fn actor_ref(entity: &ObservedEntity) -> EventActorRef {
    EventActorRef {
        source_entity_id: FieldPresence::Present {
            value: entity.source_entity_id,
            source: entity.source_refs.as_slice().first().cloned(),
        },
        observed_name: entity.observed_name.clone(),
        side: entity.identity.side.clone(),
    }
}

fn rule_id(value: &str) -> Option<RuleId> {
    RuleId::new(value).ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use super::*;
    use parser_contract::{
        identity::{EntityKind, ObservedIdentity},
        presence::NullReason,
        source_ref::{ReplaySource, SourceChecksum},
    };

    fn context() -> SourceContext {
        SourceContext::new(&ReplaySource {
            replay_id: Some("side-facts-unit-test".to_owned()),
            source_file: "side-facts-unit-test.ocap.json".to_owned(),
            checksum: FieldPresence::Present {
                value: SourceChecksum::sha256(
                    "9999999999999999999999999999999999999999999999999999999999999999",
                )
                .expect("test checksum should be valid"),
                source: None,
            },
        })
    }

    fn source_refs() -> SourceRefs {
        SourceRefs::new(vec![context().source_ref("$.entities[0]", rule_id("side.test"))])
            .expect("test source refs should be non-empty")
    }

    fn observed_entity(side: FieldPresence<EntitySide>) -> ObservedEntity {
        let source_refs = source_refs();
        ObservedEntity {
            source_entity_id: 1,
            kind: EntityKind::Unit,
            observed_name: FieldPresence::Present {
                value: "Commander".to_owned(),
                source: source_refs.as_slice().first().cloned(),
            },
            observed_class: FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: source_refs.as_slice().first().cloned(),
            },
            is_player: FieldPresence::Present {
                value: true,
                source: source_refs.as_slice().first().cloned(),
            },
            identity: ObservedIdentity {
                nickname: FieldPresence::Present {
                    value: "Commander".to_owned(),
                    source: source_refs.as_slice().first().cloned(),
                },
                steam_id: FieldPresence::Unknown {
                    reason: UnknownReason::SourceFieldAbsent,
                    source: source_refs.as_slice().first().cloned(),
                },
                side,
                faction: FieldPresence::NotApplicable {
                    reason: "unit test faction omitted".to_owned(),
                },
                group: FieldPresence::NotApplicable {
                    reason: "unit test group omitted".to_owned(),
                },
                squad: FieldPresence::NotApplicable {
                    reason: "unit test squad omitted".to_owned(),
                },
                role: FieldPresence::Inferred {
                    value: "commander".to_owned(),
                    reason: "unit test".to_owned(),
                    confidence: Confidence::new(1.0).ok(),
                    source: source_refs.as_slice().first().cloned(),
                    rule_id: rule_id("side.role.test").expect("rule id should be valid"),
                },
                description: FieldPresence::NotApplicable {
                    reason: "unit test description omitted".to_owned(),
                },
            },
            compatibility_hints: Vec::new(),
            source_refs,
        }
    }

    #[test]
    fn side_fact_helpers_should_cover_empty_conflict_and_side_name_states() {
        // Arrange
        let context = context();
        let mut diagnostics = DiagnosticAccumulator::new(8);

        // Act
        push_conflicting_outcome_diagnostic(&[], &context, &mut diagnostics);

        // Assert
        assert!(diagnostics.finish(&context).diagnostics.is_empty());
        assert_eq!(side_name(EntitySide::East), Some("east"));
        assert_eq!(side_name(EntitySide::West), Some("west"));
        assert_eq!(side_name(EntitySide::Guer), Some("guer"));
        assert_eq!(side_name(EntitySide::Civ), Some("civ"));
        assert_eq!(side_name(EntitySide::Unknown), None);
        assert!(matches!(
            side_name_presence(&FieldPresence::Present {
                value: EntitySide::Unknown,
                source: Some(context.source_ref("$.winner", rule_id("side.outcome.test"))),
            }),
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
        ));
        assert!(matches!(
            side_name_presence(&FieldPresence::<EntitySide>::ExplicitNull {
                reason: NullReason::SourceNull,
                source: Some(context.source_ref("$.winner", rule_id("side.outcome.test"))),
            }),
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
        ));
        assert!(matches!(
            side_name_presence(&FieldPresence::<EntitySide>::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: Some(context.source_ref("$.winner", rule_id("side.outcome.test"))),
            }),
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
        ));
        assert!(matches!(
            side_name_presence(&FieldPresence::<EntitySide>::Inferred {
                value: EntitySide::West,
                reason: "unit test".to_owned(),
                confidence: Confidence::new(1.0).ok(),
                source: Some(context.source_ref("$.winner", rule_id("side.outcome.test"))),
                rule_id: rule_id("side.outcome.test").expect("rule id should be valid"),
            }),
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
        ));
        assert!(matches!(
            side_name_presence(&FieldPresence::<EntitySide>::NotApplicable {
                reason: "unit test".to_owned()
            }),
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: None }
        ));
    }

    #[test]
    fn commander_helpers_should_accept_inferred_text_and_unknown_side() {
        // Arrange
        let entity = observed_entity(FieldPresence::Present {
            value: EntitySide::Unknown,
            source: source_refs().as_slice().first().cloned(),
        });

        // Act
        let candidate = commander_candidate(&entity).expect("commander candidate should be built");

        // Assert
        assert!(has_commander_keyword(&entity));
        assert!(contains_commander_keyword("field commander"));
        assert!(matches!(
            candidate.side_name,
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
        ));
    }
}
