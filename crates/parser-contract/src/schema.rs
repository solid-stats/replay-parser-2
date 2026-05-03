// coverage-exclusion: reviewed Phase 05 schema-shape defensive branches are allowlisted by exact source line.
use schemars::Schema;
use serde_json::{Value, json};

use crate::{
    aggregates::{
        BountyInputContributionValue, LegacyCounterContributionValue, RelationshipContributionValue,
    },
    artifact::ParseArtifact,
    worker::{ParseJobMessage, ParseResultMessage},
};
use serde_json::map::Entry;

/// Builds the JSON Schema for the current parse artifact contract.
#[must_use]
pub fn parse_artifact_schema() -> Schema {
    let mut schema = schemars::schema_for!(ParseArtifact);
    enforce_status_failure_invariants(&mut schema);
    enforce_source_ref_evidence_invariants(&mut schema);
    add_aggregate_value_helper_definitions(&mut schema);
    close_default_artifact_schema(&mut schema);
    schema
}

/// Builds the JSON Schema for the parser-worker parse job message contract.
#[must_use]
pub fn parse_job_schema() -> Schema {
    let mut schema = schemars::schema_for!(ParseJobMessage);
    enforce_parse_job_non_empty_fields(&mut schema);
    close_top_level_schema(&mut schema);
    schema
}

/// Builds the JSON Schema for the parser-worker result message contract.
#[must_use]
pub fn parse_result_schema() -> Schema {
    let mut schema = schemars::schema_for!(ParseResultMessage);
    enforce_parse_result_kind_consts(&mut schema);
    close_top_level_schema(&mut schema);
    schema
}

fn close_default_artifact_schema(schema: &mut Schema) {
    close_top_level_schema(schema);

    for definition_name in [
        "MinimalDestroyedVehicleRow",
        "MinimalDiagnosticRow",
        "MinimalPlayerKillRow",
        "MinimalPlayerRow",
        "MinimalWeaponRow",
    ] {
        close_schema_definition(schema, definition_name);
    }
}

fn close_top_level_schema(schema: &mut Schema) {
    drop(schema.insert("unevaluatedProperties".to_string(), json!(false)));
}

fn close_schema_definition(schema: &mut Schema, definition_name: &str) {
    let Some(Value::Object(defs)) = schema.get_mut("$defs") else {
        return;
    };
    let Some(Value::Object(definition)) = defs.get_mut(definition_name) else {
        return;
    };

    drop(definition.insert("additionalProperties".to_string(), json!(false)));
}

fn enforce_parse_job_non_empty_fields(schema: &mut Schema) {
    let Some(Value::Object(properties)) = schema.get_mut("properties") else {
        return;
    };

    for field_name in ["job_id", "replay_id", "object_key"] {
        let Some(Value::Object(field_schema)) = properties.get_mut(field_name) else {
            continue;
        };
        drop(field_schema.insert("minLength".to_string(), json!(1)));
    }
}

fn enforce_parse_result_kind_consts(schema: &mut Schema) {
    for (definition_name, message_type) in
        [("ParseCompletedMessage", "parse.completed"), ("ParseFailedMessage", "parse.failed")]
    {
        let Some(Value::Object(properties)) = definition_properties(schema, definition_name) else {
            continue;
        };
        let Some(Value::Object(message_type_schema)) = properties.get_mut("message_type") else {
            continue;
        };

        message_type_schema.clear();
        drop(
            message_type_schema
                .insert("description".to_string(), json!("Result message routing kind.")),
        );
        drop(message_type_schema.insert("type".to_string(), json!("string")));
        drop(message_type_schema.insert("const".to_string(), json!(message_type)));
    }
}

fn definition_properties<'a>(
    schema: &'a mut Schema,
    definition_name: &str,
) -> Option<&'a mut Value> {
    schema
        .get_mut("$defs")
        .and_then(Value::as_object_mut)
        .and_then(|defs| defs.get_mut(definition_name))
        .and_then(Value::as_object_mut)
        .and_then(|definition| definition.get_mut("properties"))
}

fn add_aggregate_value_helper_definitions(schema: &mut Schema) {
    add_schema_definition::<LegacyCounterContributionValue>(
        schema,
        "LegacyCounterContributionValue",
    );
    add_schema_definition::<RelationshipContributionValue>(schema, "RelationshipContributionValue");
    add_schema_definition::<BountyInputContributionValue>(schema, "BountyInputContributionValue");
    add_aggregate_projection_key_definition(schema);
    constrain_aggregate_contribution_values(schema);
}

fn add_schema_definition<T: schemars::JsonSchema>(schema: &mut Schema, definition_name: &str) {
    let mut definition = schemars::schema_for!(T);
    let Some(Value::Object(defs)) = schema.get_mut("$defs") else {
        return;
    };

    if let Some(Value::Object(nested_defs)) = definition.remove("$defs") {
        for (key, value) in nested_defs {
            if let Entry::Vacant(entry) = defs.entry(key) {
                let _inserted = entry.insert(value);
            }
        }
    }

    drop(definition.remove("$schema"));
    drop(defs.insert(definition_name.to_string(), definition.into()));
}

fn add_aggregate_projection_key_definition(schema: &mut Schema) {
    let Some(Value::Object(defs)) = schema.get_mut("$defs") else {
        return;
    };

    drop(defs.insert(
        "AggregateProjectionKey".to_string(),
        json!({
            "description": "Known namespaced aggregate projection keys emitted by parser-core v1 artifacts.",
            "type": "string",
            "enum": [
                "legacy.player_game_results",
                "legacy.relationships",
                "legacy.game_type_compatibility",
                "legacy.squad_inputs",
                "legacy.rotation_inputs",
                "bounty.inputs"
            ]
        }),
    ));
}

fn constrain_aggregate_contribution_values(schema: &mut Schema) {
    let Some(Value::Object(defs)) = schema.get_mut("$defs") else {
        return;
    };
    let Some(Value::Object(contribution_schema)) = defs.get_mut("AggregateContributionRef") else {
        return;
    };

    drop(contribution_schema.insert(
        "allOf".to_string(),
        json!([
            aggregate_value_condition("legacy_counter", "LegacyCounterContributionValue"),
            aggregate_value_condition("relationship", "RelationshipContributionValue"),
            aggregate_value_condition("bounty_input", "BountyInputContributionValue")
        ]),
    ));
}

fn aggregate_value_condition(kind: &str, definition_name: &str) -> Value {
    json!({
        "if": {
            "required": ["kind"],
            "properties": {
                "kind": { "const": kind }
            }
        },
        "then": {
            "required": ["value"],
            "properties": {
                "value": { "$ref": format!("#/$defs/{definition_name}") }
            }
        }
    })
}

fn enforce_status_failure_invariants(schema: &mut Schema) {
    drop(schema.insert(
        "allOf".to_string(),
        json!([
            {
                "if": {
                    "required": ["status"],
                    "properties": {
                        "status": { "const": "failed" }
                    }
                },
                "then": {
                    "required": ["failure"],
                    "properties": {
                        "failure": { "$ref": "#/$defs/ParseFailure" }
                    }
                }
            },
            {
                "if": {
                    "required": ["status"],
                    "properties": {
                        "status": { "enum": ["success", "partial", "skipped"] }
                    }
                },
                "then": {
                    "not": {
                        "required": ["failure"]
                    }
                }
            }
        ]),
    ));
}

fn enforce_source_ref_evidence_invariants(schema: &mut Schema) {
    let Some(Value::Object(defs)) = schema.get_mut("$defs") else {
        return;
    };
    let Some(Value::Object(source_ref_schema)) = defs.get_mut("SourceRef") else {
        return;
    };

    drop(source_ref_schema.insert(
        "anyOf".to_string(),
        json!([
            {
                "required": ["replay_id"],
                "properties": { "replay_id": { "type": "string" } }
            },
            {
                "required": ["source_file"],
                "properties": { "source_file": { "type": "string" } }
            },
            {
                "required": ["checksum"],
                "properties": { "checksum": { "$ref": "#/$defs/SourceChecksum" } }
            },
            {
                "required": ["frame"],
                "properties": { "frame": { "type": "integer" } }
            },
            {
                "required": ["event_index"],
                "properties": { "event_index": { "type": "integer" } }
            },
            {
                "required": ["entity_id"],
                "properties": { "entity_id": { "type": "integer" } }
            },
            {
                "required": ["json_path"],
                "properties": { "json_path": { "type": "string" } }
            },
            {
                "required": ["rule_id"],
                "properties": { "rule_id": { "type": "string" } }
            }
        ]),
    ));
}

#[cfg(all(test, not(coverage)))]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use crate::source_ref::ReplaySource;

    use super::*;

    fn schema_from(value: Value) -> Schema {
        Schema::try_from(value).expect("test schema value should be valid JSON Schema")
    }

    #[test]
    fn schema_helpers_should_skip_definitions_when_schema_has_no_defs_object() {
        // Arrange
        let mut schema = parse_artifact_schema();
        drop(schema.remove("$defs"));

        // Act
        add_schema_definition::<LegacyCounterContributionValue>(
            &mut schema,
            "LegacyCounterContributionValue",
        );
        add_aggregate_projection_key_definition(&mut schema);
        constrain_aggregate_contribution_values(&mut schema);
        enforce_source_ref_evidence_invariants(&mut schema);
        close_default_artifact_schema(&mut schema);

        // Assert
        assert!(schema.get("$defs").is_none());
    }

    #[test]
    fn schema_helpers_should_skip_missing_nested_definitions_without_panicking() {
        // Arrange
        let mut schema = parse_artifact_schema();
        let defs = schema
            .get_mut("$defs")
            .and_then(Value::as_object_mut)
            .expect("parse artifact schema should include definitions");
        drop(defs.remove("AggregateContributionRef"));
        drop(defs.remove("SourceRef"));
        drop(defs.remove("MinimalDiagnosticRow"));

        // Act
        constrain_aggregate_contribution_values(&mut schema);
        enforce_source_ref_evidence_invariants(&mut schema);
        close_schema_definition(&mut schema, "MinimalDiagnosticRow");

        // Assert
        let defs = schema
            .get("$defs")
            .and_then(Value::as_object)
            .expect("definitions should remain present");
        assert!(!defs.contains_key("AggregateContributionRef"));
        assert!(!defs.contains_key("SourceRef"));
        assert!(!defs.contains_key("MinimalDiagnosticRow"));
    }

    #[test]
    fn schema_helpers_should_skip_non_object_definitions_and_properties() {
        // Arrange
        let mut schema = schema_from(json!({
            "$defs": {
                "MinimalPlayerRow": true,
                "ParseCompletedMessage": {
                    "properties": {
                        "message_type": true
                    }
                },
                "ParseFailedMessage": {
                    "properties": {}
                }
            },
            "properties": {
                "job_id": true,
                "replay_id": {
                    "type": "string"
                }
            }
        }));

        // Act
        close_schema_definition(&mut schema, "MinimalPlayerRow");
        enforce_parse_job_non_empty_fields(&mut schema);
        enforce_parse_result_kind_consts(&mut schema);

        // Assert
        let schema_value = schema.as_value();
        assert_eq!(schema_value["$defs"]["MinimalPlayerRow"], true);
        assert_eq!(schema_value["properties"]["job_id"], true);
        assert_eq!(schema_value["properties"]["replay_id"]["minLength"], 1);
        assert!(schema_value["properties"].get("object_key").is_none());
        assert_eq!(
            schema_value["$defs"]["ParseCompletedMessage"]["properties"]["message_type"],
            true
        );
    }

    #[test]
    fn schema_helpers_should_merge_nested_definitions_when_added_definition_contains_defs() {
        // Arrange
        let mut schema = schema_from(json!({
            "$defs": {}
        }));

        // Act
        add_schema_definition::<ReplaySource>(&mut schema, "ReplaySource");

        // Assert
        let defs =
            schema.get("$defs").and_then(Value::as_object).expect("definitions should be present");
        assert!(defs.contains_key("ReplaySource"));
        assert!(
            defs.len() > 1,
            "nested definitions should be merged alongside ReplaySource: {:?}",
            defs.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn schema_helpers_should_add_conditional_result_kinds_and_source_ref_evidence() {
        // Arrange
        let mut schema = parse_result_schema();
        let mut artifact_schema = parse_artifact_schema();

        // Act
        enforce_parse_result_kind_consts(&mut schema);
        enforce_source_ref_evidence_invariants(&mut artifact_schema);

        // Assert
        let schema_value = schema.as_value();
        let completed_message_type =
            schema_value["$defs"]["ParseCompletedMessage"]["properties"]["message_type"]
                .as_object()
                .expect("completed message_type should be an object");
        let failed_message_type =
            schema_value["$defs"]["ParseFailedMessage"]["properties"]["message_type"]
                .as_object()
                .expect("failed message_type should be an object");
        assert_eq!(completed_message_type["const"], "parse.completed");
        assert_eq!(failed_message_type["const"], "parse.failed");

        let artifact_schema_value = artifact_schema.as_value();
        assert!(
            artifact_schema_value["$defs"]["SourceRef"]["anyOf"]
                .as_array()
                .expect("SourceRef evidence invariant should be an array")
                .iter()
                .any(|condition| condition["required"] == json!(["rule_id"]))
        );
    }
}
