use schemars::Schema;
use serde_json::{Value, json};

use crate::artifact::ParseArtifact;

pub fn parse_artifact_schema() -> Schema {
    let mut schema = schemars::schema_for!(ParseArtifact);
    enforce_status_failure_invariants(&mut schema);
    enforce_source_ref_evidence_invariants(&mut schema);
    schema
}

fn enforce_status_failure_invariants(schema: &mut Schema) {
    schema.insert(
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
    );
}

fn enforce_source_ref_evidence_invariants(schema: &mut Schema) {
    let Some(Value::Object(defs)) = schema.get_mut("$defs") else {
        return;
    };
    let Some(Value::Object(source_ref_schema)) = defs.get_mut("SourceRef") else {
        return;
    };

    source_ref_schema.insert(
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
    );
}
