//! End-to-end worker job processor.
// coverage-exclusion: reviewed v1.0 processor transport/error fallback regions are allowlisted by exact source line.

use std::{pin::Pin, time::Instant};

use parser_contract::{
    artifact::ParseStatus,
    failure::{ErrorCode, ParseFailure, ParseStage, Retryability},
    presence::{FieldPresence, UnknownReason},
    source_ref::{ReplaySource, SourceChecksum, SourceRef, SourceRefs},
    version::{ContractVersion, ParserInfo},
    worker::{ParseCompletedMessage, ParseFailedMessage, ParseJobMessage},
};
use parser_core::{ParserInput, ParserOptions, public_parse_replay};

use crate::{
    amqp::{DeliveryAction, PublishedOutcome, RabbitMqClient, delivery_action_after_publish},
    artifact_key::artifact_key,
    checksum::verify_source_checksum,
    config::WorkerConfig,
    error::{WorkerError, WorkerFailureKind},
    logging::{
        OUTCOME_COMPLETED, OUTCOME_FAILED, WORKER_ARTIFACT_CONFLICT, WORKER_JOB_COMPLETED,
        WORKER_JOB_FAILED, WORKER_PARSE_FINISHED, WORKER_PARSE_STARTED, duration_ms,
    },
    storage::{ArtifactWrite, ObjectStore},
};

/// Boxed future returned by result-publisher operations.
pub type PublisherFuture<'a> =
    Pin<Box<dyn Future<Output = Result<PublishedOutcome, WorkerError>> + Send + 'a>>;

/// Minimal result publisher boundary used by the processor and no-network tests.
pub trait ResultPublisher: Sync {
    /// Publishes a confirmed `parse.completed` message.
    fn publish_completed<'a>(&'a self, message: &'a ParseCompletedMessage) -> PublisherFuture<'a>;

    /// Publishes a confirmed `parse.failed` message.
    fn publish_failed<'a>(&'a self, message: &'a ParseFailedMessage) -> PublisherFuture<'a>;
}

impl ResultPublisher for RabbitMqClient {
    fn publish_completed<'a>(&'a self, message: &'a ParseCompletedMessage) -> PublisherFuture<'a> {
        Box::pin(async move {
            Self::publish_completed(self, message).await?;
            Ok(PublishedOutcome::Completed)
        })
    }

    fn publish_failed<'a>(&'a self, message: &'a ParseFailedMessage) -> PublisherFuture<'a> {
        Box::pin(async move {
            Self::publish_failed(self, message).await?;
            Ok(PublishedOutcome::Failed)
        })
    }
}

/// Processes one `RabbitMQ` job body through storage, parser-core, artifact write, and result publish.
///
/// # Errors
///
/// Returns [`WorkerError`] only when the worker cannot construct a handled outcome. Confirmed
/// result publication failures are represented as [`DeliveryAction::NackRequeue`].
pub async fn process_job_body(
    body: &[u8],
    config: &WorkerConfig,
    store: &impl ObjectStore,
    publisher: &impl ResultPublisher,
    parser_info: ParserInfo,
) -> Result<DeliveryAction, WorkerError> {
    match serde_json::from_slice::<ParseJobMessage>(body) {
        Ok(job) => process_decoded_job(job, config, store, publisher, parser_info).await,
        Err(error) => {
            let failed = malformed_job_failed_message(body, &error, parser_info)?;
            publish_failed_action(config, publisher, &failed).await
        }
    }
}

async fn process_decoded_job(
    job: ParseJobMessage,
    config: &WorkerConfig,
    store: &impl ObjectStore,
    publisher: &impl ResultPublisher,
    parser_info: ParserInfo,
) -> Result<DeliveryAction, WorkerError> {
    let context = JobContext::from_job(&job);

    if let Some(source_cause) = validate_job_fields(&job) {
        let failure = build_parse_failure(
            &context,
            ParseStage::Schema,
            "schema.parse_job",
            "parse job JSON did not match the worker contract",
            Retryability::NotRetryable,
            source_cause,
        )?;
        let failed = context.failed_message(failure, parser_info);
        return publish_failed_action(config, publisher, &failed).await;
    }

    if job.parser_contract_version != ContractVersion::current() {
        let failed = ParseFailedMessage::unsupported_contract_version(
            context.job_id.clone(),
            context.replay_id.clone(),
            context.object_key.clone(),
            context.parser_contract_version.clone(),
            context.source_checksum.clone(),
            parser_info,
        )
        .map_err(|source| internal_error("internal.failure_payload", source.to_string()))?;
        return publish_failed_action(config, publisher, &failed).await;
    }

    let downloaded = match store.download_raw(&job.object_key).await {
        Ok(downloaded) => downloaded,
        Err(error) => {
            let failed = storage_failed_message(&context, error, parser_info)?;
            return publish_failed_action(config, publisher, &failed).await;
        }
    };

    if let Err(kind) = verify_source_checksum(&downloaded.bytes, &job.checksum) {
        let failed = worker_failure_message(&context, &kind, parser_info)?;
        return publish_failed_action(config, publisher, &failed).await;
    }

    let source = ReplaySource {
        replay_id: Some(job.replay_id.clone()),
        source_file: job.object_key.clone(),
        checksum: FieldPresence::Present { value: job.checksum.clone(), source: None },
    };
    log_parse_started(config, &job);
    let parse_start = Instant::now();
    let artifact = public_parse_replay(ParserInput {
        bytes: &downloaded.bytes,
        source,
        parser: parser_info.clone(),
        options: ParserOptions::default(),
    });
    log_parse_finished(config, &job, parse_start);

    if artifact.status == ParseStatus::Failed {
        let failed = parser_failed_message(&context, artifact.failure, parser_info)?;
        return publish_failed_action(config, publisher, &failed).await;
    }

    let mut artifact_bytes = serde_json::to_vec(&artifact)?;
    artifact_bytes.push(b'\n');
    let key = match artifact_key(&config.artifact_prefix, &job.replay_id, &job.checksum) {
        Ok(key) => key,
        Err(error) => {
            let failed = storage_failed_message(&context, error, parser_info)?;
            return publish_failed_action(config, publisher, &failed).await;
        }
    };

    let artifact_write_start = Instant::now();
    let write = match store.write_artifact_if_absent_or_matching(&key, &artifact_bytes).await {
        Ok(write) => {
            log_artifact_write(config, &job, &write, artifact_write_start);
            write
        }
        Err(error) => {
            log_artifact_conflict(config, &job, &error, artifact_write_start);
            let failed = storage_failed_message(&context, error, parser_info)?;
            return publish_failed_action(config, publisher, &failed).await;
        }
    };

    let completed = ParseCompletedMessage::new(
        job.job_id,
        job.replay_id,
        ContractVersion::current(),
        downloaded.checksum,
        write.reference,
        write.checksum,
        write.size_bytes,
        parser_info,
    );
    publish_completed_action(config, publisher, &completed).await
}

fn log_parse_started(config: &WorkerConfig, job: &ParseJobMessage) {
    tracing::info!(
        event = WORKER_PARSE_STARTED,
        worker_id = %config.worker_id,
        job_id = %job.job_id,
        replay_id = %job.replay_id,
        stage = "parse",
        "worker_parse_started"
    );
}

fn log_parse_finished(config: &WorkerConfig, job: &ParseJobMessage, start: Instant) {
    tracing::info!(
        event = WORKER_PARSE_FINISHED,
        worker_id = %config.worker_id,
        job_id = %job.job_id,
        replay_id = %job.replay_id,
        stage = "parse",
        duration_ms = duration_ms(start),
        "worker_parse_finished"
    );
}

fn log_artifact_write(
    config: &WorkerConfig,
    job: &ParseJobMessage,
    write: &ArtifactWrite,
    start: Instant,
) {
    tracing::info!(
        event = write.log_event_name(),
        worker_id = %config.worker_id,
        job_id = %job.job_id,
        replay_id = %job.replay_id,
        artifact_key = %write.reference.key,
        artifact_size_bytes = write.size_bytes,
        duration_ms = duration_ms(start),
        "worker_artifact_decision"
    );
}

fn log_artifact_conflict(
    config: &WorkerConfig,
    job: &ParseJobMessage,
    error: &WorkerError,
    start: Instant,
) {
    let WorkerError::Failure(WorkerFailureKind::ArtifactConflict {
        key,
        existing_size_bytes,
        new_size_bytes,
        ..
    }) = error
    else {
        return;
    };

    tracing::warn!(
        event = WORKER_ARTIFACT_CONFLICT,
        worker_id = %config.worker_id,
        job_id = %job.job_id,
        replay_id = %job.replay_id,
        artifact_key = %key,
        artifact_size_bytes = *new_size_bytes,
        existing_artifact_size_bytes = *existing_size_bytes,
        error_type = "output.artifact_conflict",
        duration_ms = duration_ms(start),
        "worker_artifact_conflict"
    );
}

fn validate_job_fields(job: &ParseJobMessage) -> Option<String> {
    for (field_name, value) in [
        ("job_id", job.job_id.as_str()),
        ("replay_id", job.replay_id.as_str()),
        ("object_key", job.object_key.as_str()),
    ] {
        if value.trim().is_empty() {
            return Some(format!("parse job field {field_name} must not be empty"));
        }
    }
    None
}

async fn publish_completed_action(
    config: &WorkerConfig,
    publisher: &impl ResultPublisher,
    message: &ParseCompletedMessage,
) -> Result<DeliveryAction, WorkerError> {
    let publish_result = publisher.publish_completed(message).await;
    match &publish_result {
        Ok(PublishedOutcome::Completed) => tracing::info!(
            event = WORKER_JOB_COMPLETED,
            worker_id = %config.worker_id,
            job_id = %message.job_id,
            replay_id = %message.replay_id,
            artifact_key = %message.artifact.key,
            outcome = OUTCOME_COMPLETED,
            "worker_job_completed"
        ),
        Ok(PublishedOutcome::Failed) => tracing::warn!(
            event = WORKER_JOB_FAILED,
            worker_id = %config.worker_id,
            job_id = %message.job_id,
            replay_id = %message.replay_id,
            artifact_key = %message.artifact.key,
            error_code = "internal.unexpected_publish_outcome",
            error_type = "internal.unexpected_publish_outcome",
            retryability = ?Retryability::Unknown,
            outcome = OUTCOME_FAILED,
            "worker_job_failed"
        ),
        Err(error) => tracing::warn!(
            event = WORKER_JOB_FAILED,
            worker_id = %config.worker_id,
            job_id = %message.job_id,
            replay_id = %message.replay_id,
            artifact_key = %message.artifact.key,
            error_code = "output.rabbitmq_publish",
            error_type = "output.rabbitmq_publish",
            retryability = ?Retryability::Unknown,
            outcome = OUTCOME_FAILED,
            error = %error,
            "worker_job_failed"
        ),
    }
    Ok(delivery_action_after_publish(publish_result))
}

async fn publish_failed_action(
    config: &WorkerConfig,
    publisher: &impl ResultPublisher,
    message: &ParseFailedMessage,
) -> Result<DeliveryAction, WorkerError> {
    let publish_result = publisher.publish_failed(message).await;
    let job_id = field_presence_value(&message.job_id);
    let replay_id = field_presence_value(&message.replay_id);
    let object_key = field_presence_value(&message.object_key);
    match &publish_result {
        Ok(PublishedOutcome::Failed) => tracing::info!(
            event = WORKER_JOB_FAILED,
            worker_id = %config.worker_id,
            job_id = ?job_id,
            replay_id = ?replay_id,
            object_key = ?object_key,
            error_code = %message.failure.error_code.as_str(),
            error_type = %message.failure.error_code.as_str(),
            retryability = ?message.failure.retryability,
            outcome = OUTCOME_FAILED,
            "worker_job_failed"
        ),
        Ok(PublishedOutcome::Completed) => tracing::warn!(
            event = WORKER_JOB_FAILED,
            worker_id = %config.worker_id,
            job_id = ?job_id,
            replay_id = ?replay_id,
            object_key = ?object_key,
            error_code = "internal.unexpected_publish_outcome",
            error_type = "internal.unexpected_publish_outcome",
            retryability = ?Retryability::Unknown,
            outcome = OUTCOME_FAILED,
            "worker_job_failed"
        ),
        Err(error) => tracing::warn!(
            event = WORKER_JOB_FAILED,
            worker_id = %config.worker_id,
            job_id = ?job_id,
            replay_id = ?replay_id,
            object_key = ?object_key,
            error_code = "output.rabbitmq_publish",
            error_type = "output.rabbitmq_publish",
            retryability = ?Retryability::Unknown,
            outcome = OUTCOME_FAILED,
            error = %error,
            "worker_job_failed"
        ),
    }
    Ok(delivery_action_after_publish(publish_result))
}

const fn field_presence_value(presence: &FieldPresence<String>) -> Option<&str> {
    match presence {
        FieldPresence::Present { value, .. } | FieldPresence::Inferred { value, .. } => {
            Some(value.as_str())
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn malformed_job_failed_message(
    body: &[u8],
    error: &serde_json::Error,
    parser: ParserInfo,
) -> Result<ParseFailedMessage, WorkerError> {
    let json = serde_json::from_slice::<serde_json::Value>(body).ok();
    let context = JobContext::from_value(json.as_ref());
    let (stage, code, message) = if json.is_some() {
        (ParseStage::Schema, "schema.parse_job", "parse job JSON did not match the worker contract")
    } else {
        (ParseStage::JsonDecode, "json.decode", "parse job JSON could not be decoded")
    };
    let failure = build_parse_failure(
        &context,
        stage,
        code,
        message,
        Retryability::NotRetryable,
        error.to_string(),
    )?;

    Ok(context.failed_message(failure, parser))
}

fn parser_failed_message(
    context: &JobContext,
    failure: Option<ParseFailure>,
    parser: ParserInfo,
) -> Result<ParseFailedMessage, WorkerError> {
    let failure = failure.map_or_else(
        || {
            build_parse_failure(
                context,
                ParseStage::Internal,
                "internal.missing_failure_payload",
                "parser returned failed status without a structured failure payload",
                Retryability::Unknown,
                "missing parser failure payload".to_owned(),
            )
        },
        |failure| Ok(enrich_parser_failure(context, failure)),
    )?;

    Ok(context.failed_message(failure, parser))
}

fn worker_failure_message(
    context: &JobContext,
    kind: &WorkerFailureKind,
    parser: ParserInfo,
) -> Result<ParseFailedMessage, WorkerError> {
    let failure = build_parse_failure(
        context,
        kind.stage(),
        kind.error_code(),
        &kind.to_string(),
        kind.retryability(),
        kind.to_string(),
    )?;
    Ok(context.failed_message(failure, parser))
}

fn storage_failed_message(
    context: &JobContext,
    error: WorkerError,
    parser: ParserInfo,
) -> Result<ParseFailedMessage, WorkerError> {
    match error {
        WorkerError::Failure(kind) => worker_failure_message(context, &kind, parser),
        WorkerError::ObjectNotFound { stage, retryability, key, .. } => {
            let code = storage_error_code(stage);
            let failure = build_parse_failure(
                context,
                stage,
                code,
                "S3 object was not found",
                retryability,
                format!("S3 object not found: {key}"),
            )?;
            Ok(context.failed_message(failure, parser))
        }
        WorkerError::S3 { stage, retryability, operation, key, message, .. } => {
            let code = storage_error_code(stage);
            let failure = build_parse_failure(
                context,
                stage,
                code,
                "S3 operation failed",
                retryability,
                format!("{operation} failed for {key}: {message}"),
            )?;
            Ok(context.failed_message(failure, parser))
        }
        WorkerError::ArtifactKey(message) => {
            let failure = build_parse_failure(
                context,
                ParseStage::Output,
                "output.artifact_key",
                "artifact key construction failed",
                Retryability::Unknown,
                message,
            )?;
            Ok(context.failed_message(failure, parser))
        }
        WorkerError::ChecksumValidation(message) => {
            let failure = build_parse_failure(
                context,
                ParseStage::Internal,
                "internal.checksum_validation",
                "checksum validation failed",
                Retryability::Unknown,
                message,
            )?;
            Ok(context.failed_message(failure, parser))
        }
        other => Err(other),
    }
}

const fn storage_error_code(stage: ParseStage) -> &'static str {
    match stage {
        ParseStage::Input => "io.s3_read",
        ParseStage::Output => "output.s3_write",
        ParseStage::Checksum
        | ParseStage::JsonDecode
        | ParseStage::Schema
        | ParseStage::Normalize
        | ParseStage::Aggregate
        | ParseStage::Internal => "internal.storage_stage",
    }
}

fn enrich_parser_failure(context: &JobContext, mut failure: ParseFailure) -> ParseFailure {
    failure.job_id = context.job_id.clone();
    failure.replay_id = context.replay_id.clone();
    failure.source_file = context.object_key.clone();
    failure.checksum = context.source_checksum.clone();
    failure
}

fn build_parse_failure(
    context: &JobContext,
    stage: ParseStage,
    error_code: &str,
    message: &str,
    retryability: Retryability,
    source_cause: String,
) -> Result<ParseFailure, WorkerError> {
    Ok(ParseFailure {
        job_id: context.job_id.clone(),
        replay_id: context.replay_id.clone(),
        source_file: context.object_key.clone(),
        checksum: context.source_checksum.clone(),
        stage,
        error_code: ErrorCode::new(error_code)
            .map_err(|source| internal_error("internal.failure_payload", source.to_string()))?,
        message: message.to_owned(),
        retryability,
        source_cause: FieldPresence::Present { value: source_cause, source: None },
        source_refs: SourceRefs::new(vec![SourceRef {
            replay_id: context.replay_id_value.clone(),
            source_file: context.object_key_value.clone(),
            checksum: context.source_checksum_value.clone(),
            frame: None,
            event_index: None,
            entity_id: None,
            json_path: Some("$".to_owned()),
            rule_id: None,
        }])
        .map_err(|source| internal_error("internal.failure_payload", source.to_string()))?,
    })
}

#[derive(Debug, Clone)]
struct JobContext {
    job_id: FieldPresence<String>,
    replay_id: FieldPresence<String>,
    object_key: FieldPresence<String>,
    parser_contract_version: FieldPresence<ContractVersion>,
    source_checksum: FieldPresence<SourceChecksum>,
    replay_id_value: Option<String>,
    object_key_value: Option<String>,
    source_checksum_value: Option<SourceChecksum>,
}

impl JobContext {
    fn from_job(job: &ParseJobMessage) -> Self {
        Self {
            job_id: present(job.job_id.clone()),
            replay_id: present(job.replay_id.clone()),
            object_key: present(job.object_key.clone()),
            parser_contract_version: present(job.parser_contract_version.clone()),
            source_checksum: present(job.checksum.clone()),
            replay_id_value: Some(job.replay_id.clone()),
            object_key_value: Some(job.object_key.clone()),
            source_checksum_value: Some(job.checksum.clone()),
        }
    }

    fn from_value(value: Option<&serde_json::Value>) -> Self {
        Self {
            job_id: string_presence(value, "job_id"),
            replay_id: string_presence(value, "replay_id"),
            object_key: string_presence(value, "object_key"),
            parser_contract_version: contract_version_presence(value),
            source_checksum: checksum_presence(value),
            replay_id_value: string_value(value, "replay_id"),
            object_key_value: string_value(value, "object_key"),
            source_checksum_value: value
                .and_then(|root| root.get("checksum"))
                .and_then(|checksum| serde_json::from_value(checksum.clone()).ok()),
        }
    }

    fn failed_message(&self, failure: ParseFailure, parser: ParserInfo) -> ParseFailedMessage {
        ParseFailedMessage::new(
            self.job_id.clone(),
            self.replay_id.clone(),
            self.object_key.clone(),
            self.parser_contract_version.clone(),
            self.source_checksum.clone(),
            failure,
            parser,
        )
    }
}

const fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present { value, source: None }
}

fn string_presence(value: Option<&serde_json::Value>, key: &str) -> FieldPresence<String> {
    match value.and_then(|root| root.get(key)) {
        Some(serde_json::Value::String(value)) => present(value.clone()),
        Some(_other) => unknown(UnknownReason::SchemaDrift),
        None => unknown(UnknownReason::SourceFieldAbsent),
    }
}

fn string_value(value: Option<&serde_json::Value>, key: &str) -> Option<String> {
    value.and_then(|root| root.get(key)).and_then(serde_json::Value::as_str).map(str::to_owned)
}

const fn unknown<T>(reason: UnknownReason) -> FieldPresence<T> {
    FieldPresence::Unknown { reason, source: None }
}

fn contract_version_presence(value: Option<&serde_json::Value>) -> FieldPresence<ContractVersion> {
    value.and_then(|root| root.get("parser_contract_version")).map_or_else(
        || unknown(UnknownReason::SourceFieldAbsent),
        |field| {
            serde_json::from_value::<ContractVersion>(field.clone())
                .map_or_else(|_error| unknown(UnknownReason::SchemaDrift), present)
        },
    )
}

fn checksum_presence(value: Option<&serde_json::Value>) -> FieldPresence<SourceChecksum> {
    value.and_then(|root| root.get("checksum")).map_or_else(
        || unknown(UnknownReason::SourceFieldAbsent),
        |field| {
            serde_json::from_value::<SourceChecksum>(field.clone())
                .map_or_else(|_error| unknown(UnknownReason::SchemaDrift), present)
        },
    )
}

const fn internal_error(code: &'static str, message: String) -> WorkerError {
    WorkerError::Failure(WorkerFailureKind::Internal { code, message })
}
