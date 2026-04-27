//! Replay metadata and observed identity contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    identity::{
        EntityCompatibilityHint, EntityCompatibilityHintKind, EntityKind, EntitySide,
        ObservedEntity, ObservedIdentity,
    },
    metadata::{FrameBounds, ReplayMetadata, ReplayTimeBounds},
    presence::{Confidence, FieldPresence, NullReason, UnknownReason},
    source_ref::{RuleId, SourceRef, SourceRefs},
};
use serde_json::json;

const fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present { value, source: None }
}

const fn unknown<T>(reason: UnknownReason) -> FieldPresence<T> {
    FieldPresence::Unknown { reason, source: None }
}

fn observed_identity_fixture() -> ObservedIdentity {
    ObservedIdentity {
        nickname: present("Afganor".to_string()),
        steam_id: unknown(UnknownReason::MissingSteamId),
        side: present(EntitySide::West),
        faction: unknown(UnknownReason::SourceFieldAbsent),
        group: present("Alpha 1-1".to_string()),
        squad: unknown(UnknownReason::SourceFieldAbsent),
        role: present("Rifleman".to_string()),
        description: present("Squad member".to_string()),
    }
}

fn entity_source_ref(entity_id: i64, json_path: &str, rule_id: &str) -> SourceRef {
    SourceRef {
        replay_id: None,
        source_file: None,
        checksum: None,
        frame: None,
        event_index: None,
        entity_id: Some(entity_id),
        json_path: Some(json_path.to_string()),
        rule_id: Some(RuleId::new(rule_id).expect("test rule ID should be valid")),
    }
}

fn entity_source_refs(entity_id: i64, json_path: &str, rule_id: &str) -> SourceRefs {
    SourceRefs::new(vec![entity_source_ref(entity_id, json_path, rule_id)])
        .expect("test source refs should be non-empty")
}

fn observed_entity_fixture() -> ObservedEntity {
    ObservedEntity {
        source_entity_id: 42,
        kind: EntityKind::Unit,
        observed_name: FieldPresence::Present { value: "Afganor".to_string(), source: None },
        observed_class: FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: None,
        },
        is_player: FieldPresence::Present { value: true, source: None },
        identity: observed_identity_fixture(),
        compatibility_hints: Vec::new(),
        source_refs: entity_source_refs(42, "$.entities[0]", "entity.observed"),
    }
}

#[test]
fn field_presence_missing_steam_id_should_serialize_unknown_reason() {
    let steam_id: FieldPresence<String> =
        FieldPresence::Unknown { reason: UnknownReason::MissingSteamId, source: None };

    let serialized = serde_json::to_value(&steam_id).expect("presence should serialize");

    assert_eq!(
        serialized,
        json!({
            "state": "unknown",
            "reason": "missing_steam_id",
            "source": null
        })
    );
}

#[test]
fn replay_metadata_should_serialize_observed_top_level_keys_as_snake_case() {
    let metadata = ReplayMetadata {
        mission_name: present("Operation Solid".to_string()),
        world_name: present("Altis".to_string()),
        mission_author: present("SolidGames".to_string()),
        players_count: present(vec![42, 39]),
        capture_delay: present(2.5),
        end_frame: present(98_765),
        time_bounds: present(ReplayTimeBounds {
            start_seconds: Some(0.0),
            end_seconds: Some(987.65),
        }),
        frame_bounds: present(FrameBounds { start_frame: 0, end_frame: 98_765 }),
    };

    let serialized = serde_json::to_value(&metadata).expect("metadata should serialize");
    let object = serialized.as_object().expect("metadata should serialize as an object");

    for key in [
        "mission_name",
        "world_name",
        "mission_author",
        "players_count",
        "capture_delay",
        "end_frame",
        "time_bounds",
        "frame_bounds",
    ] {
        assert!(object.contains_key(key), "metadata should contain {key}");
    }

    for key in
        ["missionName", "worldName", "missionAuthor", "playersCount", "captureDelay", "endFrame"]
    {
        assert!(!object.contains_key(key), "metadata should not contain {key}");
    }

    assert_eq!(serialized["mission_name"]["value"], "Operation Solid");
    assert_eq!(serialized["world_name"]["value"], "Altis");
    assert_eq!(serialized["mission_author"]["value"], "SolidGames");
    assert_eq!(serialized["players_count"]["value"], json!([42, 39]));
    assert_eq!(serialized["capture_delay"]["value"], 2.5);
    assert_eq!(serialized["end_frame"]["value"], 98_765);
}

#[test]
fn observed_identity_should_preserve_nickname_and_source_entity_id_without_canonical_player_id() {
    let entity = observed_entity_fixture();

    let serialized = serde_json::to_value(&entity).expect("entity should serialize");
    let serialized_text = serialized.to_string();

    assert_eq!(serialized["source_entity_id"], 42);
    assert_eq!(serialized["kind"], "unit");
    assert_eq!(serialized["identity"]["nickname"]["value"], "Afganor");
    assert!(!serialized_text.contains("canonical_player"));
    assert!(!serialized_text.contains("canonical_id"));
    assert!(!serialized_text.contains("account_id"));
    assert!(!serialized_text.contains("user_id"));
}

#[test]
fn observed_entity_should_serialize_name_class_and_non_empty_source_refs() {
    let entity = observed_entity_fixture();

    let serialized = serde_json::to_value(&entity).expect("entity should serialize");

    assert_eq!(serialized["observed_name"]["state"], "present");
    assert_eq!(serialized["observed_name"]["value"], "Afganor");
    assert_eq!(serialized["observed_class"]["state"], "unknown");
    assert_eq!(serialized["is_player"]["state"], "present");
    assert_eq!(serialized["is_player"]["value"], true);
    assert_eq!(serialized["source_refs"][0]["json_path"], "$.entities[0]");
}

#[test]
fn observed_entity_should_serialize_duplicate_slot_compatibility_hint_without_merging_entities() {
    let hint = EntityCompatibilityHint {
        kind: EntityCompatibilityHintKind::DuplicateSlotSameName,
        related_entity_ids: vec![41, 42],
        observed_name: FieldPresence::Present { value: "SameName".to_string(), source: None },
        rule_id: RuleId::new("entity.duplicate_slot_same_name")
            .expect("test rule ID should be valid"),
        source_refs: entity_source_refs(42, "$.entities[0]", "entity.duplicate_slot_same_name"),
    };
    let entity = ObservedEntity { compatibility_hints: vec![hint], ..observed_entity_fixture() };

    let serialized = serde_json::to_value(&entity).expect("entity should serialize");

    assert_eq!(serialized["compatibility_hints"][0]["kind"], "duplicate_slot_same_name");
    assert_eq!(serialized["compatibility_hints"][0]["related_entity_ids"], json!([41, 42]));
}

#[test]
fn observed_identity_should_represent_missing_steam_id_as_explicit_unknown_state() {
    let identity = observed_identity_fixture();

    let serialized = serde_json::to_value(&identity).expect("identity should serialize");

    assert_eq!(serialized["steam_id"]["state"], "unknown");
    assert_eq!(serialized["steam_id"]["reason"], "missing_steam_id");
}

#[test]
fn observed_identity_should_preserve_group_and_role_fields_when_observed() {
    let identity = observed_identity_fixture();

    let serialized = serde_json::to_value(&identity).expect("identity should serialize");

    assert_eq!(serialized["group"]["state"], "present");
    assert_eq!(serialized["group"]["value"], "Alpha 1-1");
    assert_eq!(serialized["role"]["state"], "present");
    assert_eq!(serialized["role"]["value"], "Rifleman");
}

#[test]
fn field_presence_null_killer_should_serialize_explicit_null_reason() {
    let killer: FieldPresence<i64> =
        FieldPresence::ExplicitNull { reason: NullReason::NullKiller, source: None };

    let serialized = serde_json::to_value(&killer).expect("presence should serialize");

    assert_eq!(
        serialized,
        json!({
            "state": "explicit_null",
            "reason": "null_killer",
            "source": null
        })
    );
}

#[test]
fn field_presence_inferred_value_should_serialize_confidence_and_rule_id() {
    let side = FieldPresence::Inferred {
        value: "west".to_string(),
        reason: "connected event side backfill".to_string(),
        confidence: Some(Confidence::new(0.75).expect("test confidence should be valid")),
        source: None,
        rule_id: RuleId::new("identity.side.inferred").expect("test rule ID should be non-empty"),
    };

    let serialized = serde_json::to_value(&side).expect("presence should serialize");

    assert_eq!(
        serialized,
        json!({
            "state": "inferred",
            "value": "west",
            "reason": "connected event side backfill",
            "confidence": 0.75,
            "source": null,
            "rule_id": "identity.side.inferred"
        })
    );
}

#[test]
fn field_presence_inferred_confidence_should_accept_values_between_zero_and_one() {
    for value in [0.0, 0.75, 1.0] {
        let confidence = Confidence::new(value).expect("confidence should be valid");

        assert_eq!(confidence.get().to_bits(), value.to_bits());
    }
}

#[test]
fn field_presence_inferred_confidence_should_reject_values_outside_zero_to_one() {
    for value in [-0.1, 1.1, f32::NAN] {
        assert!(Confidence::new(value).is_err(), "{value:?} should be rejected");
    }
}
