//! Golden fixture manifest coverage tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{collections::BTreeSet, path::Path};

use serde::Deserialize;

const MANIFEST: &str = include_str!("fixtures/golden/manifest.json");
const REQUIRED_CATEGORIES: [&str; 12] = [
    "normal",
    "malformed",
    "partial_schema_drift",
    "old_shape",
    "winner_present",
    "winner_missing",
    "vehicle_kill",
    "teamkill",
    "commander_side",
    "null_killer",
    "duplicate_slot_same_name",
    "connected_player_backfill",
];
const PHASE5_REQUIREMENTS: [&str; 6] =
    ["TEST-01", "TEST-03", "TEST-08", "TEST-09", "TEST-10", "TEST-11"];

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    fixture: String,
    fixture_strategy: String,
    category: String,
    source: FixtureSource,
    requirements: Vec<String>,
    decisions: Vec<String>,
    expected_status: String,
    expected_features: Vec<String>,
    cross_app_impact: CrossAppImpact,
}

#[derive(Debug, Deserialize)]
struct FixtureSource {
    artifact: String,
    source_file: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
struct CrossAppImpact {
    parser_artifact: String,
    server_2: String,
    web: String,
}

fn manifest_entries() -> Vec<ManifestEntry> {
    serde_json::from_str(MANIFEST).expect("golden fixture manifest should deserialize")
}

#[test]
fn golden_fixture_manifest_should_include_every_required_phase_5_category() {
    // Arrange
    let entries = manifest_entries();

    // Act
    let categories = entries.iter().map(|entry| entry.category.as_str()).collect::<BTreeSet<_>>();

    // Assert
    for category in REQUIRED_CATEGORIES {
        assert!(categories.contains(category), "manifest should contain category {category}");
    }
}

#[test]
fn golden_fixture_manifest_should_cover_every_phase_5_fixture_requirement() {
    // Arrange
    let entries = manifest_entries();

    // Act
    let covered_requirements = entries
        .iter()
        .flat_map(|entry| entry.requirements.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();

    // Assert
    for requirement in PHASE5_REQUIREMENTS {
        assert!(
            covered_requirements.contains(requirement),
            "manifest should cover requirement {requirement}"
        );
    }
}

#[test]
fn golden_fixture_manifest_should_keep_entries_traceable_and_executable() {
    // Arrange
    let entries = manifest_entries();
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Act + Assert
    for entry in &entries {
        let fixture_path = crate_root.join(&entry.fixture);
        let has_source_reference =
            !entry.source.artifact.is_empty() && !entry.source.source_file.is_empty();
        let has_decision_reference = !entry.decisions.is_empty();

        assert!(fixture_path.is_file(), "fixture path should exist: {}", entry.fixture);
        assert_eq!(entry.fixture_strategy, "linked_existing_focused_fixture");
        assert!(
            has_source_reference || has_decision_reference,
            "entry {} should include a source or decision reference",
            entry.category
        );
        assert!(!entry.source.notes.is_empty(), "entry {} should explain source", entry.category);
        assert!(
            ["success", "partial", "failed"].contains(&entry.expected_status.as_str()),
            "entry {} should use a known expected status",
            entry.category
        );
        assert!(
            !entry.expected_features.is_empty(),
            "entry {} should declare expected features",
            entry.category
        );
        assert!(
            !entry.cross_app_impact.parser_artifact.is_empty()
                && !entry.cross_app_impact.server_2.is_empty()
                && !entry.cross_app_impact.web.is_empty(),
            "entry {} should include downstream impact notes",
            entry.category
        );
    }
}
