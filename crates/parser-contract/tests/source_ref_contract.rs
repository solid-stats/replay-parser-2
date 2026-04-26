//! Source reference and aggregate contribution contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeMap;

use parser_contract::{
    aggregates::{AggregateContributionKind, AggregateContributionRef, AggregateSection},
    events::{EventActorRef, NormalizedEvent, NormalizedEventKind},
    identity::EntitySide,
    presence::FieldPresence,
    source_ref::{ChecksumValue, RuleId, SourceChecksum, SourceRef, SourceRefs},
};
use serde_json::{Value, json};

const fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present { value, source: None }
}

fn checksum() -> SourceChecksum {
    SourceChecksum::sha256("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("test checksum should be valid")
}

fn source_ref(rule_id: &str) -> SourceRef {
    SourceRef {
        replay_id: Some("replay-0001".to_string()),
        source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
        checksum: Some(checksum()),
        frame: Some(12_345),
        event_index: Some(7),
        entity_id: Some(99),
        json_path: Some("$.events[7]".to_string()),
        rule_id: Some(RuleId::new(rule_id).expect("test source rule ID should be valid")),
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
        assert!(RuleId::new(invalid_rule_id).is_err(), "{invalid_rule_id:?} should be rejected");
    }
}

#[test]
fn source_ref_contract_checksum_should_accept_only_sha256_lowercase_hex_when_value_is_passed() {
    let checksum =
        SourceChecksum::sha256("0000000000000000000000000000000000000000000000000000000000000000")
            .expect("lowercase sha256 checksum should be valid");

    assert_eq!(
        checksum.value.as_str(),
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert!(ChecksumValue::new("not-a-hash").is_err());
    assert!(
        ChecksumValue::new("ABCDEF0000000000000000000000000000000000000000000000000000000000")
            .is_err()
    );
    assert!(ChecksumValue::new("0000").is_err());
}

#[test]
fn source_ref_contract_source_ref_should_serialize_replay_frame_event_entity_path_and_rule_coordinates()
 {
    let source_ref = SourceRef {
        replay_id: Some("replay-0001".to_string()),
        source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
        checksum: Some(checksum()),
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
            "checksum": {
                "algorithm": "sha256",
                "value": "0000000000000000000000000000000000000000000000000000000000000000"
            },
            "frame": 12345,
            "event_index": 7,
            "entity_id": 42,
            "json_path": "$.events[7]",
            "rule_id": "event.kill.player"
        })
    );
}

#[test]
fn source_ref_contract_source_ref_should_reject_hollow_evidence_when_deserialized() {
    let result = serde_json::from_value::<SourceRef>(json!({}));

    assert!(result.is_err());
    assert!(
        result
            .expect_err("hollow source ref should fail")
            .to_string()
            .contains("source reference must include at least one evidence coordinate")
    );
}

#[test]
fn source_ref_contract_source_refs_should_reject_empty_arrays_when_created() {
    let result = SourceRefs::new(Vec::new());

    assert!(result.is_err());
}

#[test]
fn normalized_event_source_refs_should_serialize_vehicle_killed_event_with_source_coordinates() {
    let attributes =
        BTreeMap::from([("vehicle_class".to_string(), Value::String("rhs_btr80".to_string()))]);
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
        source_refs: SourceRefs::new(vec![source_ref("event.vehicle_killed.source")])
            .expect("source refs should be non-empty"),
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
    assert_eq!(serialized["source_refs"][0]["rule_id"], "event.vehicle_killed.source");
    assert_eq!(serialized["rule_id"], "event.vehicle_killed");
    assert_eq!(serialized["attributes"]["vehicle_class"], "rhs_btr80");
}

#[test]
fn normalized_event_source_refs_should_require_non_empty_source_refs() {
    let result = SourceRefs::new(Vec::new());

    assert!(result.is_err());
}

#[test]
fn aggregate_contribution_refs_should_serialize_vehicle_score_input_with_source_refs() {
    let contribution = AggregateContributionRef {
        contribution_id: "contribution-vehicle-score-0007".to_string(),
        kind: AggregateContributionKind::VehicleScoreInput,
        event_id: Some("event-0007".to_string()),
        source_refs: SourceRefs::new(vec![source_ref("aggregate.vehicle_score.source")])
            .expect("source refs should be non-empty"),
        rule_id: RuleId::new("aggregate.vehicle_score.contribution")
            .expect("test contribution rule ID should be valid"),
        value: json!({
            "weight": 1.5,
            "attacker_vehicle": "apc",
            "killed_entity": "player"
        }),
    };
    let section =
        AggregateSection { contributions: vec![contribution], projections: BTreeMap::new() };

    let serialized = serde_json::to_value(&section).expect("aggregate section should serialize");

    assert_eq!(
        serialized["contributions"][0]["contribution_id"],
        "contribution-vehicle-score-0007"
    );
    assert_eq!(serialized["contributions"][0]["kind"], "vehicle_score_input");
    assert_eq!(serialized["contributions"][0]["event_id"], "event-0007");
    assert!(serialized["contributions"][0]["source_refs"].is_array());
    assert_eq!(
        serialized["contributions"][0]["source_refs"][0]["rule_id"],
        "aggregate.vehicle_score.source"
    );
    assert_eq!(serialized["contributions"][0]["rule_id"], "aggregate.vehicle_score.contribution");
    assert_eq!(serialized["contributions"][0]["value"]["weight"], 1.5);
    assert_eq!(serialized["contributions"][0]["value"]["attacker_vehicle"], "apc");
    assert_eq!(serialized["contributions"][0]["value"]["killed_entity"], "player");
    assert_eq!(
        serialized["projections"]
            .as_object()
            .expect("projections should serialize as an object")
            .len(),
        0
    );
}

#[test]
fn aggregate_contribution_refs_should_require_non_empty_source_refs() {
    let result = SourceRefs::new(Vec::new());

    assert!(result.is_err());
}
