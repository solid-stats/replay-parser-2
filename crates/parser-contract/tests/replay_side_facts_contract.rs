//! Replay-side facts contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    events::EventActorRef,
    identity::EntitySide,
    presence::{Confidence, FieldPresence},
    side_facts::{
        CommanderFactKind, CommanderSideFact, OutcomeFact, OutcomeStatus, ReplaySideFacts,
    },
    source_ref::{RuleId, SourceChecksum, SourceRef, SourceRefs},
};

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
        event_index: None,
        entity_id: Some(101),
        json_path: Some("$.entities[101]".to_string()),
        rule_id: Some(RuleId::new(rule_id).expect("test source rule ID should be valid")),
    }
}

fn commander_actor() -> EventActorRef {
    EventActorRef {
        source_entity_id: present(101),
        observed_name: present("Commander".to_string()),
        side: present(EntitySide::West),
    }
}

#[test]
fn replay_side_facts_contract_default_should_use_unknown_outcome_when_no_facts_are_available() {
    let side_facts = ReplaySideFacts::default();

    assert_eq!(side_facts.commanders, Vec::new());
    assert_eq!(side_facts.outcome.status, OutcomeStatus::Unknown);
}

#[test]
fn replay_side_facts_contract_should_serialize_candidate_commander_with_confidence_and_source_refs()
{
    let fact = CommanderSideFact {
        side: present(EntitySide::West),
        side_name: present("BLUFOR".to_string()),
        commander: present(commander_actor()),
        kind: CommanderFactKind::Candidate,
        confidence: Some(Confidence::new(0.75).expect("test confidence should be valid")),
        rule_id: RuleId::new("side_facts.commander.candidate")
            .expect("test rule ID should be valid"),
        source_refs: SourceRefs::new(vec![source_ref("side_facts.commander.source")])
            .expect("source refs should be non-empty"),
    };

    let serialized = serde_json::to_value(fact).expect("commander fact should serialize");
    let serialized_text =
        serde_json::to_string(&serialized).expect("commander fact JSON should stringify");

    assert_eq!(serialized["kind"], "candidate");
    assert_eq!(serialized["confidence"], 0.75);
    assert_eq!(serialized["commander"]["value"]["observed_name"]["value"], "Commander");
    assert_eq!(serialized["source_refs"][0]["rule_id"], "side_facts.commander.source");
    for expected_fragment in ["\"candidate\"", "\"confidence\"", "\"commander\"", "\"source_refs\""]
    {
        assert!(
            serialized_text.contains(expected_fragment),
            "commander fact JSON should contain {expected_fragment}"
        );
    }
}

#[test]
fn replay_side_facts_contract_should_serialize_known_outcome_with_winner_side() {
    let outcome = OutcomeFact {
        status: OutcomeStatus::Known,
        winner_side: present(EntitySide::West),
        source_label: present("WEST WON".to_string()),
        confidence: None,
        rule_id: RuleId::new("side_facts.outcome.known").expect("test rule ID should be valid"),
        source_refs: Some(
            SourceRefs::new(vec![source_ref("side_facts.outcome.source")])
                .expect("source refs should be non-empty"),
        ),
    };

    let serialized = serde_json::to_value(outcome).expect("outcome fact should serialize");
    let serialized_text =
        serde_json::to_string(&serialized).expect("outcome fact JSON should stringify");

    assert_eq!(serialized["status"], "known");
    assert_eq!(serialized["winner_side"]["value"], "west");
    assert!(serialized_text.contains("\"known\""));
    assert!(serialized_text.contains("\"winner_side\""));
}
