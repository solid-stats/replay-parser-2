//! Artifact key and checksum behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::source_ref::SourceChecksum;
use parser_worker::{
    artifact_key::artifact_key,
    checksum::{sha256_hex, source_checksum_from_bytes, verify_source_checksum},
    error::WorkerFailureKind,
};

const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const ARTIFACT_SHA256: &str = "2f6dfdd7781ccb5fa30c7a122c8b0a52ad1a2f7a387865311356c0e335e1d3e5";

fn checksum(value: &str) -> SourceChecksum {
    SourceChecksum::sha256(value).expect("test checksum should be valid SHA-256")
}

#[test]
fn artifact_key_should_use_normalized_prefix_replay_segment_and_source_checksum() {
    // Arrange
    let source_checksum = checksum(ABC_SHA256);

    // Act
    let key = artifact_key("/artifacts/v3/", "replay-0001", &source_checksum)
        .expect("artifact key should be built");

    // Assert
    assert_eq!(
        key,
        "artifacts/v3/replay-0001/ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad.json"
    );
}

#[test]
fn artifact_key_should_use_source_checksum_not_artifact_checksum() {
    // Arrange
    let source_checksum = checksum(ABC_SHA256);

    // Act
    let key = artifact_key("artifacts/v3", "replay-0001", &source_checksum)
        .expect("artifact key should be built");

    // Assert
    assert!(key.contains(ABC_SHA256));
    assert!(!key.contains(ARTIFACT_SHA256));
}

#[test]
fn artifact_key_should_percent_encode_path_separators_spaces_and_non_ascii() {
    // Arrange
    let source_checksum = checksum(ABC_SHA256);

    // Act
    let slash_key = artifact_key("artifacts/v3", "replay/id", &source_checksum)
        .expect("slash should be encoded");
    let traversal_key = artifact_key("artifacts/v3", "../replay", &source_checksum)
        .expect("traversal-like value should be encoded");
    let space_key = artifact_key("artifacts/v3", "replay id", &source_checksum)
        .expect("space should be encoded");
    let unicode_key = artifact_key("artifacts/v3", "реплей", &source_checksum)
        .expect("unicode should be encoded");

    // Assert
    assert!(slash_key.contains("replay%2Fid"));
    assert!(traversal_key.contains("..%2Freplay"));
    assert!(space_key.contains("replay%20id"));
    assert!(unicode_key.contains("%D1%80%D0%B5%D0%BF%D0%BB%D0%B5%D0%B9"));
    assert!(!slash_key.contains("/replay/id/"));
    assert!(!traversal_key.contains("/../"));
}

#[test]
fn artifact_key_should_reject_empty_and_dot_segment_replay_ids() {
    // Arrange
    let source_checksum = checksum(ABC_SHA256);

    // Act
    let empty = artifact_key("artifacts/v3", "", &source_checksum);
    let dot = artifact_key("artifacts/v3", ".", &source_checksum);
    let dot_dot = artifact_key("artifacts/v3", "..", &source_checksum);

    // Assert
    assert!(empty.is_err());
    assert!(dot.is_err());
    assert!(dot_dot.is_err());
}

#[test]
fn artifact_key_should_reject_prefix_that_is_empty_after_trimming() {
    // Arrange
    let source_checksum = checksum(ABC_SHA256);

    // Act
    let error = artifact_key("///", "replay-0001", &source_checksum)
        .expect_err("empty normalized prefix should fail");

    // Assert
    assert!(error.to_string().contains("artifact prefix"));
}

#[test]
fn artifact_key_should_keep_safe_dot_inside_replay_id_segment() {
    // Arrange
    let source_checksum = checksum(ABC_SHA256);

    // Act
    let key = artifact_key("artifacts/v3", "replay.0001", &source_checksum)
        .expect("safe dot inside segment should be accepted");

    // Assert
    assert!(key.contains("/replay.0001/"));
}

#[test]
fn sha256_hex_should_return_known_sha256_for_abc() {
    // Act
    let checksum = sha256_hex(b"abc");

    // Assert
    assert_eq!(checksum, ABC_SHA256);
}

#[test]
fn source_checksum_from_bytes_should_return_sha256_source_checksum() {
    // Act
    let checksum =
        source_checksum_from_bytes(b"abc").expect("locally generated checksum should validate");

    // Assert
    assert_eq!(checksum.value.as_str(), ABC_SHA256);
}

#[test]
fn verify_source_checksum_should_accept_matching_checksum() {
    // Arrange
    let expected = checksum(ABC_SHA256);

    // Act
    let result = verify_source_checksum(b"abc", &expected);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn verify_source_checksum_should_return_checksum_mismatch_failure() {
    // Arrange
    let expected = checksum(ABC_SHA256);

    // Act
    let error =
        verify_source_checksum(b"different", &expected).expect_err("checksum should mismatch");

    // Assert
    assert!(matches!(error, WorkerFailureKind::ChecksumMismatch { .. }));
    assert_eq!(error.error_code(), "checksum.mismatch");
    assert_eq!(error.stage(), parser_contract::failure::ParseStage::Checksum);
    assert_eq!(error.retryability(), parser_contract::failure::Retryability::NotRetryable);
}

#[test]
fn worker_failure_kind_should_classify_output_and_internal_failures() {
    let publish = WorkerFailureKind::RabbitMqPublish { message: "nack".to_owned() };
    let internal =
        WorkerFailureKind::Internal { code: "internal.example", message: "example".to_owned() };

    assert_eq!(publish.error_code(), "output.rabbitmq_publish");
    assert_eq!(publish.stage(), parser_contract::failure::ParseStage::Output);
    assert_eq!(publish.retryability(), parser_contract::failure::Retryability::Unknown);
    assert_eq!(internal.error_code(), "internal.example");
    assert_eq!(internal.stage(), parser_contract::failure::ParseStage::Internal);
    assert_eq!(internal.retryability(), parser_contract::failure::Retryability::Unknown);
}
