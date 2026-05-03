//! Deterministic S3 artifact key construction. coverage-exclusion: reviewed artifact-key postcondition defensive branches are allowlisted by exact source line.

use parser_contract::source_ref::SourceChecksum;

use crate::error::WorkerError;

/// Builds the deterministic parser artifact key for a replay/source checksum pair.
///
/// The key format is `{normalized_prefix}/{encoded_replay_id}/{source_sha256}.json`.
///
/// # Errors
///
/// Returns [`WorkerError`] when the prefix is empty or the replay ID cannot be represented as a
/// safe single path segment.
pub fn artifact_key(
    prefix: &str,
    replay_id: &str,
    source_checksum: &SourceChecksum,
) -> Result<String, WorkerError> {
    let prefix = normalize_prefix(prefix)?;
    let encoded_replay_id = encode_replay_id(replay_id)?;

    Ok(format!("{prefix}/{encoded_replay_id}/{}.json", source_checksum.value.as_str()))
}

fn normalize_prefix(prefix: &str) -> Result<&str, WorkerError> {
    let normalized = prefix.trim_matches('/');
    if normalized.is_empty() {
        return Err(WorkerError::ArtifactKey(
            "artifact prefix must not be empty after trimming slashes".to_owned(),
        ));
    }
    Ok(normalized)
}

fn encode_replay_id(replay_id: &str) -> Result<String, WorkerError> {
    if replay_id.is_empty() {
        return Err(WorkerError::ArtifactKey("replay_id must not be empty".to_owned()));
    }
    if matches!(replay_id, "." | "..") {
        return Err(WorkerError::ArtifactKey(
            "replay_id must not be a standalone dot segment".to_owned(),
        ));
    }

    let mut encoded = String::with_capacity(replay_id.len());
    for byte in replay_id.bytes() {
        if is_allowed_replay_id_byte(byte) {
            encoded.push(char::from(byte));
        } else {
            percent_encode_byte(&mut encoded, byte);
        }
    }

    if matches!(encoded.as_str(), "." | "..")
        || encoded.contains('/')
        || encoded.contains('\\')
        || encoded.split('/').any(|segment| matches!(segment, "." | ".."))
    {
        return Err(WorkerError::ArtifactKey(
            "encoded replay_id must be a safe single path segment".to_owned(),
        ));
    }

    Ok(encoded)
}

const fn is_allowed_replay_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
}

fn percent_encode_byte(output: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    output.push('%');
    output.push(char::from(HEX[usize::from(byte >> 4)]));
    output.push(char::from(HEX[usize::from(byte & 0x0F)]));
}
