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
    raw::{RawReplay, RawStringCandidate, string_candidates},
};

const OUTCOME_EXPLICIT_FIELD_RULE_ID: &str = "side_facts.outcome.explicit_field";
const OUTCOME_UNKNOWN_RULE_ID: &str = "side_facts.outcome.unknown";
const OUTCOME_UNRECOGNIZED_CODE: &str = "side_facts.outcome_unrecognized";
const OUTCOME_CONFLICT_CODE: &str = "side_facts.outcome_conflict";
const COMMANDER_KEYWORD_RULE_ID: &str = "side_facts.commander.keyword_candidate";

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
    ReplaySideFacts {
        commanders: commander_candidates(entities),
        outcome: normalize_outcome(*raw, context, diagnostics),
    }
}

fn normalize_outcome(
    raw: RawReplay<'_>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> OutcomeFact {
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

#[cfg(all(test, not(coverage)))]
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
