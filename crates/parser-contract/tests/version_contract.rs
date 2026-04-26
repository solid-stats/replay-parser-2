//! Parser and contract version contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::version::{
    CURRENT_CONTRACT_VERSION, ContractVersion, ParserBuildInfo, ParserInfo,
};
use schemars::JsonSchema;
use semver::Version;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize, JsonSchema)]
struct VersionArtifactStub {
    contract_version: ContractVersion,
    parser: ParserInfo,
}

#[test]
fn version_contract_contract_version_should_serialize_as_semver_string_when_current_version_is_used()
 {
    // Arrange
    let artifact = VersionArtifactStub {
        contract_version: ContractVersion::current(),
        parser: parser_info("0.1.0", None),
    };

    // Act
    let serialized_contract_version =
        serde_json::to_value(&artifact.contract_version).expect("contract version serializes");
    let serialized_artifact = serde_json::to_value(&artifact).expect("artifact stub serializes");

    // Assert
    assert_eq!(serialized_contract_version, json!(CURRENT_CONTRACT_VERSION));
    assert_eq!(serialized_artifact["contract_version"], json!(CURRENT_CONTRACT_VERSION));
}

#[test]
fn version_contract_parser_info_should_keep_parser_version_separate_from_contract_version() {
    // Arrange
    let artifact = VersionArtifactStub {
        contract_version: ContractVersion::current(),
        parser: parser_info(
            "0.1.0",
            Some(ParserBuildInfo {
                git_commit: Some("abc1234".to_owned()),
                target: None,
                profile: None,
            }),
        ),
    };

    // Act
    let serialized_artifact = serde_json::to_value(&artifact).expect("artifact stub serializes");

    // Assert
    assert_eq!(
        serialized_artifact,
        json!({
            "contract_version": "1.0.0",
            "parser": {
                "name": "replay-parser-2",
                "version": "0.1.0",
                "build": {
                    "git_commit": "abc1234"
                }
            }
        })
    );
    assert_ne!(serialized_artifact["contract_version"], serialized_artifact["parser"]["version"]);
    assert_eq!(serialized_artifact["parser"]["build"]["git_commit"], json!("abc1234"));
}

fn parser_info(version: &str, build: Option<ParserBuildInfo>) -> ParserInfo {
    ParserInfo {
        name: "replay-parser-2".to_owned(),
        version: Version::parse(version).expect("test parser version is valid semver"),
        build,
    }
}
