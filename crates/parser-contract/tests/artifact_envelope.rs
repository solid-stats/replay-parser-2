use std::collections::BTreeMap;

use parser_contract::{
    aggregates::AggregateSection,
    artifact::{ParseArtifact, ParseStatus},
    source_ref::{ReplaySource, SourceChecksum},
    version::{ContractVersion, ParserInfo},
};
use semver::Version;
use serde_json::{Value, json};

fn parser_info() -> ParserInfo {
    ParserInfo {
        name: "replay-parser-2".to_string(),
        version: Version::parse("0.1.0").expect("test parser version should be valid"),
        build: None,
    }
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-0001".to_string()),
        source_file: "2025_04_05__23_27_21__1_ocap.json".to_string(),
        checksum: SourceChecksum {
            algorithm: "sha256".to_string(),
            value: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        },
    }
}

fn success_artifact() -> ParseArtifact {
    ParseArtifact {
        contract_version: ContractVersion::current(),
        parser: parser_info(),
        source: replay_source(),
        status: ParseStatus::Success,
        produced_at: None,
        diagnostics: Vec::new(),
        replay: None,
        entities: Vec::new(),
        events: Vec::new(),
        aggregates: AggregateSection::default(),
        failure: None,
        extensions: BTreeMap::new(),
    }
}

#[test]
fn artifact_envelope_parse_status_should_serialize_exact_status_values_for_every_parser_outcome() {
    let statuses = json!([
        ParseStatus::Success,
        ParseStatus::Partial,
        ParseStatus::Skipped,
        ParseStatus::Failed
    ]);

    assert_eq!(statuses, json!(["success", "partial", "skipped", "failed"]));
}

#[test]
fn artifact_envelope_parse_artifact_should_serialize_unified_envelope_fields_with_deterministic_extensions()
 {
    let mut artifact = success_artifact();
    artifact
        .extensions
        .insert("zeta".to_string(), Value::String("last".to_string()));
    artifact
        .extensions
        .insert("alpha".to_string(), Value::String("first".to_string()));

    let serialized = serde_json::to_value(&artifact).expect("artifact should serialize");

    assert_eq!(serialized["contract_version"], "1.0.0");
    assert_eq!(serialized["parser"]["name"], "replay-parser-2");
    assert_eq!(
        serialized["source"]["source_file"],
        "2025_04_05__23_27_21__1_ocap.json"
    );
    assert_eq!(serialized["source"]["checksum"]["algorithm"], "sha256");
    assert_eq!(serialized["status"], "success");
    assert!(serialized.get("produced_at").is_some());
    assert!(serialized.get("diagnostics").is_some());
    assert!(serialized.get("replay").is_some());
    assert!(serialized.get("entities").is_some());
    assert!(serialized.get("events").is_some());
    assert!(serialized.get("aggregates").is_some());
    assert!(serialized.get("failure").is_some());

    let extension_keys = serialized["extensions"]
        .as_object()
        .expect("extensions should serialize as an object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        extension_keys,
        vec!["alpha".to_string(), "zeta".to_string()]
    );
}
