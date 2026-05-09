//! Comparison report vocabulary tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_harness::{
    comparison::compare_artifacts,
    report::{
        ComparisonBaseline, ComparisonFinding, ComparisonInput, ComparisonReport, ImpactAssessment,
        ImpactLevel, MismatchCategory, ReportValidationError,
    },
    summary_report::{ComparisonReviewSummary, render_markdown_summary},
};
use serde_json::{Value, json};

#[test]
fn comparison_report_mismatch_categories_should_serialize_as_phase_1_snake_case_values() {
    // Arrange
    let cases = [
        (MismatchCategory::Compatible, "compatible"),
        (MismatchCategory::IntentionalChange, "intentional_change"),
        (MismatchCategory::OldBugPreserved, "old_bug_preserved"),
        (MismatchCategory::OldBugFixed, "old_bug_fixed"),
        (MismatchCategory::NewBug, "new_bug"),
        (MismatchCategory::InsufficientData, "insufficient_data"),
        (MismatchCategory::HumanReview, "human_review"),
    ];

    // Act + Assert
    for (category, expected) in cases {
        let serialized = serde_json::to_value(category)
            .expect("mismatch category should serialize to JSON value");
        assert_eq!(serialized, Value::String(expected.to_owned()));
    }
}

#[test]
fn comparison_report_mismatch_categories_should_return_stable_string_values() {
    // Arrange
    let cases = [
        (MismatchCategory::Compatible, "compatible"),
        (MismatchCategory::IntentionalChange, "intentional_change"),
        (MismatchCategory::OldBugPreserved, "old_bug_preserved"),
        (MismatchCategory::OldBugFixed, "old_bug_fixed"),
        (MismatchCategory::NewBug, "new_bug"),
        (MismatchCategory::InsufficientData, "insufficient_data"),
        (MismatchCategory::HumanReview, "human_review"),
    ];

    // Act + Assert
    for (category, expected) in cases {
        assert_eq!(category.as_str(), expected);
        assert_eq!(category.to_string(), expected);
    }
}

#[test]
fn comparison_report_impact_assessment_should_require_every_downstream_dimension() {
    // Arrange + Act
    let error = ImpactAssessment::try_new(
        Some(ImpactLevel::Yes),
        Some(ImpactLevel::No),
        None,
        Some(ImpactLevel::Unknown),
    )
    .expect_err("missing server_2_recalculation should fail validation");

    // Assert
    assert_eq!(
        error,
        ReportValidationError::MissingImpactDimension { dimension: "server_2_recalculation" }
    );
}

#[test]
fn comparison_report_impact_assessment_should_accept_complete_dimensions() {
    // Arrange + Act
    let assessment = ImpactAssessment::try_new(
        Some(ImpactLevel::Yes),
        Some(ImpactLevel::No),
        Some(ImpactLevel::Unknown),
        Some(ImpactLevel::Yes),
    )
    .expect("complete impact assessment should pass validation");

    // Assert
    assert_eq!(assessment.parser_artifact, ImpactLevel::Yes);
    assert_eq!(assessment.server_2_persistence, ImpactLevel::No);
    assert_eq!(assessment.server_2_recalculation, ImpactLevel::Unknown);
    assert_eq!(assessment.ui_visible_public_stats, ImpactLevel::Yes);
}

#[test]
fn comparison_report_impact_assessment_should_report_each_missing_dimension() {
    // Arrange
    let cases = [
        (
            ImpactAssessment::try_new(
                None,
                Some(ImpactLevel::No),
                Some(ImpactLevel::Unknown),
                Some(ImpactLevel::Yes),
            ),
            "parser_artifact",
        ),
        (
            ImpactAssessment::try_new(
                Some(ImpactLevel::Yes),
                None,
                Some(ImpactLevel::Unknown),
                Some(ImpactLevel::No),
            ),
            "server_2_persistence",
        ),
        (
            ImpactAssessment::try_new(
                Some(ImpactLevel::Yes),
                Some(ImpactLevel::No),
                Some(ImpactLevel::Unknown),
                None,
            ),
            "ui_visible_public_stats",
        ),
    ];

    // Act + Assert
    for (result, expected_dimension) in cases {
        assert_eq!(
            result.expect_err("missing impact dimension should fail validation"),
            ReportValidationError::MissingImpactDimension { dimension: expected_dimension }
        );
    }
}

#[test]
fn comparison_report_current_vs_regenerated_drift_should_remain_human_review() {
    // Arrange
    let baseline = ComparisonBaseline {
        old_profile: "current-results-vs-worker-count-1".to_owned(),
        old_command: "pnpm run parse".to_owned(),
        worker_count: Some(1),
        source: "current_vs_regenerated_drift".to_owned(),
        diagnostic_only: false,
    };
    let finding = ComparisonFinding::new(
        "results.digest",
        None,
        MismatchCategory::Compatible,
        ImpactAssessment::unknown(),
        json!("old"),
        json!("new"),
    );

    // Act
    let error = ComparisonReport::new(
        baseline,
        vec![ComparisonInput::new("old", "~/sg_stats/results")],
        vec![finding],
    )
    .expect_err("unexplained drift cannot be classified before human review");

    // Assert
    assert_eq!(
        error,
        ReportValidationError::BaselineDriftMustRemainHumanReview {
            surface: "results.digest".to_owned(),
            category: MismatchCategory::Compatible,
        }
    );
}

#[test]
fn comparison_report_compare_artifacts_should_emit_compatible_findings_when_selected_surfaces_match()
 {
    // Arrange
    let old_json = selected_artifact_json("success");
    let new_json = selected_artifact_json("success");

    // Act
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("matching selected artifacts should produce a report");

    // Assert
    assert!(report.findings.iter().any(|finding| finding.category == MismatchCategory::Compatible));
    assert_eq!(report.summary.by_category["compatible"], 5);
}

#[test]
fn comparison_report_compare_artifacts_should_derive_legacy_view_from_minimal_tables() {
    // Arrange
    let old_json = legacy_surface_artifact_json("success");
    let new_json = minimal_artifact_json("success");

    // Act
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("minimal artifact should derive legacy comparison surfaces");

    // Assert
    let player_results = finding(&report, "legacy.player_game_results");
    assert_eq!(player_results.category, MismatchCategory::Compatible);
    assert_eq!(player_results.new_value[0]["killsFromVehicle"], 1);

    let relationships = finding(&report, "legacy.relationships");
    assert_eq!(relationships.category, MismatchCategory::Compatible);
    assert_eq!(relationships.new_value["killed"][0]["relationship"], "killed");
    assert_eq!(relationships.new_value["teamkillers"][0]["relationship"], "teamkillers");

    let bounty_inputs = finding(&report, "bounty.inputs");
    assert_eq!(bounty_inputs.category, MismatchCategory::Compatible);
    assert_eq!(bounty_inputs.new_value[0]["weapon"], "M2");
    assert_eq!(bounty_inputs.new_value[0]["attacker_vehicle_class"], "offroad_hmg");
}

#[test]
fn comparison_report_compare_artifacts_should_use_top_level_kills_when_present() {
    // Arrange
    let old_json = legacy_surface_artifact_json("success");
    let mut new_value: Value = serde_json::from_slice(&minimal_artifact_json("success"))
        .expect("minimal artifact fixture should deserialize");
    new_value["kills"] = json!([
        {
            "k": 1,
            "v": 2,
            "c": "enemy_kill",
            "w": 2,
            "av": 20,
            "avc": "offroad_hmg"
        },
        {
            "k": 1,
            "v": 3,
            "c": "teamkill",
            "w": 1
        }
    ]);
    new_value["players"][0]["kills"] = json!([
        {
            "v": 99,
            "c": "enemy_kill"
        }
    ]);
    let new_json =
        serde_json::to_vec(&new_value).expect("modified minimal fixture should serialize");

    // Act
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("top-level kills should drive derived legacy surfaces when present");

    // Assert
    let relationships = finding(&report, "legacy.relationships");
    assert_eq!(relationships.category, MismatchCategory::Compatible);
    assert_eq!(relationships.new_value["killed"][0]["target_entity_id"], 2);
}

#[test]
fn comparison_report_compare_artifacts_should_fallback_minimal_player_identity_fields() {
    // Arrange
    let new_json = serde_json::to_vec(&json!({
        "status": "success",
        "replay": {
            "mission_name": "SolidGames"
        },
        "players": [
            {
                "eid": 1,
                "eids": [1, 10],
                "s": "guer",
                "k": 1,
                "kills": [
                    {
                        "v": 2,
                        "c": "enemy_kill"
                    }
                ]
            },
            {
                "eid": 2,
                "n": "Civilian",
                "s": "civ",
                "d": 1
            },
            {
                "eid": 3,
                "n": "Unknown",
                "s": "unknown"
            }
        ],
        "weapons": [],
        "destroyed_vehicles": []
    }))
    .expect("fallback fixture should serialize");
    let old_json = new_json.clone();

    // Act
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("minimal identity fallbacks should derive comparable legacy surfaces");

    // Assert
    let player_results = finding(&report, "legacy.player_game_results");
    assert_eq!(player_results.category, MismatchCategory::Compatible);
    assert_eq!(player_results.new_value[0]["compatibility_key"], "entity:1");
    assert_eq!(player_results.new_value[0]["observed_entity_ids"], json!([1, 10]));
    assert_eq!(player_results.new_value[0]["side"], "guer");
    assert_eq!(player_results.new_value[1]["compatibility_key"], "legacy_name:Civilian");
    assert_eq!(player_results.new_value[1]["side"], "civ");
    assert_eq!(player_results.new_value[2]["side"], "unknown");
}

#[test]
fn comparison_report_compare_artifacts_should_ignore_relationships_with_missing_player_refs() {
    // Arrange
    let new_json = serde_json::to_vec(&json!({
        "status": "success",
        "replay": {
            "mission_name": "SolidGames"
        },
        "players": [
            {
                "eid": 1,
                "n": "Alpha",
                "k": 2,
                "kills": [
                    {
                        "v": 2,
                        "c": "suicide"
                    },
                    {
                        "v": 3,
                        "c": "null_killer"
                    },
                    {
                        "v": 4,
                        "c": "unknown"
                    }
                ]
            }
        ],
        "kills": [
            {
                "k": 1,
                "c": "enemy_kill"
            },
            {
                "v": 1,
                "c": "teamkill"
            }
        ],
        "weapons": [],
        "destroyed_vehicles": [
            {
                "a": 1,
                "c": "enemy"
            }
        ]
    }))
    .expect("missing-ref fixture should serialize");
    let old_json = new_json.clone();

    // Act
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("missing relationship refs should be ignored without failing comparison");

    // Assert
    let player_results = finding(&report, "legacy.player_game_results");
    assert_eq!(player_results.category, MismatchCategory::Compatible);
    assert_eq!(player_results.new_value[0]["vehicleKills"], 1);
    let relationships = finding(&report, "legacy.relationships");
    assert_eq!(relationships.new_value["killed"], json!([]));
    assert_eq!(relationships.new_value["teamkillers"], json!([]));
}

#[test]
fn comparison_report_compare_artifacts_should_emit_insufficient_data_when_selected_surface_is_missing()
 {
    // Arrange
    let old_json = br#"{"status":"success"}"#;
    let new_json = br#"{"status":"success","events":[]}"#;

    // Act
    let report =
        compare_artifacts("old-selected-artifact", old_json, "new-selected-artifact", new_json)
            .expect("structurally different selected artifacts should produce a report");

    // Assert
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.category == MismatchCategory::InsufficientData)
    );
}

#[test]
fn comparison_report_compare_artifacts_should_emit_human_review_when_baseline_label_marks_regenerated_drift()
 {
    // Arrange
    let old_json = selected_artifact_json("success");
    let new_json = selected_artifact_json("success");

    // Act
    let report = compare_artifacts(
        "current-vs-regenerated-drift worker-count-1",
        &old_json,
        "new-selected-artifact",
        &new_json,
    )
    .expect("baseline drift reports should be held for human review");

    // Assert
    assert!(
        report.findings.iter().all(|finding| finding.category == MismatchCategory::HumanReview)
    );
}

#[test]
fn comparison_report_markdown_summary_should_include_required_review_sections() {
    // Arrange
    let old_json = selected_artifact_json("success");
    let new_json = selected_artifact_json("failed");
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("different selected artifacts should produce a report");

    // Act
    let markdown = render_markdown_summary(&report);

    // Assert
    assert!(markdown.starts_with("# Comparison Summary"));
    assert!(markdown.contains("## Counts by Category"));
    assert!(markdown.contains("## Counts by Impact"));
    assert!(markdown.contains("## Top Diffs"));
    assert!(markdown.contains("## Next Action"));
    assert!(markdown.contains("Review top human_review diffs before accepting parity"));
}

#[test]
fn comparison_report_markdown_summary_should_surface_derived_legacy_top_diffs() {
    // Arrange
    let old_json = legacy_surface_artifact_json("success");
    let mut new_value: Value = serde_json::from_slice(&minimal_artifact_json("success"))
        .expect("minimal artifact fixture should deserialize");
    new_value["players"][0]["k"] = json!(2);
    let new_json =
        serde_json::to_vec(&new_value).expect("modified minimal fixture should serialize");
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("different derived legacy surfaces should produce a report");

    // Act
    let markdown = render_markdown_summary(&report);

    // Assert
    assert!(markdown.contains("- `compatible`: 4"));
    assert!(markdown.contains("- `human_review`: 1"));
    assert!(markdown.contains("`legacy.player_game_results`: `human_review`"));
}

#[test]
fn comparison_report_review_summary_should_record_human_review_next_action() {
    // Arrange
    let old_json = selected_artifact_json("success");
    let new_json = selected_artifact_json("failed");
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("different selected artifacts should produce a report");

    // Act
    let summary = ComparisonReviewSummary::from_report(&report);

    // Assert
    assert_eq!(summary.next_action, "Review top human_review diffs before accepting parity.");
    assert!(
        summary.top_diffs.iter().any(|diff| diff.note.contains("Review top human_review diffs"))
    );
}

#[test]
fn comparison_report_review_summary_should_not_list_compatible_surfaces_as_top_diffs() {
    // Arrange
    let old_json = selected_artifact_json("success");
    let new_json = selected_artifact_json("success");
    let report =
        compare_artifacts("old-selected-artifact", &old_json, "new-selected-artifact", &new_json)
            .expect("matching selected artifacts should produce a report");

    // Act
    let summary = ComparisonReviewSummary::from_report(&report);
    let markdown = render_markdown_summary(&report);

    // Assert
    assert!(summary.top_diffs.is_empty());
    assert!(markdown.contains("## Top Diffs\n\n- None"));
}

#[test]
fn comparison_report_review_summary_should_prioritize_categories_and_notes() {
    // Arrange
    let baseline = ComparisonBaseline {
        old_profile: "old-selected-artifact".to_owned(),
        old_command: "pnpm run parse".to_owned(),
        worker_count: Some(1),
        source: "selected".to_owned(),
        diagnostic_only: false,
    };
    let impact = ImpactAssessment::new(
        ImpactLevel::Yes,
        ImpactLevel::No,
        ImpactLevel::Unknown,
        ImpactLevel::Yes,
    );
    let mut noted_new_bug = ComparisonFinding::new(
        "new-bug",
        None,
        MismatchCategory::NewBug,
        impact,
        json!(1),
        json!(2),
    );
    noted_new_bug.notes.push("fix parser regression".to_owned());
    let findings = vec![
        ComparisonFinding::new(
            "compatible",
            None,
            MismatchCategory::Compatible,
            impact,
            json!(1),
            json!(1),
        ),
        ComparisonFinding::new(
            "old-bug-preserved",
            None,
            MismatchCategory::OldBugPreserved,
            impact,
            json!(1),
            json!(2),
        ),
        noted_new_bug,
        ComparisonFinding::new(
            "intentional",
            None,
            MismatchCategory::IntentionalChange,
            impact,
            json!(1),
            json!(2),
        ),
        ComparisonFinding::new(
            "old-bug-fixed",
            None,
            MismatchCategory::OldBugFixed,
            impact,
            json!(1),
            json!(2),
        ),
    ];
    let report = ComparisonReport::new(
        baseline,
        vec![ComparisonInput::new("old", "~/sg_stats/results")],
        findings,
    )
    .expect("non-drift report should accept mixed categories");

    // Act
    let summary = ComparisonReviewSummary::from_report(&report);
    let markdown = render_markdown_summary(&report);

    // Assert
    assert_eq!(summary.top_diffs[0].surface, "new-bug");
    assert_eq!(summary.top_diffs[0].note, "fix parser regression");
    assert_eq!(summary.top_diffs[1].surface, "intentional");
    assert!(markdown.contains("fix parser regression"));
    assert!(markdown.contains("- `parser_artifact.yes`: 5"));
    assert!(markdown.contains("- `server_2_recalculation.unknown`: 5"));
}

#[test]
fn comparison_report_markdown_summary_should_render_empty_count_sections() {
    // Arrange
    let baseline = ComparisonBaseline::saved_artifact("old-selected-artifact");
    let report = ComparisonReport::new(
        baseline,
        vec![ComparisonInput::new("old", "~/sg_stats/results")],
        Vec::new(),
    )
    .expect("empty comparison report should be renderable");

    // Act
    let markdown = render_markdown_summary(&report);

    // Assert
    assert!(markdown.contains("## Counts by Category\n\n- None"));
    assert!(markdown.contains("## Top Diffs\n\n- None"));
}

fn selected_artifact_json(status: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "status": status,
        "replay": {
            "mission_name": "SolidGames"
        },
        "legacy": {
            "player_game_results": [],
            "relationships": {
                "killed": [],
                "killers": [],
                "teamkilled": [],
                "teamkillers": []
            }
        },
        "bounty": {
            "inputs": []
        }
    }))
    .expect("selected fixture JSON should serialize")
}

fn finding<'a>(report: &'a ComparisonReport, surface: &str) -> &'a ComparisonFinding {
    report
        .findings
        .iter()
        .find(|finding| finding.surface == surface)
        .expect("requested surface finding should exist")
}

fn legacy_surface_artifact_json(status: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "status": status,
        "replay": {
            "mission_name": "SolidGames"
        },
        "legacy": expected_legacy_surfaces(),
        "bounty": {
            "inputs": expected_bounty_inputs()
        }
    }))
    .expect("legacy surface fixture JSON should serialize")
}

#[allow(
    clippy::too_many_lines,
    reason = "integration fixture JSON stays inline so old/new parity expectations are reviewable"
)]
fn minimal_artifact_json(status: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "status": status,
        "replay": {
            "mission_name": "SolidGames"
        },
        "players": [
            {
                "eid": 1,
                "n": "Alpha",
                "s": "west",
                "g": "Alpha 1-1",
                "r": "Rifleman",
                "ck": "legacy_name:Alpha",
                "k": 1,
                "tk": 1,
                "vk": 1,
                "kfv": 1,
                "kills": [
                    {
                        "v": 2,
                        "c": "enemy_kill",
                        "w": 2,
                        "av": 20,
                        "avc": "offroad_hmg"
                    },
                    {
                        "v": 3,
                        "c": "teamkill",
                        "w": 1
                    }
                ]
            },
            {
                "eid": 2,
                "n": "Bravo",
                "s": "east",
                "g": "Bravo 1-1",
                "r": "Rifleman",
                "ck": "legacy_name:Bravo",
                "d": 1
            },
            {
                "eid": 3,
                "n": "Charlie",
                "s": "west",
                "g": "Alpha 1-2",
                "r": "Rifleman",
                "ck": "legacy_name:Charlie",
                "d": 1,
                "td": 1
            }
        ],
        "weapons": [
            {
                "id": 1,
                "n": "AK-74"
            },
            {
                "id": 2,
                "n": "M2"
            },
            {
                "id": 3,
                "n": "RPG-7"
            }
        ],
        "destroyed_vehicles": [
            {
                "a": 1,
                "c": "enemy",
                "w": 3,
                "de": 30,
                "dt": "vehicle",
                "dc": "apc"
            }
        ]
    }))
    .expect("minimal fixture JSON should serialize")
}

fn expected_legacy_surfaces() -> Value {
    json!({
        "player_game_results": [
            {
                "compatibility_key": "legacy_name:Alpha",
                "observed_entity_ids": [1],
                "observed_name": "Alpha",
                "observed_tag": null,
                "side": "west",
                "kills": 1,
                "killsFromVehicle": 1,
                "vehicleKills": 1,
                "teamkills": 1,
                "isDead": false,
                "isDeadByTeamkill": false,
                "deaths": {
                    "total": 0,
                    "byTeamkills": 0
                },
                "kdRatio": 0.0,
                "killsFromVehicleCoef": 1.0,
                "score": 0.0,
                "totalPlayedGames": 1
            },
            {
                "compatibility_key": "legacy_name:Bravo",
                "observed_entity_ids": [2],
                "observed_name": "Bravo",
                "observed_tag": null,
                "side": "east",
                "kills": 0,
                "killsFromVehicle": 0,
                "vehicleKills": 0,
                "teamkills": 0,
                "isDead": true,
                "isDeadByTeamkill": false,
                "deaths": {
                    "total": 1,
                    "byTeamkills": 0
                },
                "kdRatio": 0.0,
                "killsFromVehicleCoef": 0.0,
                "score": 0.0,
                "totalPlayedGames": 1
            },
            {
                "compatibility_key": "legacy_name:Charlie",
                "observed_entity_ids": [3],
                "observed_name": "Charlie",
                "observed_tag": null,
                "side": "west",
                "kills": 0,
                "killsFromVehicle": 0,
                "vehicleKills": 0,
                "teamkills": 0,
                "isDead": true,
                "isDeadByTeamkill": true,
                "deaths": {
                    "total": 1,
                    "byTeamkills": 1
                },
                "kdRatio": 0.0,
                "killsFromVehicleCoef": 0.0,
                "score": 0.0,
                "totalPlayedGames": 1
            }
        ],
        "relationships": {
            "killed": [
                relationship_row("killed", "Alpha", 1, "Bravo", 2)
            ],
            "killers": [
                relationship_row("killers", "Bravo", 2, "Alpha", 1)
            ],
            "teamkilled": [
                relationship_row("teamkilled", "Alpha", 1, "Charlie", 3)
            ],
            "teamkillers": [
                relationship_row("teamkillers", "Charlie", 3, "Alpha", 1)
            ]
        }
    })
}

fn expected_bounty_inputs() -> Value {
    json!([
        {
            "killer_player_id": "entity:1",
            "killer_source_entity_id": 1,
            "killer_compatibility_key": "legacy_name:Alpha",
            "killer_side": "west",
            "victim_player_id": "entity:2",
            "victim_source_entity_id": 2,
            "victim_compatibility_key": "legacy_name:Bravo",
            "victim_side": "east",
            "weapon": "M2",
            "attacker_vehicle_entity_id": 20,
            "attacker_vehicle_class": "offroad_hmg"
        }
    ])
}

fn relationship_row(
    relationship: &str,
    source_name: &str,
    source_id: i64,
    target_name: &str,
    target_id: i64,
) -> Value {
    json!({
        "relationship": relationship,
        "source_player_id": format!("entity:{source_id}"),
        "source_entity_id": source_id,
        "source_compatibility_key": format!("legacy_name:{source_name}"),
        "source_observed_entity_ids": [source_id],
        "source_observed_name": source_name,
        "source_observed_tag": null,
        "target_player_id": format!("entity:{target_id}"),
        "target_entity_id": target_id,
        "target_compatibility_key": format!("legacy_name:{target_name}"),
        "target_observed_entity_ids": [target_id],
        "target_observed_name": target_name,
        "target_observed_tag": null,
        "count": 1
    })
}
