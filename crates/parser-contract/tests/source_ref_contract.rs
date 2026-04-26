use std::collections::BTreeMap;

use parser_contract::{
    events::{EventActorRef, NormalizedEvent, NormalizedEventKind},
    identity::EntitySide,
    presence::FieldPresence,
    source_ref::{RuleId, SourceRef},
};
use serde_json::{Value, json};

fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present {
        value,
        source: None,
    }
}

#[test]
fn source_ref_contract_rule_id_should_accept_stable_namespaced_ids_when_lowercase_dotted_value_is_passed()
 {
    let rule_id = RuleId::new("event.kill.player").expect("namespaced rule ID should be valid");

    assert_eq!(rule_id.as_str(), "event.kill.player");
}

#[test]
fn source_ref_contract_rule_id_should_reject_empty_or_non_namespaced_ids() {
    for invalid_rule_id in [
        "",
        "   ",
        "vehicle_score",
        "Event.Kill.Player",
        "event kill player",
        "event.kill.player!",
        ".event",
        "event.",
    ] {
        assert!(
            RuleId::new(invalid_rule_id).is_err(),
            "{invalid_rule_id:?} should be rejected"
        );
    }
}

#[test]
fn source_ref_contract_source_ref_should_serialize_replay_frame_event_entity_path_and_rule_coordinates()
 {
    let source_ref = SourceRef {
        replay_id: Some("replay-0001".to_string()),
        source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
        checksum: Some("sha256:abc123".to_string()),
        frame: Some(12_345),
        event_index: Some(7),
        entity_id: Some(42),
        json_path: Some("$.events[7]".to_string()),
        rule_id: Some(RuleId::new("event.kill.player").expect("test rule ID should be valid")),
    };

    let serialized = serde_json::to_value(&source_ref).expect("source ref should serialize");

    assert_eq!(
        serialized,
        json!({
            "replay_id": "replay-0001",
            "source_file": "2025_04_05__23_27_21__1_ocap.json",
            "checksum": "sha256:abc123",
            "frame": 12345,
            "event_index": 7,
            "entity_id": 42,
            "json_path": "$.events[7]",
            "rule_id": "event.kill.player"
        })
    );
}

#[test]
fn normalized_event_source_refs_should_serialize_vehicle_killed_event_with_source_coordinates() {
    let attributes = BTreeMap::from([(
        "vehicle_class".to_string(),
        Value::String("rhs_btr80".to_string()),
    )]);
    let event = NormalizedEvent {
        event_id: "event-0007".to_string(),
        kind: NormalizedEventKind::VehicleKilled,
        frame: present(12_345),
        event_index: present(7),
        actors: vec![EventActorRef {
            source_entity_id: present(101),
            observed_name: present("Afganor".to_string()),
            side: present(EntitySide::West),
        }],
        source_refs: vec![SourceRef {
            replay_id: Some("replay-0001".to_string()),
            source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
            checksum: Some("sha256:abc123".to_string()),
            frame: Some(12_345),
            event_index: Some(7),
            entity_id: Some(99),
            json_path: Some("$.events[7]".to_string()),
            rule_id: Some(
                RuleId::new("event.vehicle_killed.source")
                    .expect("test source rule ID should be valid"),
            ),
        }],
        rule_id: RuleId::new("event.vehicle_killed").expect("test event rule ID should be valid"),
        attributes,
    };

    let serialized = serde_json::to_value(&event).expect("normalized event should serialize");

    assert_eq!(serialized["event_id"], "event-0007");
    assert_eq!(serialized["kind"], "vehicle_killed");
    assert_eq!(serialized["frame"]["value"], 12_345);
    assert_eq!(serialized["event_index"]["value"], 7);
    assert_eq!(serialized["actors"][0]["source_entity_id"]["value"], 101);
    assert_eq!(serialized["actors"][0]["observed_name"]["value"], "Afganor");
    assert_eq!(serialized["actors"][0]["side"]["value"], "west");
    assert_eq!(serialized["source_refs"][0]["frame"], 12_345);
    assert_eq!(serialized["source_refs"][0]["event_index"], 7);
    assert_eq!(serialized["source_refs"][0]["entity_id"], 99);
    assert_eq!(serialized["source_refs"][0]["json_path"], "$.events[7]");
    assert_eq!(
        serialized["source_refs"][0]["rule_id"],
        "event.vehicle_killed.source"
    );
    assert_eq!(serialized["rule_id"], "event.vehicle_killed");
    assert_eq!(serialized["attributes"]["vehicle_class"], "rhs_btr80");
}
