//! Parser-core raw event accessor behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    presence::FieldPresence,
    source_ref::{ReplaySource, RuleId, SourceChecksum},
};
use parser_core::{
    artifact::SourceContext,
    raw::{KilledEventKillInfo, KilledEventObservation, RawReplay, killed_events},
};
use serde_json::Value;

const KILLED_EVENTS_FIXTURE: &[u8] = include_bytes!("fixtures/killed-events.ocap.json");

fn killed_observations() -> Vec<KilledEventObservation> {
    let value = serde_json::from_slice::<Value>(KILLED_EVENTS_FIXTURE)
        .expect("killed events fixture should be valid JSON");
    let root = value.as_object().expect("killed events fixture should be a root object");
    let raw = RawReplay::new(root);

    killed_events(raw)
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-killed-events".to_string()),
        source_file: "fixtures/killed-events.ocap.json".to_string(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

#[test]
fn raw_event_accessors_should_read_killed_event_killer_weapon_and_distance() {
    let observations = killed_observations();

    let first_observation =
        observations.first().expect("first killed event observation should exist");

    assert_eq!(observations.len(), 5);
    assert_eq!(first_observation.event_index, 0);
    assert_eq!(first_observation.frame, Some(10));
    assert_eq!(first_observation.killed_entity_id, Some(2));
    assert!(matches!(
        &first_observation.kill_info,
        KilledEventKillInfo::Killer {
            killer_entity_id: 1,
            weapon: Some(weapon),
        } if weapon == "AK-74"
    ));
    assert_eq!(first_observation.distance_meters.map(f64::to_bits), Some(100.0_f64.to_bits()));
    assert_eq!(first_observation.json_path, "$.events[0]");

    let empty_weapon_observation =
        observations.get(2).expect("empty-weapon killed event observation should exist");
    assert!(matches!(
        &empty_weapon_observation.kill_info,
        KilledEventKillInfo::Killer { killer_entity_id: 1, weapon: None }
    ));
    assert_eq!(
        empty_weapon_observation.distance_meters.map(f64::to_bits),
        Some(12.0_f64.to_bits())
    );
}

#[test]
fn raw_event_accessors_should_read_null_killer_event() {
    let observations = killed_observations();

    let null_killer_observation =
        observations.get(1).expect("null-killer killed event observation should exist");

    assert_eq!(null_killer_observation.event_index, 1);
    assert_eq!(null_killer_observation.frame, Some(20));
    assert_eq!(null_killer_observation.killed_entity_id, Some(3));
    assert_eq!(null_killer_observation.kill_info, KilledEventKillInfo::NullKiller);
}

#[test]
fn raw_event_accessors_should_preserve_malformed_kill_info_without_panicking() {
    let observations = killed_observations();

    let malformed_observation =
        observations.get(3).expect("malformed killed event observation should exist");

    assert_eq!(malformed_observation.event_index, 3);
    assert_eq!(malformed_observation.frame, Some(40));
    assert_eq!(malformed_observation.killed_entity_id, None);
    assert!(matches!(
        &malformed_observation.kill_info,
        KilledEventKillInfo::Malformed { observed_shape } if observed_shape == "object"
    ));
    assert_eq!(malformed_observation.distance_meters, None);
}

#[test]
fn raw_event_accessors_should_preserve_killed_event_when_frame_is_malformed() {
    let observations = killed_observations();

    let malformed_frame_observation =
        observations.get(4).expect("malformed-frame killed event observation should exist");

    assert_eq!(malformed_frame_observation.event_index, 4);
    assert_eq!(malformed_frame_observation.frame, None);
    assert_eq!(malformed_frame_observation.killed_entity_id, Some(2));
    assert!(matches!(
        &malformed_frame_observation.kill_info,
        KilledEventKillInfo::Killer {
            killer_entity_id: 1,
            weapon: Some(weapon),
        } if weapon == "AK-74"
    ));
}

#[test]
fn raw_event_accessors_should_ignore_non_killed_events() {
    let observations = killed_observations();

    let connected_event_was_ignored = observations
        .iter()
        .all(|observation| observation.event_index != 5 && observation.json_path != "$.events[5]");

    assert_eq!(observations.len(), 5);
    assert!(connected_event_was_ignored);
}

#[test]
fn raw_event_accessors_should_build_event_source_ref_with_coordinates() {
    let source = replay_source();
    let context = SourceContext::new(&source);
    let rule_id =
        RuleId::new("event.killed.observed").expect("test event source rule ID should be valid");

    let source_ref =
        context.event_source_ref("$.events[0]", Some(10), Some(0), Some(2), Some(rule_id));

    assert_eq!(source_ref.replay_id.as_deref(), Some("replay-killed-events"));
    assert_eq!(source_ref.source_file.as_deref(), Some("fixtures/killed-events.ocap.json"));
    assert_eq!(source_ref.frame, Some(10));
    assert_eq!(source_ref.event_index, Some(0));
    assert_eq!(source_ref.entity_id, Some(2));
    assert_eq!(source_ref.json_path.as_deref(), Some("$.events[0]"));
    assert_eq!(source_ref.rule_id.as_ref().map(RuleId::as_str), Some("event.killed.observed"));
}
