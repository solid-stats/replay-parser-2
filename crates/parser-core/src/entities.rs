//! Observed entity normalization from OCAP entity rows.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use parser_contract::{
    diagnostic::{Diagnostic, DiagnosticSeverity},
    identity::{
        EntityCompatibilityHint, EntityCompatibilityHintKind, EntityKind, EntitySide,
        ObservedEntity, ObservedIdentity,
    },
    presence::{Confidence, FieldPresence, UnknownReason},
    source_ref::{RuleId, SourceRef, SourceRefs},
};
use serde_json::Value;

use crate::{
    artifact::SourceContext,
    diagnostics::{DiagnosticAccumulator, DiagnosticImpact},
    raw::{
        ConnectedEventObservation, RawField, RawReplay, connected_events, entity_class,
        entity_description, entity_group, entity_has_positions, entity_id, entity_is_player,
        entity_name, entity_side, entity_type,
    },
};

const CONNECTED_PLAYER_BACKFILL_RULE_ID: &str = "entity.connected_player_backfill";
const DUPLICATE_SLOT_SAME_NAME_RULE_ID: &str = "entity.duplicate_slot_same_name";

/// Normalizes observed unit/player, vehicle, and static weapon entity facts.
#[must_use]
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the plan requires normalize_entities to accept a borrowed RawReplay"
)]
pub fn normalize_entities(
    raw: &RawReplay<'_>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Vec<ObservedEntity> {
    let RawField::Present { value: entities, json_path: _ } = raw.array_field("entities") else {
        push_entities_section_diagnostic(*raw, context, diagnostics);
        return Vec::new();
    };

    let mut normalized = entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| normalize_entity(entity, index, context, diagnostics))
        .collect::<Vec<_>>();

    apply_connected_player_backfill(*raw, context, &mut normalized);
    apply_duplicate_slot_same_name_hints(diagnostics, &mut normalized);
    normalized.sort_by(compare_entities);
    normalized
}

fn apply_connected_player_backfill(
    raw: RawReplay<'_>,
    context: &SourceContext,
    entities: &mut [ObservedEntity],
) {
    let entity_index = entities
        .iter()
        .enumerate()
        .map(|(index, entity)| (entity.source_entity_id, index))
        .collect::<BTreeMap<_, _>>();

    for connected_event in connected_events(raw) {
        if connected_event.name.is_empty() {
            continue;
        }

        let Some(entity_index) = entity_index.get(&connected_event.entity_id).copied() else {
            continue;
        };
        let entity = &mut entities[entity_index];

        if matches!(entity.kind, EntityKind::Vehicle | EntityKind::StaticWeapon) {
            continue;
        }

        let connected_source_ref = connected_event_source_ref(
            context,
            &connected_event,
            CONNECTED_PLAYER_BACKFILL_RULE_ID,
        );
        let hint = connected_player_backfill_hint(entity, &connected_event, &connected_source_ref);

        let inferred_name = inferred_connected_name(&connected_event.name, connected_source_ref);
        if should_infer_connected_name(&entity.observed_name) {
            entity.observed_name = inferred_name.clone();
        }
        entity.identity.nickname = inferred_name;

        if let Some(hint) = hint {
            entity.compatibility_hints.push(hint);
        }
    }
}

const fn should_infer_connected_name(observed_name: &FieldPresence<String>) -> bool {
    match observed_name {
        FieldPresence::Present { value, source: _ } => value.is_empty(),
        FieldPresence::Unknown { .. } => true,
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => false,
    }
}

fn inferred_connected_name(name: &str, source_ref: SourceRef) -> FieldPresence<String> {
    FieldPresence::Inferred {
        value: name.to_string(),
        reason: "legacy connected event player backfill".to_string(),
        confidence: Confidence::new(1.0).ok(),
        source: Some(source_ref),
        rule_id: RuleId(CONNECTED_PLAYER_BACKFILL_RULE_ID.to_owned()),
    }
}

fn apply_duplicate_slot_same_name_hints(
    diagnostics: &mut DiagnosticAccumulator,
    entities: &mut [ObservedEntity],
) {
    let duplicate_groups = duplicate_slot_same_name_groups(entities);

    for group in duplicate_groups {
        if group.has_side_conflict {
            push_duplicate_side_conflict_diagnostic(diagnostics, &group);
        }

        for entity_index in group.entity_indexes {
            entities[entity_index].compatibility_hints.push(group.hint.clone());
        }
    }
}

fn duplicate_slot_same_name_groups(entities: &[ObservedEntity]) -> Vec<DuplicateSlotGroup> {
    let mut grouped = BTreeMap::<String, SameNameAccumulator>::new();

    for (entity_index, entity) in entities.iter().enumerate() {
        if !matches!(entity.kind, EntityKind::Unit) {
            continue;
        }

        let Some((name, name_state)) = duplicate_group_name(&entity.observed_name) else {
            continue;
        };
        let group = grouped.entry(name.clone()).or_insert_with(|| SameNameAccumulator {
            name,
            entity_indexes: Vec::new(),
            has_present_name: false,
        });
        group.entity_indexes.push(entity_index);
        group.has_present_name |= matches!(name_state, DuplicateNameState::Present);
    }

    grouped
        .into_values()
        .filter(|group| group.entity_indexes.len() > 1)
        .filter_map(|group| duplicate_slot_group(group, entities))
        .collect()
}

fn duplicate_slot_group(
    group: SameNameAccumulator,
    entities: &[ObservedEntity],
) -> Option<DuplicateSlotGroup> {
    let mut entity_indexes = group.entity_indexes;
    entity_indexes.sort_by_key(|entity_index| entities[*entity_index].source_entity_id);

    let related_entity_ids = entity_indexes
        .iter()
        .map(|entity_index| entities[*entity_index].source_entity_id)
        .collect::<Vec<_>>();
    let source_refs = duplicate_group_source_refs(&entity_indexes, entities);
    let source_refs = SourceRefs::new(source_refs).ok()?;
    let observed_name =
        duplicate_hint_observed_name(&group.name, group.has_present_name, &source_refs)?;
    let hint = EntityCompatibilityHint {
        kind: EntityCompatibilityHintKind::DuplicateSlotSameName,
        related_entity_ids,
        observed_name,
        rule_id: RuleId::new(DUPLICATE_SLOT_SAME_NAME_RULE_ID).ok()?,
        source_refs: source_refs.clone(),
    };
    let side_names = duplicate_group_present_side_names(&entity_indexes, entities);
    let has_side_conflict = side_names.len() > 1;
    let side_names = side_names.into_iter().collect::<Vec<_>>().join(",");

    Some(DuplicateSlotGroup { entity_indexes, hint, source_refs, has_side_conflict, side_names })
}

fn duplicate_group_name(
    observed_name: &FieldPresence<String>,
) -> Option<(String, DuplicateNameState)> {
    match observed_name {
        FieldPresence::Present { value, source: _ } if !value.is_empty() => {
            Some((value.clone(), DuplicateNameState::Present))
        }
        FieldPresence::Inferred { value, .. } if !value.is_empty() => {
            Some((value.clone(), DuplicateNameState::Inferred))
        }
        FieldPresence::Present { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn duplicate_group_source_refs(
    entity_indexes: &[usize],
    entities: &[ObservedEntity],
) -> Vec<SourceRef> {
    let mut source_refs = Vec::new();

    for entity_index in entity_indexes {
        let entity = &entities[*entity_index];
        source_refs.extend(entity.source_refs.as_slice().iter().cloned());

        if let Some(source_ref) = field_source_ref(&entity.observed_name) {
            source_refs.push(source_ref.clone());
        }
    }

    source_refs
}

fn duplicate_hint_observed_name(
    name: &str,
    has_present_name: bool,
    source_refs: &SourceRefs,
) -> Option<FieldPresence<String>> {
    let source = source_refs.as_slice().first().cloned();

    if has_present_name {
        return Some(FieldPresence::Present { value: name.to_string(), source });
    }

    Some(FieldPresence::Inferred {
        value: name.to_string(),
        reason: "legacy duplicate-slot same-name compatibility".to_string(),
        confidence: Confidence::new(1.0).ok(),
        source,
        rule_id: RuleId::new(DUPLICATE_SLOT_SAME_NAME_RULE_ID).ok()?,
    })
}

fn duplicate_group_present_side_names(
    entity_indexes: &[usize],
    entities: &[ObservedEntity],
) -> BTreeSet<&'static str> {
    entity_indexes
        .iter()
        .filter_map(|entity_index| present_side_name(&entities[*entity_index].identity.side))
        .collect::<BTreeSet<_>>()
}

#[allow(
    clippy::expect_used,
    reason = "duplicate-side conflict diagnostics clone an already validated non-empty SourceRefs"
)]
fn push_duplicate_side_conflict_diagnostic(
    diagnostics: &mut DiagnosticAccumulator,
    group: &DuplicateSlotGroup,
) {
    let source_refs = SourceRefs::new(group.source_refs.as_slice().to_vec())
        .expect("duplicate-side conflict source refs were already validated");

    diagnostics.push(
        Diagnostic {
            code: "compat.entity_duplicate_side_conflict".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "Duplicate same-name entity slots for {} have conflicting present sides",
                duplicate_hint_name_value(&group.hint.observed_name)
            ),
            json_path: Some("$.entities".to_string()),
            expected_shape: Some("same present side for duplicate same-name slots".to_string()),
            observed_shape: Some(group.side_names.clone()),
            parser_action: "kept_entities_unmerged".to_string(),
            source_refs,
        },
        DiagnosticImpact::DataLoss,
    );
}

const fn duplicate_hint_name_value(observed_name: &FieldPresence<String>) -> &str {
    match observed_name {
        FieldPresence::Present { value, source: _ } | FieldPresence::Inferred { value, .. } => {
            value.as_str()
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => "unknown",
    }
}

const fn present_side_name(side: &FieldPresence<EntitySide>) -> Option<&'static str> {
    match side {
        FieldPresence::Present { value, source: _ } => Some(match value {
            EntitySide::East => "east",
            EntitySide::West => "west",
            EntitySide::Guer => "guer",
            EntitySide::Civ => "civ",
            EntitySide::Unknown => "unknown",
        }),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn field_source_ref<T>(field: &FieldPresence<T>) -> Option<&SourceRef> {
    match field {
        FieldPresence::Present { source: Some(source), .. }
        | FieldPresence::ExplicitNull { source: Some(source), .. }
        | FieldPresence::Unknown { source: Some(source), .. }
        | FieldPresence::Inferred { source: Some(source), .. } => Some(source),
        FieldPresence::Present { source: None, .. }
        | FieldPresence::ExplicitNull { source: None, .. }
        | FieldPresence::Unknown { source: None, .. }
        | FieldPresence::Inferred { source: None, .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

struct SameNameAccumulator {
    name: String,
    entity_indexes: Vec<usize>,
    has_present_name: bool,
}

struct DuplicateSlotGroup {
    entity_indexes: Vec<usize>,
    hint: EntityCompatibilityHint,
    source_refs: SourceRefs,
    has_side_conflict: bool,
    side_names: String,
}

#[derive(Clone, Copy)]
enum DuplicateNameState {
    Present,
    Inferred,
}

fn connected_player_backfill_hint(
    entity: &ObservedEntity,
    connected_event: &ConnectedEventObservation,
    connected_source_ref: &SourceRef,
) -> Option<EntityCompatibilityHint> {
    let mut source_refs = Vec::with_capacity(entity.source_refs.as_slice().len() + 1);
    source_refs.push(connected_source_ref.clone());
    source_refs.extend(entity.source_refs.as_slice().iter().cloned());

    Some(EntityCompatibilityHint {
        kind: EntityCompatibilityHintKind::ConnectedPlayerBackfill,
        related_entity_ids: vec![entity.source_entity_id],
        observed_name: FieldPresence::Present {
            value: connected_event.name.clone(),
            source: Some(connected_source_ref.clone()),
        },
        rule_id: RuleId::new(CONNECTED_PLAYER_BACKFILL_RULE_ID).ok()?,
        source_refs: SourceRefs::new(source_refs).ok()?,
    })
}

fn push_entities_section_diagnostic(
    raw: RawReplay<'_>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) {
    match raw.array_field("entities") {
        RawField::Absent { json_path } => push_diagnostic(
            diagnostics,
            EntityDiagnostic {
                code: "schema.entities_absent",
                message: "OCAP replay has no entities section",
                json_path: &json_path,
                expected_shape: "array",
                observed_shape: "absent",
                parser_action: "skip_entities",
                entity_id: None,
            },
            context,
            DiagnosticImpact::DataLoss,
        ),
        RawField::Drift { json_path, expected_shape, observed_shape } => push_diagnostic(
            diagnostics,
            EntityDiagnostic {
                code: "schema.entities_shape",
                message: "OCAP replay entities section had unexpected source shape",
                json_path: &json_path,
                expected_shape,
                observed_shape: &observed_shape,
                parser_action: "skip_entities",
                entity_id: None,
            },
            context,
            DiagnosticImpact::DataLoss,
        ),
        RawField::Present { .. } => {}
    }
}

fn normalize_entity(
    entity: &Value,
    index: usize,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Option<ObservedEntity> {
    let source_entity_id = required_entity_id(entity_id(entity, index), context, diagnostics)?;
    let kind = classify_entity(entity, index, source_entity_id, context, diagnostics);
    let observed_name =
        string_presence(entity_name(entity, index), "name", source_entity_id, context, diagnostics);
    let observed_class =
        observed_class(entity, index, source_entity_id, kind, context, diagnostics);
    let identity = observed_identity(
        entity,
        index,
        source_entity_id,
        kind,
        &observed_name,
        context,
        diagnostics,
    );

    let is_player = player_flag_presence(
        entity_is_player(entity, index),
        source_entity_id,
        kind,
        context,
        diagnostics,
    );

    let source_refs = entity_source_refs(entity, index, source_entity_id, context)?;

    Some(ObservedEntity {
        source_entity_id,
        kind,
        observed_name,
        observed_class,
        is_player,
        identity,
        compatibility_hints: Vec::new(),
        source_refs,
    })
}

fn required_entity_id(
    raw_field: RawField<i64>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Option<i64> {
    match raw_field {
        RawField::Present { value, json_path: _ } => Some(value),
        RawField::Absent { json_path } => {
            push_diagnostic(
                diagnostics,
                EntityDiagnostic {
                    code: "schema.entity_id_absent",
                    message: "Entity row has no numeric source identifier",
                    json_path: &json_path,
                    expected_shape: "integer",
                    observed_shape: "absent",
                    parser_action: "drop_entity",
                    entity_id: None,
                },
                context,
                DiagnosticImpact::DataLoss,
            );
            None
        }
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            push_diagnostic(
                diagnostics,
                EntityDiagnostic {
                    code: "schema.entity_id_shape",
                    message: "Entity row source identifier had unexpected shape",
                    json_path: &json_path,
                    expected_shape,
                    observed_shape: &observed_shape,
                    parser_action: "drop_entity",
                    entity_id: None,
                },
                context,
                DiagnosticImpact::DataLoss,
            );
            None
        }
    }
}

fn classify_entity(
    entity: &Value,
    index: usize,
    source_entity_id: i64,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> EntityKind {
    match entity_type(entity, index) {
        RawField::Present { value, json_path } => match value.as_str() {
            "unit" => EntityKind::Unit,
            "vehicle" => match entity_class(entity, index) {
                RawField::Present { value, json_path: _ } if value == "static-weapon" => {
                    EntityKind::StaticWeapon
                }
                RawField::Present { .. } | RawField::Absent { .. } | RawField::Drift { .. } => {
                    EntityKind::Vehicle
                }
            },
            _ => {
                push_diagnostic(
                    diagnostics,
                    EntityDiagnostic {
                        code: "schema.entity_type_unknown",
                        message: "Entity row has an unknown source type",
                        json_path: &json_path,
                        expected_shape: "unit or vehicle",
                        observed_shape: &value,
                        parser_action: "set_entity_kind_unknown",
                        entity_id: Some(source_entity_id),
                    },
                    context,
                    DiagnosticImpact::DataLoss,
                );
                EntityKind::Unknown
            }
        },
        RawField::Absent { json_path } => {
            push_diagnostic(
                diagnostics,
                EntityDiagnostic {
                    code: "schema.entity_type_unknown",
                    message: "Entity row has no source type",
                    json_path: &json_path,
                    expected_shape: "unit or vehicle",
                    observed_shape: "absent",
                    parser_action: "set_entity_kind_unknown",
                    entity_id: Some(source_entity_id),
                },
                context,
                DiagnosticImpact::DataLoss,
            );
            EntityKind::Unknown
        }
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            push_diagnostic(
                diagnostics,
                EntityDiagnostic {
                    code: "schema.entity_type_unknown",
                    message: "Entity row source type had unexpected shape",
                    json_path: &json_path,
                    expected_shape,
                    observed_shape: &observed_shape,
                    parser_action: "set_entity_kind_unknown",
                    entity_id: Some(source_entity_id),
                },
                context,
                DiagnosticImpact::DataLoss,
            );
            EntityKind::Unknown
        }
    }
}

fn observed_class(
    entity: &Value,
    index: usize,
    source_entity_id: i64,
    kind: EntityKind,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> FieldPresence<String> {
    if matches!(kind, EntityKind::Unit) {
        return FieldPresence::NotApplicable {
            reason: "unit entity has no observed vehicle class".to_string(),
        };
    }

    string_presence(entity_class(entity, index), "class", source_entity_id, context, diagnostics)
}

fn observed_identity(
    entity: &Value,
    index: usize,
    source_entity_id: i64,
    kind: EntityKind,
    observed_name: &FieldPresence<String>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> ObservedIdentity {
    let side = side_presence(entity_side(entity, index), source_entity_id, context, diagnostics);

    if matches!(kind, EntityKind::Unit) {
        let group = string_presence(
            entity_group(entity, index),
            "group",
            source_entity_id,
            context,
            diagnostics,
        );
        let description = string_presence(
            entity_description(entity, index),
            "description",
            source_entity_id,
            context,
            diagnostics,
        );

        return ObservedIdentity {
            nickname: observed_name.clone(),
            steam_id: FieldPresence::Unknown {
                reason: UnknownReason::MissingSteamId,
                source: None,
            },
            side,
            faction: FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: None,
            },
            group,
            squad: FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: None,
            },
            role: description.clone(),
            description,
        };
    }

    ObservedIdentity {
        nickname: FieldPresence::NotApplicable {
            reason: "non-player entity has no observed nickname".to_string(),
        },
        steam_id: FieldPresence::NotApplicable {
            reason: "non-player entity has no observed SteamID".to_string(),
        },
        side,
        faction: FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: None },
        group: FieldPresence::NotApplicable {
            reason: "non-player entity has no observed group".to_string(),
        },
        squad: FieldPresence::NotApplicable {
            reason: "non-player entity has no observed squad".to_string(),
        },
        role: FieldPresence::NotApplicable {
            reason: "non-player entity has no observed role".to_string(),
        },
        description: FieldPresence::NotApplicable {
            reason: "non-player entity has no observed description".to_string(),
        },
    }
}

fn string_presence(
    raw_field: RawField<String>,
    contract_field: &'static str,
    source_entity_id: i64,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> FieldPresence<String> {
    match raw_field {
        RawField::Present { value, json_path } => FieldPresence::Present {
            value,
            source: Some(entity_source_ref(
                context,
                &json_path,
                source_entity_id,
                rule_id(&format!("entity.{contract_field}.observed")),
            )),
        },
        RawField::Absent { json_path } => FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(entity_source_ref(
                context,
                &json_path,
                source_entity_id,
                rule_id(&format!("entity.{contract_field}.observed")),
            )),
        },
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            push_diagnostic(
                diagnostics,
                EntityDiagnostic {
                    code: "schema.entity_field",
                    message: "Entity field had unexpected source shape",
                    json_path: &json_path,
                    expected_shape,
                    observed_shape: &observed_shape,
                    parser_action: "set_unknown",
                    entity_id: Some(source_entity_id),
                },
                context,
                DiagnosticImpact::DataLoss,
            );

            FieldPresence::Unknown {
                reason: UnknownReason::SchemaDrift,
                source: Some(entity_source_ref(
                    context,
                    &json_path,
                    source_entity_id,
                    rule_id(&format!("entity.{contract_field}.observed")),
                )),
            }
        }
    }
}

fn side_presence(
    raw_field: RawField<String>,
    source_entity_id: i64,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> FieldPresence<EntitySide> {
    match raw_field {
        RawField::Present { value, json_path } => {
            let side = match value.as_str() {
                "EAST" => EntitySide::East,
                "WEST" => EntitySide::West,
                "GUER" => EntitySide::Guer,
                "CIV" => EntitySide::Civ,
                "UNKNOWN" => EntitySide::Unknown,
                _ => {
                    push_diagnostic(
                        diagnostics,
                        EntityDiagnostic {
                            code: "schema.entity_side_unknown",
                            message: "Entity side had an unknown source value",
                            json_path: &json_path,
                            expected_shape: "EAST, WEST, GUER, CIV, or UNKNOWN",
                            observed_shape: &value,
                            parser_action: "set_unknown",
                            entity_id: Some(source_entity_id),
                        },
                        context,
                        DiagnosticImpact::DataLoss,
                    );
                    EntitySide::Unknown
                }
            };

            FieldPresence::Present {
                value: side,
                source: Some(entity_source_ref(
                    context,
                    &json_path,
                    source_entity_id,
                    rule_id("entity.side.observed"),
                )),
            }
        }
        RawField::Absent { json_path } => FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(entity_source_ref(
                context,
                &json_path,
                source_entity_id,
                rule_id("entity.side.observed"),
            )),
        },
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            push_diagnostic(
                diagnostics,
                EntityDiagnostic {
                    code: "schema.entity_side_shape",
                    message: "Entity side had unexpected source shape",
                    json_path: &json_path,
                    expected_shape,
                    observed_shape: &observed_shape,
                    parser_action: "set_unknown",
                    entity_id: Some(source_entity_id),
                },
                context,
                DiagnosticImpact::DataLoss,
            );

            FieldPresence::Unknown {
                reason: UnknownReason::SchemaDrift,
                source: Some(entity_source_ref(
                    context,
                    &json_path,
                    source_entity_id,
                    rule_id("entity.side.observed"),
                )),
            }
        }
    }
}

fn player_flag_presence(
    raw_field: RawField<bool>,
    source_entity_id: i64,
    kind: EntityKind,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> FieldPresence<bool> {
    if !matches!(kind, EntityKind::Unit) {
        return FieldPresence::NotApplicable {
            reason: "non-unit entity has no observed player flag".to_string(),
        };
    }

    match raw_field {
        RawField::Present { value, json_path } => FieldPresence::Present {
            value,
            source: Some(entity_source_ref(
                context,
                &json_path,
                source_entity_id,
                rule_id("entity.is_player.observed"),
            )),
        },
        RawField::Absent { json_path } => FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(entity_source_ref(
                context,
                &json_path,
                source_entity_id,
                rule_id("entity.is_player.observed"),
            )),
        },
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            push_diagnostic(
                diagnostics,
                EntityDiagnostic {
                    code: "schema.entity_is_player_shape",
                    message: "Entity isPlayer flag had unexpected source shape",
                    json_path: &json_path,
                    expected_shape,
                    observed_shape: &observed_shape,
                    parser_action: "set_unknown",
                    entity_id: Some(source_entity_id),
                },
                context,
                DiagnosticImpact::DataLoss,
            );

            FieldPresence::Unknown {
                reason: UnknownReason::SchemaDrift,
                source: Some(entity_source_ref(
                    context,
                    &json_path,
                    source_entity_id,
                    rule_id("entity.is_player.observed"),
                )),
            }
        }
    }
}

fn entity_source_refs(
    entity: &Value,
    index: usize,
    source_entity_id: i64,
    context: &SourceContext,
) -> Option<SourceRefs> {
    let mut refs = vec![entity_source_ref(
        context,
        &format!("$.entities[{index}]"),
        source_entity_id,
        rule_id("entity.observed"),
    )];

    if entity_has_positions(entity, index) {
        refs.push(entity_source_ref(
            context,
            &format!("$.entities[{index}].positions"),
            source_entity_id,
            rule_id("entity.positions.observed"),
        ));
    }

    SourceRefs::new(refs).ok()
}

fn entity_source_ref(
    context: &SourceContext,
    json_path: &str,
    source_entity_id: i64,
    rule_id: Option<RuleId>,
) -> SourceRef {
    let mut source_ref = context.source_ref(json_path, rule_id);
    source_ref.entity_id = Some(source_entity_id);
    source_ref
}

fn connected_event_source_ref(
    context: &SourceContext,
    connected_event: &ConnectedEventObservation,
    rule_id_value: &str,
) -> SourceRef {
    let mut source_ref = context.source_ref(&connected_event.json_path, rule_id(rule_id_value));
    source_ref.frame = Some(connected_event.frame);
    source_ref.event_index = u64::try_from(connected_event.event_index).ok();
    source_ref.entity_id = Some(connected_event.entity_id);
    source_ref
}

fn push_diagnostic(
    diagnostics: &mut DiagnosticAccumulator,
    spec: EntityDiagnostic<'_>,
    context: &SourceContext,
    impact: DiagnosticImpact,
) {
    if let Some(diagnostic) = entity_diagnostic(spec, context) {
        diagnostics.push(diagnostic, impact);
    }
}

fn entity_diagnostic(spec: EntityDiagnostic<'_>, context: &SourceContext) -> Option<Diagnostic> {
    let source_ref = spec.entity_id.map_or_else(
        || context.source_ref(spec.json_path, rule_id("diagnostic.schema_drift.entity")),
        |entity_id| {
            entity_source_ref(
                context,
                spec.json_path,
                entity_id,
                rule_id("diagnostic.schema_drift.entity"),
            )
        },
    );
    let source_refs = SourceRefs::new(vec![source_ref]).ok()?;

    Some(Diagnostic {
        code: spec.code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: spec.message.to_string(),
        json_path: Some(spec.json_path.to_string()),
        expected_shape: Some(spec.expected_shape.to_string()),
        observed_shape: Some(spec.observed_shape.to_string()),
        parser_action: spec.parser_action.to_string(),
        source_refs,
    })
}

#[derive(Clone, Copy)]
struct EntityDiagnostic<'a> {
    code: &'static str,
    message: &'static str,
    json_path: &'a str,
    expected_shape: &'static str,
    observed_shape: &'a str,
    parser_action: &'static str,
    entity_id: Option<i64>,
}

fn compare_entities(left: &ObservedEntity, right: &ObservedEntity) -> Ordering {
    left.source_entity_id
        .cmp(&right.source_entity_id)
        .then_with(|| kind_rank(left.kind).cmp(&kind_rank(right.kind)))
        .then_with(|| {
            present_string(&left.observed_name).cmp(&present_string(&right.observed_name))
        })
        .then_with(|| {
            present_string(&left.observed_class).cmp(&present_string(&right.observed_class))
        })
        .then_with(|| first_source_path(left).cmp(&first_source_path(right)))
}

const fn kind_rank(kind: EntityKind) -> u8 {
    match kind {
        EntityKind::Unit => 0,
        EntityKind::Vehicle => 1,
        EntityKind::StaticWeapon => 2,
        EntityKind::Unknown => 3,
    }
}

const fn present_string(field: &FieldPresence<String>) -> Option<&str> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(value.as_str()),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn first_source_path(entity: &ObservedEntity) -> Option<&str> {
    entity.source_refs.as_slice().first().and_then(|source_ref| source_ref.json_path.as_deref())
}

fn rule_id(value: &str) -> Option<RuleId> {
    RuleId::new(value).ok()
}

#[cfg(all(test, not(coverage)))]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use super::*;
    use parser_contract::{
        presence::NullReason,
        source_ref::{ReplaySource, SourceChecksum},
    };
    use serde_json::json;

    fn context() -> SourceContext {
        SourceContext::new(&ReplaySource {
            replay_id: Some("entities-unit-test".to_owned()),
            source_file: "entities-unit-test.ocap.json".to_owned(),
            checksum: FieldPresence::Present {
                value: SourceChecksum::sha256(
                    "8888888888888888888888888888888888888888888888888888888888888888",
                )
                .expect("test checksum should be valid"),
                source: None,
            },
        })
    }

    fn source_ref(path: &str) -> SourceRef {
        context().source_ref(path, rule_id("entity.unit_test"))
    }

    fn source_refs(path: &str) -> SourceRefs {
        SourceRefs::new(vec![source_ref(path)]).expect("test source refs should be non-empty")
    }

    fn entity(
        source_entity_id: i64,
        kind: EntityKind,
        observed_name: FieldPresence<String>,
        observed_class: FieldPresence<String>,
        source_path: &str,
    ) -> ObservedEntity {
        ObservedEntity {
            source_entity_id,
            kind,
            observed_name: observed_name.clone(),
            observed_class,
            is_player: FieldPresence::Present {
                value: matches!(kind, EntityKind::Unit),
                source: Some(source_ref(source_path)),
            },
            identity: ObservedIdentity {
                nickname: observed_name,
                steam_id: FieldPresence::Unknown {
                    reason: UnknownReason::SourceFieldAbsent,
                    source: Some(source_ref(source_path)),
                },
                side: FieldPresence::Unknown {
                    reason: UnknownReason::SourceFieldAbsent,
                    source: Some(source_ref(source_path)),
                },
                faction: FieldPresence::NotApplicable {
                    reason: "unit test faction omitted".to_owned(),
                },
                group: FieldPresence::NotApplicable {
                    reason: "unit test group omitted".to_owned(),
                },
                squad: FieldPresence::NotApplicable {
                    reason: "unit test squad omitted".to_owned(),
                },
                role: FieldPresence::NotApplicable { reason: "unit test role omitted".to_owned() },
                description: FieldPresence::NotApplicable {
                    reason: "unit test description omitted".to_owned(),
                },
            },
            compatibility_hints: Vec::new(),
            source_refs: source_refs(source_path),
        }
    }

    #[test]
    fn entity_presence_helpers_should_cover_defensive_duplicate_states() {
        // Arrange
        let refs = source_refs("$.entities[0]");
        let explicit_null = FieldPresence::<String>::ExplicitNull {
            reason: NullReason::SourceNull,
            source: Some(source_ref("$.entities[0].name")),
        };
        let inferred = FieldPresence::Inferred {
            value: "Echo".to_owned(),
            reason: "unit test".to_owned(),
            confidence: Confidence::new(1.0).ok(),
            source: Some(source_ref("$.events[0]")),
            rule_id: rule_id("entity.inferred.test").expect("rule id should be valid"),
        };

        // Act
        let duplicate_name = duplicate_hint_observed_name("Echo", false, &refs);

        // Assert
        assert!(should_infer_connected_name(&FieldPresence::Present {
            value: String::new(),
            source: Some(source_ref("$.entities[0].name")),
        }));
        assert!(should_infer_connected_name(&FieldPresence::<String>::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(source_ref("$.entities[0].name")),
        }));
        assert!(!should_infer_connected_name(&inferred));
        assert!(!should_infer_connected_name(&FieldPresence::<String>::NotApplicable {
            reason: "unit test".to_owned(),
        }));
        assert!(matches!(
            duplicate_name,
            Some(FieldPresence::Inferred { value, source: Some(_), .. }) if value == "Echo"
        ));
        assert_eq!(
            duplicate_hint_name_value(&FieldPresence::<String>::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: None,
            }),
            "unknown"
        );
        assert!(field_source_ref(&explicit_null).is_some());
        assert!(
            field_source_ref(&FieldPresence::<String>::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: Some(source_ref("$.entities[0].name")),
            })
            .is_some()
        );
        assert!(field_source_ref(&inferred).is_some());
        assert!(
            field_source_ref(&FieldPresence::<String>::NotApplicable {
                reason: "unit test".to_owned()
            })
            .is_none()
        );
    }

    #[test]
    fn entity_side_and_classification_helpers_should_cover_all_absent_states() {
        // Arrange
        let context = context();
        let mut diagnostics = DiagnosticAccumulator::new(16);
        let entity_without_type = json!({ "id": 10 });

        // Act
        let kind = classify_entity(&entity_without_type, 0, 10, &context, &mut diagnostics);
        let player_flag = player_flag_presence(
            RawField::Absent { json_path: "$.entities[0].isPlayer".to_owned() },
            10,
            EntityKind::Unit,
            &context,
            &mut diagnostics,
        );

        // Assert
        assert_eq!(kind, EntityKind::Unknown);
        assert!(matches!(
            player_flag,
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
        ));
        for (side, expected) in [
            (EntitySide::East, Some("east")),
            (EntitySide::West, Some("west")),
            (EntitySide::Guer, Some("guer")),
            (EntitySide::Civ, Some("civ")),
            (EntitySide::Unknown, Some("unknown")),
        ] {
            assert_eq!(
                present_side_name(&FieldPresence::Present {
                    value: side,
                    source: Some(source_ref("$.entities[0].side")),
                }),
                expected
            );
        }
        assert_eq!(
            present_side_name(&FieldPresence::<EntitySide>::NotApplicable {
                reason: "unit test".to_owned()
            }),
            None
        );
    }

    #[test]
    fn entity_sort_helpers_should_compare_missing_values_and_unknown_kind_rank() {
        // Arrange
        let missing_name = FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(source_ref("$.entities[0].name")),
        };
        let missing_class =
            FieldPresence::NotApplicable { reason: "unit test class omitted".to_owned() };
        let left = entity(
            10,
            EntityKind::Unit,
            missing_name.clone(),
            missing_class.clone(),
            "$.entities[0]",
        );
        let right = entity(10, EntityKind::Unit, missing_name, missing_class, "$.entities[1]");

        // Act + Assert
        assert_eq!(compare_entities(&left, &right), Ordering::Less);
        assert_eq!(kind_rank(EntityKind::Vehicle), 1);
        assert_eq!(kind_rank(EntityKind::StaticWeapon), 2);
        assert_eq!(kind_rank(EntityKind::Unknown), 3);
        assert_eq!(
            present_string(&FieldPresence::Present {
                value: "Alpha".to_owned(),
                source: Some(source_ref("$.entities[0].name")),
            }),
            Some("Alpha")
        );
        assert_eq!(present_string(&left.observed_name), None);
    }

    #[test]
    fn connected_backfill_should_update_nickname_even_when_name_is_already_observed() {
        // Arrange
        let root = json!({
            "events": [[1, "connected", "Connected Alpha", 10]]
        });
        let raw = RawReplay::new(root.as_object().expect("test root should be an object"));
        let context = context();
        let mut entities = vec![entity(
            10,
            EntityKind::Unit,
            FieldPresence::Present {
                value: "Existing Alpha".to_owned(),
                source: Some(source_ref("$.entities[0].name")),
            },
            FieldPresence::Present {
                value: "rifleman".to_owned(),
                source: Some(source_ref("$.entities[0].class")),
            },
            "$.entities[0]",
        )];

        // Act
        apply_connected_player_backfill(raw, &context, &mut entities);

        // Assert
        assert!(matches!(
            &entities[0].observed_name,
            FieldPresence::Present { value, .. } if value == "Existing Alpha"
        ));
        assert!(matches!(
            &entities[0].identity.nickname,
            FieldPresence::Inferred { value, .. } if value == "Connected Alpha"
        ));
    }

    #[test]
    fn entities_section_diagnostic_should_ignore_present_arrays() {
        // Arrange
        let root = json!({ "entities": [] });
        let raw = RawReplay::new(root.as_object().expect("test root should be an object"));
        let context = context();
        let mut diagnostics = DiagnosticAccumulator::new(8);

        // Act
        push_entities_section_diagnostic(raw, &context, &mut diagnostics);

        // Assert
        assert!(diagnostics.finish(&context).diagnostics.is_empty());
    }
}
