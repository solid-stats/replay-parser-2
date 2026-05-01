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
    minimal::KillClassification,
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde::Deserialize;
use serde_json::json;

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

#[test]
fn golden_fixture_behavior_should_return_failed_artifact_when_fixture_is_malformed() {
    let artifact = parse_manifest_category("malformed");

    assert_eq!(artifact.status, ParseStatus::Failed);
    assert!(artifact.failure.is_some());
    assert!(artifact.players.is_empty());
    assert!(artifact.player_stats.is_empty());
    assert!(artifact.kills.is_empty());
    assert!(artifact.destroyed_vehicles.is_empty());
}

#[test]
fn golden_fixture_behavior_should_return_partial_diagnostics_when_schema_drift_is_present() {
    let artifact = parse_manifest_category("partial_schema_drift");

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "schema.metadata_field" && diagnostic.parser_action == "set_unknown"
    }));
}

#[test]
fn golden_fixture_behavior_should_preserve_old_shape_cases_without_panicking() {
    let artifact = parse_manifest_category("old_shape");

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(!artifact.kills.is_empty());
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "event.killed_shape_unknown")
    );
}

#[test]
fn golden_fixture_behavior_should_emit_destroyed_vehicle_rows_for_vehicle_kill_fixture() {
    let artifact = parse_manifest_category("vehicle_kill");

    assert_eq!(artifact.status, ParseStatus::Success);
    assert!(!artifact.destroyed_vehicles.is_empty());
    assert!(artifact.destroyed_vehicles.iter().any(|row| row.attacker_source_entity_id.is_some()));
}

#[test]
fn golden_fixture_behavior_should_exclude_teamkill_from_bounty_awards() {
    let artifact = parse_manifest_category("teamkill");
    let teamkill_row = artifact
        .kills
        .iter()
        .find(|row| row.classification == KillClassification::Teamkill)
        .expect("teamkill row should exist");

    assert!(!teamkill_row.bounty_eligible);
    assert_eq!(teamkill_row.bounty_exclusion_reasons, vec!["teamkill"]);
}

#[test]
fn golden_fixture_behavior_should_keep_side_facts_empty_in_default_minimal_artifact() {
    let winner_present = parse_manifest_category("winner_present");
    let commander_side = parse_manifest_category("commander_side");
    let serialized =
        serde_json::to_string(&commander_side).expect("default artifact should serialize");

    assert!(winner_present.side_facts.commanders.is_empty());
    assert!(commander_side.side_facts.commanders.is_empty());
    assert!(!serialized.contains("source_refs"));
}

#[test]
fn golden_fixture_behavior_should_preserve_null_killer_semantics() {
    let artifact = parse_manifest_category("null_killer");

    assert!(artifact.kills.iter().any(|row| row.classification == KillClassification::NullKiller
        && row.bounty_exclusion_reasons == vec!["null_killer"]));
}

#[test]
fn golden_fixture_behavior_should_preserve_duplicate_slot_players() {
    let artifact = parse_manifest_category("duplicate_slot_same_name");
    let duplicate_players = artifact
        .players
        .iter()
        .filter(|player| player.compatibility_key == "legacy_name:SameName")
        .count();

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(artifact.players.len(), 3);
    assert_eq!(duplicate_players, 2);
}

#[test]
fn golden_fixture_behavior_should_preserve_connected_player_backfill_name() {
    let artifact = parse_manifest_category("connected_player_backfill");
    let player = artifact
        .players
        .iter()
        .find(|participant| participant.source_entity_id == 11)
        .expect("backfilled player should exist");

    assert_eq!(player.observed_name.as_deref(), Some("BackfilledName"));
}
