//! Aggregate contribution value contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    aggregates::{
        AggregateContributionKind, AggregateContributionRef, BountyInputContributionValue,
    },
    source_ref::{RuleId, SourceChecksum, SourceRef, SourceRefs},
};

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
        entity_id: Some(101),
        json_path: Some("$.events[7]".to_string()),
        rule_id: Some(RuleId::new(rule_id).expect("test source rule ID should be valid")),
    }
}

fn bounty_input_value() -> BountyInputContributionValue {
    BountyInputContributionValue {
        killer_entity_id: 101,
        victim_entity_id: 202,
        killer_side: "west".to_string(),
        victim_side: "east".to_string(),
        frame: Some(12_345),
        event_id: "event-0007".to_string(),
        eligible: true,
        exclusion_reasons: Vec::new(),
    }
}

#[test]
fn aggregate_contract_should_serialize_bounty_input_evidence_when_enemy_kill_is_eligible() {
    let value = bounty_input_value();

    let serialized = serde_json::to_value(value).expect("bounty input value should serialize");
    let serialized_text =
        serde_json::to_string(&serialized).expect("bounty input JSON should stringify");

    assert_eq!(serialized["killer_entity_id"], 101);
    assert_eq!(serialized["victim_entity_id"], 202);
    assert_eq!(serialized["eligible"], true);
    for expected_fragment in [
        "\"killer_entity_id\":101",
        "\"victim_entity_id\":202",
        "\"killer_side\":\"west\"",
        "\"victim_side\":\"east\"",
        "\"eligible\":true",
    ] {
        assert!(
            serialized_text.contains(expected_fragment),
            "bounty input JSON should contain {expected_fragment}"
        );
    }
}

#[test]
fn aggregate_contract_should_wrap_bounty_input_value_in_source_backed_contribution_ref() {
    let value =
        serde_json::to_value(bounty_input_value()).expect("bounty input value should serialize");
    let contribution = AggregateContributionRef {
        contribution_id: "contribution-bounty-0007".to_string(),
        kind: AggregateContributionKind::BountyInput,
        event_id: Some("event-0007".to_string()),
        source_refs: SourceRefs::new(vec![source_ref("aggregate.bounty.source")])
            .expect("source refs should be non-empty"),
        rule_id: RuleId::new("aggregate.bounty.input")
            .expect("test contribution rule ID should be valid"),
        value,
    };

    let serialized = serde_json::to_value(contribution).expect("contribution should serialize");

    assert_eq!(serialized["kind"], "bounty_input");
    assert_eq!(serialized["event_id"], "event-0007");
    assert_eq!(serialized["source_refs"][0]["rule_id"], "aggregate.bounty.source");
    assert_eq!(serialized["rule_id"], "aggregate.bounty.input");
    assert_eq!(serialized["value"]["eligible"], true);
    assert_eq!(serialized["value"]["killer_entity_id"], 101);
}

#[test]
fn aggregate_contract_should_serialize_bounty_input_kind_as_snake_case_when_kind_is_bounty_input() {
    let serialized = serde_json::to_value(AggregateContributionKind::BountyInput)
        .expect("aggregate contribution kind should serialize");

    assert_eq!(serialized, "bounty_input");
}
