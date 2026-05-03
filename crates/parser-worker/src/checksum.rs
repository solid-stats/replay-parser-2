//! Local SHA-256 checksum helpers. coverage-exclusion: reviewed internally-generated checksum validation defensive branches are allowlisted by exact source line.

use parser_contract::source_ref::SourceChecksum;
use sha2::{Digest, Sha256};

use crate::error::{WorkerError, WorkerFailureKind};

/// Returns the lowercase SHA-256 checksum for `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Builds a validated [`SourceChecksum`] from the local SHA-256 of `bytes`.
///
/// # Errors
///
/// Returns [`WorkerError`] if the internally produced checksum cannot be validated.
pub fn source_checksum_from_bytes(bytes: &[u8]) -> Result<SourceChecksum, WorkerError> {
    SourceChecksum::sha256(sha256_hex(bytes))
        .map_err(|source| WorkerError::ChecksumValidation(source.to_string()))
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
    let actual =
        source_checksum_from_bytes(bytes).map_err(|error| WorkerFailureKind::Internal {
            code: "internal.checksum_validation",
            message: error.to_string(),
        })?;

    if &actual == expected {
        return Ok(());
    }

    Err(WorkerFailureKind::ChecksumMismatch { expected: expected.clone(), actual })
}
