//! Legacy player eligibility used by combat and aggregate compatibility projections.
// coverage-exclusion: reviewed Phase 05 legacy player fallback branch is allowlisted by exact source line.

use parser_contract::{
    identity::{EntityCompatibilityHintKind, EntityKind, ObservedEntity},
    presence::FieldPresence,
};

/// Returns true when an observed entity should participate in legacy player projections.
#[must_use]
pub fn is_legacy_player_entity(entity: &ObservedEntity) -> bool {
    if matches!(entity.kind, EntityKind::Vehicle | EntityKind::StaticWeapon) {
        return false;
    }

    has_connected_player_backfill(entity)
        || (matches!(entity.kind, EntityKind::Unit)
            && present_bool(&entity.is_player) == Some(true)
            && observed_string(&entity.observed_name).is_some_and(non_empty)
            && observed_string(&entity.identity.description).is_some_and(non_empty))
}

fn has_connected_player_backfill(entity: &ObservedEntity) -> bool {
    entity.compatibility_hints.iter().any(|hint| {
        hint.kind == EntityCompatibilityHintKind::ConnectedPlayerBackfill
            && observed_string(&hint.observed_name).is_some_and(non_empty)
    })
}

const fn present_bool(field: &FieldPresence<bool>) -> Option<bool> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(*value),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn observed_string(field: &FieldPresence<String>) -> Option<&str> {
    match field {
        FieldPresence::Present { value, source: _ } | FieldPresence::Inferred { value, .. } => {
            Some(value.as_str())
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn non_empty(value: &str) -> bool {
    !value.is_empty()
}

#[cfg(test)]
mod tests {
    use parser_contract::{
        identity::{EntityKind, ObservedEntity, ObservedIdentity},
        presence::{FieldPresence, UnknownReason},
        source_ref::{RuleId, SourceRef, SourceRefs},
    };

    use super::is_legacy_player_entity;

    fn source_refs() -> SourceRefs {
        SourceRefs::single(SourceRef {
            replay_id: Some("replay-1".to_owned()),
            source_file: None,
            checksum: None,
            frame: None,
            event_index: None,
            entity_id: Some(1),
            json_path: None,
            rule_id: Some(RuleId::new("test.entity").expect("rule id should be valid")),
        })
    }

    fn present_string(value: &str) -> FieldPresence<String> {
        FieldPresence::Present { value: value.to_owned(), source: None }
    }

    #[test]
    fn legacy_player_entity_should_ignore_not_applicable_identity_strings() {
        let entity = ObservedEntity {
            source_entity_id: 1,
            kind: EntityKind::Unit,
            observed_name: present_string("Player One"),
            observed_class: present_string("B_Soldier_F"),
            is_player: FieldPresence::Present { value: true, source: None },
            identity: ObservedIdentity {
                nickname: FieldPresence::NotApplicable {
                    reason: "no observed nickname".to_owned(),
                },
                steam_id: FieldPresence::Unknown {
                    reason: UnknownReason::MissingSteamId,
                    source: None,
                },
                side: FieldPresence::Unknown {
                    reason: UnknownReason::SourceFieldAbsent,
                    source: None,
                },
                faction: FieldPresence::Unknown {
                    reason: UnknownReason::SourceFieldAbsent,
                    source: None,
                },
                group: FieldPresence::NotApplicable { reason: "no observed group".to_owned() },
                squad: FieldPresence::NotApplicable { reason: "no observed squad".to_owned() },
                role: FieldPresence::NotApplicable { reason: "no observed role".to_owned() },
                description: FieldPresence::NotApplicable {
                    reason: "no observed description".to_owned(),
                },
            },
            compatibility_hints: Vec::new(),
            source_refs: source_refs(),
        };

        assert!(!is_legacy_player_entity(&entity));
    }
}
