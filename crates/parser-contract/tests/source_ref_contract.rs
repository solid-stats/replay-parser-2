use parser_contract::source_ref::{RuleId, SourceRef};
use serde_json::json;

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
