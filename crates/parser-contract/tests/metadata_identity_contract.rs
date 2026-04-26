use parser_contract::{
    metadata::{FrameBounds, ReplayMetadata, ReplayTimeBounds},
    presence::{FieldPresence, NullReason, UnknownReason},
    source_ref::RuleId,
};
use serde_json::json;

fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present {
        value,
        source: None,
    }
}

#[test]
fn field_presence_missing_steam_id_should_serialize_unknown_reason() {
    let steam_id: FieldPresence<String> = FieldPresence::Unknown {
        reason: UnknownReason::MissingSteamId,
        source: None,
    };

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
        frame_bounds: present(FrameBounds {
            start_frame: 0,
            end_frame: 98_765,
        }),
    };

    let serialized = serde_json::to_value(&metadata).expect("metadata should serialize");
    let object = serialized
        .as_object()
        .expect("metadata should serialize as an object");

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

    for key in [
        "missionName",
        "worldName",
        "missionAuthor",
        "playersCount",
        "captureDelay",
        "endFrame",
    ] {
        assert!(
            !object.contains_key(key),
            "metadata should not contain {key}"
        );
    }

    assert_eq!(serialized["mission_name"]["value"], "Operation Solid");
    assert_eq!(serialized["world_name"]["value"], "Altis");
    assert_eq!(serialized["mission_author"]["value"], "SolidGames");
    assert_eq!(serialized["players_count"]["value"], json!([42, 39]));
    assert_eq!(serialized["capture_delay"]["value"], 2.5);
    assert_eq!(serialized["end_frame"]["value"], 98_765);
}

#[test]
fn field_presence_null_killer_should_serialize_explicit_null_reason() {
    let killer: FieldPresence<i64> = FieldPresence::ExplicitNull {
        reason: NullReason::NullKiller,
        source: None,
    };

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
        confidence: Some(0.75),
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
