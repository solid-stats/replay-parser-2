//! Local SHA-256 checksum helpers.

use parser_contract::source_ref::SourceChecksum;
use sha2::{Digest, Sha256};

use crate::error::WorkerFailureKind;

/// Returns the lowercase SHA-256 checksum for `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Builds a validated [`SourceChecksum`] from the local SHA-256 of `bytes`.
#[must_use]
pub fn source_checksum_from_bytes(bytes: &[u8]) -> SourceChecksum {
    source_checksum_from_valid_sha256(sha256_hex(bytes))
}

/// Verifies that `bytes` match an expected source checksum.
///
/// # Errors
///
/// Returns [`WorkerFailureKind::ChecksumMismatch`] when the locally computed checksum does not
/// match `expected`.
pub fn verify_source_checksum(
    bytes: &[u8],
    expected: &SourceChecksum,
) -> Result<(), WorkerFailureKind> {
    let actual = source_checksum_from_valid_sha256(sha256_hex(bytes));

    if &actual == expected {
        return Ok(());
    }

    Err(WorkerFailureKind::ChecksumMismatch { expected: expected.clone(), actual })
}

#[allow(
    clippy::expect_used,
    reason = "sha256_hex always produces exactly 64 lowercase hexadecimal characters"
)]
fn source_checksum_from_valid_sha256(value: String) -> SourceChecksum {
    SourceChecksum::sha256(value).expect("locally computed SHA-256 should validate")
}
