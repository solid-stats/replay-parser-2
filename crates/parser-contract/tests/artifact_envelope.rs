use std::collections::BTreeMap;

use parser_contract::{
    aggregates::AggregateSection,
    artifact::{ParseArtifact, ParseStatus},
    diagnostic::{Diagnostic, DiagnosticSeverity},
    source_ref::{ReplaySource, RuleId, SourceChecksum, SourceRef},
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
fn artifact_envelope_serializes_exact_status_values() {
    let statuses = json!([
        ParseStatus::Success,
        ParseStatus::Partial,
        ParseStatus::Skipped,
        ParseStatus::Failed
    ]);

    assert_eq!(statuses, json!(["success", "partial", "skipped", "failed"]));
}

#[test]
fn artifact_envelope_serializes_unified_fields_with_deterministic_extensions() {
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

#[test]
fn diagnostics_are_path_based_and_do_not_serialize_raw_replay_snippets() {
    let diagnostic = Diagnostic {
        code: "schema.event_shape".to_string(),
        severity: DiagnosticSeverity::Warning,
        message: "Malformed event at events[12] was skipped".to_string(),
        json_path: Some("$.events[12]".to_string()),
        expected_shape: Some("array(frame, kind, entity_id, payload, distance)".to_string()),
        observed_shape: Some("array(frame, kind, string, number)".to_string()),
        parser_action: "skipped_event".to_string(),
        source_refs: vec![SourceRef {
            replay_id: Some("replay-0001".to_string()),
            source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
            checksum: Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            ),
            frame: Some(42),
            event_index: Some(12),
            entity_id: Some(7),
            json_path: Some("$.events[12]".to_string()),
            rule_id: Some(
                RuleId::new("diagnostic.schema_drift").expect("test rule ID should be non-empty"),
            ),
        }],
    };

    let serialized = serde_json::to_value(&diagnostic).expect("diagnostic should serialize");
    let serialized_object = serialized
        .as_object()
        .expect("diagnostic should serialize as an object");

    assert!(serialized_object.contains_key("json_path"));
    assert!(serialized_object.contains_key("expected_shape"));
    assert!(serialized_object.contains_key("observed_shape"));
    assert!(serialized_object.contains_key("parser_action"));
    assert_eq!(
        serialized["source_refs"][0]["rule_id"],
        "diagnostic.schema_drift"
    );
    assert!(!serialized_object.contains_key("raw"));
    assert!(!serialized_object.contains_key("snippet"));
    assert!(!serialized_object.contains_key("raw_value"));
}

#[test]
fn diagnostics_are_path_based_rule_id_should_reject_empty_values() {
    assert!(RuleId::new("").is_err());
    assert!(RuleId::new("   ").is_err());
    assert_eq!(
        RuleId::new("source.event_shape")
            .expect("non-empty rule ID should be accepted")
            .as_str(),
        "source.event_shape"
    );
}
