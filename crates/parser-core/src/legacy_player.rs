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
