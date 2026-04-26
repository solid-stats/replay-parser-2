use std::{fs, path::PathBuf};

use parser_contract::schema::parse_artifact_schema;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under crates/")
        .parent()
        .expect("crates/ should live under the workspace root")
        .to_path_buf()
}

fn committed_schema_path() -> PathBuf {
    workspace_root().join("schemas/parse-artifact-v1.schema.json")
}

fn freshly_generated_schema_text() -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(&parse_artifact_schema())
            .expect("parse artifact schema should serialize")
    )
}

#[test]
fn schema_contract_committed_parse_artifact_schema_should_exist() {
    assert!(
        committed_schema_path().is_file(),
        "schemas/parse-artifact-v1.schema.json should be committed"
    );
}

#[test]
fn schema_contract_committed_schema_should_name_parse_artifact_and_contract_fields() {
    let schema_text =
        fs::read_to_string(committed_schema_path()).expect("committed schema should be readable");

    assert!(schema_text.contains("ParseArtifact"));
    for expected_field in [
        "contract_version",
        "parser",
        "status",
        "diagnostics",
        "replay",
        "entities",
        "events",
        "aggregates",
        "failure",
    ] {
        assert!(
            schema_text.contains(expected_field),
            "schema should contain {expected_field}"
        );
    }
}

#[test]
fn schema_contract_committed_schema_should_match_fresh_generation() {
    let committed_schema_text =
        fs::read_to_string(committed_schema_path()).expect("committed schema should be readable");
    let fresh_schema_text = freshly_generated_schema_text();

    assert_eq!(committed_schema_text, fresh_schema_text);
}
