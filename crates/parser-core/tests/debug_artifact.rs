//! Parser-core deterministic debug artifact tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    presence::FieldPresence,
    source_ref::{ReplaySource, RuleId, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay, parse_replay_debug};
use serde_json::json;

const AGGREGATE_FIXTURE: &[u8] = include_bytes!("fixtures/aggregate-combat.ocap.json");
const COMBAT_EVENTS_FIXTURE: &[u8] = include_bytes!("fixtures/combat-events.ocap.json");
const VEHICLE_CONTEXT_FIXTURE: &[u8] = include_bytes!("fixtures/vehicle-context.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source(source_file: &str) -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-debug".to_string()),
        source_file: source_file.to_string(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "9999999999999999999999999999999999999999999999999999999999999999",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

fn parser_input_for(bytes: &'static [u8], source_file: &str) -> ParserInput<'static> {
    ParserInput {
        bytes,
        source: replay_source(source_file),
        parser: parser_info(),
        options: ParserOptions::default(),
    }
}

fn parser_input() -> ParserInput<'static> {
    parser_input_for(AGGREGATE_FIXTURE, "fixtures/aggregate-combat.ocap.json")
}

#[test]
fn debug_artifact_should_keep_full_detail_out_of_default_artifact() {
    let default_artifact = parse_replay(parser_input());
    let debug_artifact = parse_replay_debug(parser_input());

    let default_json =
        serde_json::to_value(default_artifact).expect("default artifact should serialize");
    let default_root = default_json.as_object().expect("default artifact should be an object");
    let default_serialized =
        serde_json::to_string(&default_json).expect("default artifact should stringify");

    assert!(default_root.contains_key("players"));
    assert!(default_root.contains_key("weapons"));
    assert!(default_root.contains_key("destroyed_vehicles"));
    assert!(!default_root.contains_key("kills"));
    assert!(!default_root.contains_key("player_stats"));
    assert!(!default_root.contains_key("entities"));
    assert!(!default_root.contains_key("events"));
    assert!(!default_serialized.contains("\"source_refs\""));
    assert!(!default_serialized.contains("killer_name"));
    assert!(!default_serialized.contains("victim_name"));
    assert!(!default_serialized.contains("attacker_vehicle_name"));
    assert!(!default_serialized.contains("destroyed_name"));

    assert!(!debug_artifact.entities.is_empty());
    assert!(!debug_artifact.events.is_empty());
    assert!(debug_artifact.replay.is_some());
}

#[test]
fn debug_artifact_should_serialize_full_provenance_and_rule_context() {
    let debug_artifact = parse_replay_debug(parser_input());
    let debug_json = serde_json::to_value(debug_artifact).expect("debug artifact should serialize");
    let debug_root = debug_json.as_object().expect("debug artifact should be an object");
    let debug_serialized =
        serde_json::to_string(&debug_json).expect("debug artifact should stringify");

    assert!(debug_root.contains_key("entities"));
    assert!(debug_root.contains_key("events"));
    assert!(debug_root.contains_key("side_facts"));
    assert!(debug_serialized.contains("\"source_refs\""));
    assert!(debug_serialized.contains("\"rule_id\""));
    assert!(debug_serialized.contains("\"event_index\""));
    assert!(debug_serialized.contains("\"frame\""));
}

#[test]
fn debug_artifact_should_serialize_identically_when_same_input_is_parsed_twice() {
    let first_artifact = parse_replay_debug(parser_input());
    let second_artifact = parse_replay_debug(parser_input());

    let first_serialized =
        serde_json::to_string(&first_artifact).expect("first debug artifact should serialize");
    let second_serialized =
        serde_json::to_string(&second_artifact).expect("second debug artifact should serialize");

    assert_eq!(first_serialized, second_serialized);
}

#[test]
fn debug_artifact_should_return_empty_debug_shell_when_json_is_invalid() {
    let debug_artifact =
        parse_replay_debug(parser_input_for(b"{", "fixtures/invalid-json.ocap.json"));

    assert_eq!(debug_artifact.source.source_file, "fixtures/invalid-json.ocap.json");
    assert!(debug_artifact.replay.is_none());
    assert!(debug_artifact.entities.is_empty());
    assert!(debug_artifact.events.is_empty());
    assert!(debug_artifact.side_facts.commanders.is_empty());
    assert!(debug_artifact.diagnostics.is_empty());
}

#[test]
fn debug_artifact_should_keep_event_rule_ids_consistent_with_source_refs() {
    for (fixture, source_file) in [
        (COMBAT_EVENTS_FIXTURE, "fixtures/combat-events.ocap.json"),
        (VEHICLE_CONTEXT_FIXTURE, "fixtures/vehicle-context.ocap.json"),
    ] {
        let debug_artifact = parse_replay_debug(parser_input_for(fixture, source_file));

        for event in debug_artifact.events {
            for source_ref in event.source_refs.as_slice() {
                assert_eq!(
                    source_ref.rule_id.as_ref().map(RuleId::as_str),
                    Some(event.rule_id.as_str())
                );
            }
        }
    }
}
