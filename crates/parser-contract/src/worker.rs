//! RabbitMQ worker request and result message contract types.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    failure::{ErrorCode, ParseFailure, ParseStage, Retryability},
    presence::{FieldPresence, UnknownReason},
    source_ref::{SourceChecksum, SourceRef, SourceRefs},
    version::{ContractVersion, ParserInfo},
};

/// Parser-worker request message consumed from an AMQP broker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParseJobMessage {
    /// Durable parse-job identifier owned by `server-2`.
    pub job_id: String,
    /// Replay identifier owned by `server-2`.
    pub replay_id: String,
    /// S3-compatible raw replay object key.
    pub object_key: String,
    /// Expected raw replay object checksum.
    pub checksum: SourceChecksum,
    /// Parser artifact contract version requested by the job.
    pub parser_contract_version: ContractVersion,
}

/// S3-compatible object reference for a parser artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactReference {
    /// Artifact object bucket.
    pub bucket: String,
    /// Artifact object key.
    pub key: String,
}

/// Worker result routing kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ParseResultKind {
    /// Successful parse result with a durable artifact reference.
    #[serde(rename = "parse.completed")]
    Completed,
    /// Failed parse result with structured failure data.
    #[serde(rename = "parse.failed")]
    Failed,
}

/// Successful parser-worker result message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ParseCompletedMessage {
    /// Result message routing kind.
    pub message_type: ParseResultKind,
    /// Durable parse-job identifier owned by `server-2`.
    pub job_id: String,
    /// Replay identifier owned by `server-2`.
    pub replay_id: String,
    /// Parser artifact contract version used for the artifact.
    pub parser_contract_version: ContractVersion,
    /// Verified raw replay source checksum.
    pub source_checksum: SourceChecksum,
    /// Durable parser artifact object reference.
    pub artifact: ArtifactReference,
    /// SHA-256 checksum of the exact artifact bytes written to storage.
    pub artifact_checksum: SourceChecksum,
    /// Byte size of the exact artifact bytes written to storage.
    pub artifact_size_bytes: u64,
    /// Parser binary metadata.
    pub parser: ParserInfo,
}

#[derive(Deserialize)]
struct ParseCompletedMessageWire {
    message_type: ParseResultKind,
    job_id: String,
    replay_id: String,
    parser_contract_version: ContractVersion,
    source_checksum: SourceChecksum,
    artifact: ArtifactReference,
    artifact_checksum: SourceChecksum,
    artifact_size_bytes: u64,
    parser: ParserInfo,
}

impl<'de> Deserialize<'de> for ParseCompletedMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ParseCompletedMessageWire::deserialize(deserializer)?;
        if wire.message_type != ParseResultKind::Completed {
            return Err(serde::de::Error::custom(
                "completed result must use message_type parse.completed",
            ));
        }

        Ok(Self {
            message_type: wire.message_type,
            job_id: wire.job_id,
            replay_id: wire.replay_id,
            parser_contract_version: wire.parser_contract_version,
            source_checksum: wire.source_checksum,
            artifact: wire.artifact,
            artifact_checksum: wire.artifact_checksum,
            artifact_size_bytes: wire.artifact_size_bytes,
            parser: wire.parser,
        })
    }
}

impl ParseCompletedMessage {
    /// Builds a completed worker result and sets the routing kind.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the explicit worker result wire fields"
    )]
    pub const fn new(
        job_id: String,
        replay_id: String,
        parser_contract_version: ContractVersion,
        source_checksum: SourceChecksum,
        artifact: ArtifactReference,
        artifact_checksum: SourceChecksum,
        artifact_size_bytes: u64,
        parser: ParserInfo,
    ) -> Self {
        Self {
            message_type: ParseResultKind::Completed,
            job_id,
            replay_id,
            parser_contract_version,
            source_checksum,
            artifact,
            artifact_checksum,
            artifact_size_bytes,
            parser,
        }
    }
}

/// Failed parser-worker result message.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct ParseFailedMessage {
    /// Result message routing kind.
    pub message_type: ParseResultKind,
    /// Durable parse-job identifier, when it could be read from the job.
    pub job_id: FieldPresence<String>,
    /// Replay identifier, when it could be read from the job.
    pub replay_id: FieldPresence<String>,
    /// S3-compatible raw replay object key, when it could be read from the job.
    pub object_key: FieldPresence<String>,
    /// Parser artifact contract version, when it could be read from the job.
    pub parser_contract_version: FieldPresence<ContractVersion>,
    /// Expected raw replay source checksum, when it could be read from the job.
    pub source_checksum: FieldPresence<SourceChecksum>,
    /// Structured parse or worker failure.
    pub failure: ParseFailure,
    /// Parser binary metadata.
    pub parser: ParserInfo,
}

#[derive(Deserialize)]
struct ParseFailedMessageWire {
    message_type: ParseResultKind,
    job_id: FieldPresence<String>,
    replay_id: FieldPresence<String>,
    object_key: FieldPresence<String>,
    parser_contract_version: FieldPresence<ContractVersion>,
    source_checksum: FieldPresence<SourceChecksum>,
    failure: ParseFailure,
    parser: ParserInfo,
}

impl<'de> Deserialize<'de> for ParseFailedMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ParseFailedMessageWire::deserialize(deserializer)?;
        if wire.message_type != ParseResultKind::Failed {
            return Err(serde::de::Error::custom(
                "failed result must use message_type parse.failed",
            ));
        }

        Ok(Self {
            message_type: wire.message_type,
            job_id: wire.job_id,
            replay_id: wire.replay_id,
            object_key: wire.object_key,
            parser_contract_version: wire.parser_contract_version,
            source_checksum: wire.source_checksum,
            failure: wire.failure,
            parser: wire.parser,
        })
    }
}

impl ParseFailedMessage {
    /// Builds a failed worker result and sets the routing kind.
    #[must_use]
    pub const fn new(
        job_id: FieldPresence<String>,
        replay_id: FieldPresence<String>,
        object_key: FieldPresence<String>,
        parser_contract_version: FieldPresence<ContractVersion>,
        source_checksum: FieldPresence<SourceChecksum>,
        failure: ParseFailure,
        parser: ParserInfo,
    ) -> Self {
        Self {
            message_type: ParseResultKind::Failed,
            job_id,
            replay_id,
            object_key,
            parser_contract_version,
            source_checksum,
            failure,
            parser,
        }
    }

    /// Builds the non-retryable failure emitted for an unsupported contract version.
    pub fn unsupported_contract_version(
        job_id: FieldPresence<String>,
        replay_id: FieldPresence<String>,
        object_key: FieldPresence<String>,
        parser_contract_version: FieldPresence<ContractVersion>,
        source_checksum: FieldPresence<SourceChecksum>,
        parser: ParserInfo,
    ) -> Self {
        let failure = ParseFailure {
            job_id: job_id.clone(),
            replay_id: replay_id.clone(),
            source_file: object_key.clone(),
            checksum: source_checksum.clone(),
            stage: ParseStage::Schema,
            error_code: ErrorCode::unsupported_contract_version(),
            message: "unsupported parser contract version".to_owned(),
            retryability: Retryability::NotRetryable,
            source_cause: FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: None,
            },
            source_refs: schema_message_source_refs(),
        };

        Self::new(
            job_id,
            replay_id,
            object_key,
            parser_contract_version,
            source_checksum,
            failure,
            parser,
        )
    }
}

fn schema_message_source_refs() -> SourceRefs {
    SourceRefs::single(SourceRef {
        replay_id: None,
        source_file: None,
        checksum: None,
        frame: None,
        event_index: None,
        entity_id: None,
        json_path: Some("$".to_owned()),
        rule_id: None,
    })
}

/// Parser-worker result message envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ParseResultMessage {
    /// Successful parser-worker result.
    Completed(Box<ParseCompletedMessage>),
    /// Failed parser-worker result.
    Failed(Box<ParseFailedMessage>),
}
