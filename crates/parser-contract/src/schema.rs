use schemars::Schema;
use serde_json::{Value, json};

use crate::{
    aggregates::{
        BountyInputContributionValue, LegacyCounterContributionValue,
        RelationshipContributionValue, VehicleScoreInputValue,
    },
    artifact::ParseArtifact,
};
use serde_json::map::Entry;

/// Builds the JSON Schema for the current parse artifact contract.
#[must_use]
pub fn parse_artifact_schema() -> Schema {
    let mut schema = schemars::schema_for!(ParseArtifact);
    enforce_status_failure_invariants(&mut schema);
    enforce_source_ref_evidence_invariants(&mut schema);
    add_aggregate_value_helper_definitions(&mut schema);
    schema
}

fn add_aggregate_value_helper_definitions(schema: &mut Schema) {
    add_schema_definition::<LegacyCounterContributionValue>(
        schema,
        "LegacyCounterContributionValue",
    );
    add_schema_definition::<RelationshipContributionValue>(schema, "RelationshipContributionValue");
    add_schema_definition::<BountyInputContributionValue>(schema, "BountyInputContributionValue");
    add_schema_definition::<VehicleScoreInputValue>(schema, "VehicleScoreInputValue");
    add_aggregate_projection_key_definition(schema);
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
                "bounty.inputs",
                "vehicle_score.inputs",
                "vehicle_score.denominator_inputs"
            ]
        }),
    ));
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
                    "properties": {
                        "failure": { "type": "null" }
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
