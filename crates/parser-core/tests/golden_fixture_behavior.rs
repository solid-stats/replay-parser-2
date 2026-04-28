//! Golden fixture behavior tests through the public parser-core API.

#![allow(
    clippy::expect_used,
    clippy::missing_const_for_fn,
    clippy::panic,
    reason = "integration tests use expect messages as assertion context"
)]

use std::path::Path;

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    events::{BountyExclusionReason, CombatSemantic, NormalizedEvent},
    identity::{EntityCompatibilityHintKind, EntitySide},
    presence::{FieldPresence, UnknownReason},
    side_facts::OutcomeStatus,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde::Deserialize;
use serde_json::{Value, json};

const MANIFEST: &str = include_str!("fixtures/golden/manifest.json");

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    fixture: String,
    category: String,
}

fn manifest_entries() -> Vec<ManifestEntry> {
    serde_json::from_str(MANIFEST).expect("golden fixture manifest should deserialize")
}

fn manifest_entry(category: &str) -> ManifestEntry {
    manifest_entries()
        .into_iter()
        .find(|entry| entry.category == category)
        .unwrap_or_else(|| panic!("manifest category {category} should exist"))
}

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source(entry: &ManifestEntry) -> ReplaySource {
    ReplaySource {
        replay_id: Some(format!("golden-{}", entry.category)),
        source_file: entry.fixture.clone(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "7777777777777777777777777777777777777777777777777777777777777777",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

fn parse_manifest_category(category: &str) -> ParseArtifact {
    let entry = manifest_entry(category);
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&entry.fixture);
    let bytes = std::fs::read(&fixture_path).expect("manifest fixture should be readable");
    let input = ParserInput {
        bytes: &bytes,
        source: replay_source(&entry),
        parser: parser_info(),
        options: ParserOptions::default(),
    };

    parse_replay(input)
}

fn projection_array<'a>(artifact: &'a ParseArtifact, key: &str) -> &'a Vec<Value> {
    artifact
        .aggregates
        .projections
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("projection {key} should be an array"))
}

fn event_by_id<'a>(artifact: &'a ParseArtifact, event_id: &str) -> &'a NormalizedEvent {
    artifact
        .events
        .iter()
        .find(|event| event.event_id == event_id)
        .unwrap_or_else(|| panic!("event {event_id} should exist"))
}

#[test]
fn golden_fixture_behavior_should_return_failed_artifact_when_fixture_is_malformed() {
    // Arrange + Act
    let artifact = parse_manifest_category("malformed");

    // Assert
    assert_eq!(artifact.status, ParseStatus::Failed);
    assert!(artifact.failure.is_some());
    assert!(artifact.events.is_empty());
    assert!(artifact.aggregates.contributions.is_empty());
}

#[test]
fn golden_fixture_behavior_should_return_partial_diagnostics_when_schema_drift_is_present() {
    // Arrange + Act
    let artifact = parse_manifest_category("partial_schema_drift");

    // Assert
    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "schema.metadata_field" && diagnostic.parser_action == "set_unknown"
    }));
}

#[test]
fn golden_fixture_behavior_should_preserve_old_shape_cases_without_panicking() {
    // Arrange + Act
    let artifact = parse_manifest_category("old_shape");

    // Assert
    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(!artifact.events.is_empty());
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "event.killed_shape_unknown")
    );
}

#[test]
fn golden_fixture_behavior_should_emit_vehicle_score_inputs_for_vehicle_kill_fixture() {
    // Arrange + Act
    let artifact = parse_manifest_category("vehicle_kill");
    let vehicle_score_inputs = projection_array(&artifact, "vehicle_score.inputs");

    // Assert
    assert_eq!(artifact.status, ParseStatus::Success);
    assert!(!vehicle_score_inputs.is_empty());
    assert!(vehicle_score_inputs.iter().any(|row| row["event_id"] == "event.killed.0"));
}

#[test]
fn golden_fixture_behavior_should_exclude_teamkill_from_bounty_awards() {
    // Arrange + Act
    let artifact = parse_manifest_category("teamkill");
    let teamkill_event = event_by_id(&artifact, "event.killed.1");
    let teamkill_combat =
        teamkill_event.combat.as_ref().expect("teamkill event should have combat payload");
    let bounty_inputs = projection_array(&artifact, "bounty.inputs");

    // Assert
    assert_eq!(teamkill_combat.semantic, CombatSemantic::Teamkill);
    assert!(teamkill_combat.bounty.exclusion_reasons.contains(&BountyExclusionReason::Teamkill));
    assert!(!bounty_inputs.iter().any(|row| row["event_id"] == "event.killed.1"));
}

#[test]
fn golden_fixture_behavior_should_populate_known_and_unknown_side_facts_for_winner_cases() {
    // Arrange + Act
    let winner_present = parse_manifest_category("winner_present");
    let winner_missing = parse_manifest_category("winner_missing");

    // Assert
    assert_eq!(winner_present.side_facts.outcome.status, OutcomeStatus::Known);
    assert!(matches!(
        winner_present.side_facts.outcome.winner_side,
        FieldPresence::Present { value: EntitySide::West, source: Some(_) }
    ));
    assert_eq!(winner_missing.side_facts.outcome.status, OutcomeStatus::Unknown);
    assert!(matches!(
        winner_missing.side_facts.outcome.winner_side,
        FieldPresence::Unknown { reason: UnknownReason::MissingWinner, source: None }
    ));
}

#[test]
fn golden_fixture_behavior_should_preserve_commander_side_candidate_without_canonical_identity() {
    // Arrange + Act
    let artifact = parse_manifest_category("commander_side");
    let serialized =
        serde_json::to_string(&artifact.side_facts).expect("side facts should serialize");

    // Assert
    assert_eq!(artifact.side_facts.commanders.len(), 1);
    assert!(!serialized.contains("canonical_player_id"));
    assert_eq!(
        artifact.side_facts.commanders[0].rule_id.as_str(),
        "side_facts.commander.keyword_candidate"
    );
    assert!(
        artifact.side_facts.commanders[0]
            .source_refs
            .as_slice()
            .iter()
            .any(|source_ref| source_ref.json_path.as_deref() == Some("$.entities[0]"))
    );
}

#[test]
fn golden_fixture_behavior_should_preserve_null_killer_semantics() {
    // Arrange + Act
    let artifact = parse_manifest_category("null_killer");
    let null_killer_event = event_by_id(&artifact, "event.killed.3");
    let combat =
        null_killer_event.combat.as_ref().expect("null-killer event should have combat payload");

    // Assert
    assert_eq!(combat.semantic, CombatSemantic::NullKillerDeath);
    assert!(combat.bounty.exclusion_reasons.contains(&BountyExclusionReason::NullKiller));
}

#[test]
fn golden_fixture_behavior_should_preserve_duplicate_slot_compatibility_hints() {
    // Arrange + Act
    let artifact = parse_manifest_category("duplicate_slot_same_name");

    // Assert
    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(artifact.entities.len(), 3);
    assert!(
        artifact
            .entities
            .iter()
            .filter(|entity| {
                entity
                    .compatibility_hints
                    .iter()
                    .any(|hint| hint.kind == EntityCompatibilityHintKind::DuplicateSlotSameName)
            })
            .count()
            >= 2
    );
}

#[test]
fn golden_fixture_behavior_should_preserve_connected_player_backfill_facts() {
    // Arrange + Act
    let artifact = parse_manifest_category("connected_player_backfill");
    let player = artifact
        .entities
        .iter()
        .find(|entity| entity.source_entity_id == 11)
        .expect("backfilled player entity should exist");

    // Assert
    assert!(matches!(
        &player.identity.nickname,
        FieldPresence::Inferred { value, rule_id, .. }
            if value == "BackfilledName"
                && rule_id.as_str() == "entity.connected_player_backfill"
    ));
    assert!(
        player
            .compatibility_hints
            .iter()
            .any(|hint| { hint.kind == EntityCompatibilityHintKind::ConnectedPlayerBackfill })
    );
}
