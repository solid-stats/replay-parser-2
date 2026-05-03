//! S3-compatible object storage boundary.

use std::{fmt, pin::Pin};

use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Region},
    error::{ProvideErrorMetadata, SdkError},
    operation::{get_object::GetObjectError, put_object::PutObjectError},
    primitives::ByteStream,
};
use parser_contract::{
    failure::{ParseStage, Retryability},
    source_ref::SourceChecksum,
    worker::ArtifactReference,
};

use crate::{
    checksum::source_checksum_from_bytes,
    config::WorkerConfig,
    error::{WorkerError, WorkerFailureKind},
};

/// Boxed future returned by object-store operations.
pub type ObjectStoreFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, WorkerError>> + Send + 'a>>;

/// Raw object bytes plus locally computed checksum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedObject {
    /// Downloaded object bytes.
    pub bytes: Vec<u8>,
    /// Local SHA-256 checksum of the downloaded bytes.
    pub checksum: SourceChecksum,
}

/// Result of writing or reusing a deterministic parser artifact object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactWrite {
    /// Durable artifact object reference.
    pub reference: ArtifactReference,
    /// Local SHA-256 checksum of the exact artifact bytes.
    pub checksum: SourceChecksum,
    /// Exact artifact byte size.
    pub size_bytes: u64,
    /// Whether an existing matching object was reused.
    pub reused_existing: bool,
}

impl ArtifactWrite {
    /// Returns the worker log event name for this artifact decision:
    /// `worker_artifact_written` or `worker_artifact_reused`.
    #[must_use]
    pub const fn log_event_name(&self) -> &'static str {
        if self.reused_existing {
            crate::logging::WORKER_ARTIFACT_REUSED
        } else {
            crate::logging::WORKER_ARTIFACT_WRITTEN
        }
    }
}

/// Outcome of attempting an atomic artifact create.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactPutOutcome {
    /// The artifact object was created by this write.
    Created,
    /// The artifact object already existed before this write could create it.
    AlreadyExists,
    /// The provider rejected conditional create semantics.
    UnsupportedConditionalWrite,
}

/// Minimal object-store interface used by worker processing.
pub trait ObjectStore: Sync {
    /// Returns the configured artifact/raw object bucket.
    fn bucket(&self) -> &str;

    /// Gets object bytes by key.
    fn get_object_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>>;

    /// Gets artifact object bytes by key.
    fn get_artifact_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>> {
        self.get_object_bytes(object_key)
    }

    /// Writes object bytes with the supplied content type.
    fn put_object_bytes<'a>(
        &'a self,
        object_key: &'a str,
        bytes: &'a [u8],
        content_type: &'a str,
    ) -> ObjectStoreFuture<'a, ()>;

    /// Attempts to create artifact bytes only when the key is absent.
    fn put_artifact_bytes_if_absent<'a>(
        &'a self,
        object_key: &'a str,
        bytes: &'a [u8],
        content_type: &'a str,
    ) -> ObjectStoreFuture<'a, ArtifactPutOutcome> {
        Box::pin(async move {
            self.put_object_bytes(object_key, bytes, content_type).await?;
            Ok(ArtifactPutOutcome::Created)
        })
    }

    /// Downloads a raw replay object and computes its checksum locally.
    fn download_raw<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, DownloadedObject> {
        Box::pin(async move {
            let bytes = self.get_object_bytes(object_key).await?;
            let checksum = source_checksum_from_bytes(&bytes)?;
            Ok(DownloadedObject { bytes, checksum })
        })
    }

    /// Writes a parser artifact unless an existing deterministic object already matches.
    fn write_artifact_if_absent_or_matching<'a>(
        &'a self,
        key: &'a str,
        bytes: &'a [u8],
    ) -> ObjectStoreFuture<'a, ArtifactWrite> {
        Box::pin(async move {
            let new_checksum = source_checksum_from_bytes(bytes)?;
            let new_size_bytes = byte_len(bytes)?;
            let bucket = self.bucket().to_owned();

            match self.put_artifact_bytes_if_absent(key, bytes, "application/json").await? {
                ArtifactPutOutcome::Created => {
                    Ok(artifact_write(bucket, key.to_owned(), new_checksum, new_size_bytes, false))
                }
                ArtifactPutOutcome::AlreadyExists => {
                    let existing_bytes = self.get_artifact_bytes(key).await?;
                    compare_existing_artifact(
                        bucket,
                        key,
                        &existing_bytes,
                        &new_checksum,
                        new_size_bytes,
                    )
                }
                ArtifactPutOutcome::UnsupportedConditionalWrite => {
                    match self.get_artifact_bytes(key).await {
                        Ok(existing_bytes) => compare_existing_artifact(
                            bucket,
                            key,
                            &existing_bytes,
                            &new_checksum,
                            new_size_bytes,
                        ),
                        Err(WorkerError::ObjectNotFound { .. }) => {
                            self.put_object_bytes(key, bytes, "application/json").await?;
                            Ok(artifact_write(
                                bucket,
                                key.to_owned(),
                                new_checksum,
                                new_size_bytes,
                                false,
                            ))
                        }
                        Err(error) => Err(error),
                    }
                }
            }
        })
    }
}

/// S3-compatible object store backed by `aws-sdk-s3`.
#[derive(Clone)]
pub struct S3ObjectStore {
    client: Client,
    bucket: String,
}

impl S3ObjectStore {
    /// Builds an S3 object store from worker configuration.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] when the worker configuration is invalid.
    pub async fn from_config(config: &WorkerConfig) -> Result<Self, WorkerError> {
        config.validate()?;

        let mut loader = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.s3_region.clone()));
        if let Some(endpoint_url) = config.s3_endpoint.as_deref() {
            loader = loader.endpoint_url(endpoint_url);
        }

        let shared_config = loader.load().await;
        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .behavior_version(BehaviorVersion::latest())
            .force_path_style(config.s3_force_path_style)
            .build();

        Ok(Self::new(Client::from_conf(s3_config), config.s3_bucket.clone()))
    }

    /// Creates an object store from an already configured S3 client.
    #[must_use]
    pub fn new(client: Client, bucket: impl Into<String>) -> Self {
        Self { client, bucket: bucket.into() }
    }

    /// Checks whether the configured bucket is reachable.
    #[must_use]
    pub fn check_ready(&self) -> ObjectStoreFuture<'_, ()> {
        Box::pin(async move {
            let _head_bucket_output =
                self.client.head_bucket().bucket(&self.bucket).send().await.map_err(|error| {
                    s3_error(
                        "head_bucket",
                        &self.bucket,
                        "",
                        ParseStage::Input,
                        Retryability::Retryable,
                        error,
                    )
                })?;
            Ok(())
        })
    }
}

impl fmt::Debug for S3ObjectStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("S3ObjectStore")
            .field("bucket", &self.bucket)
            .finish_non_exhaustive()
    }
}

impl ObjectStore for S3ObjectStore {
    fn bucket(&self) -> &str {
        &self.bucket
    }

    fn get_object_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>> {
        self.get_s3_object_bytes(object_key, ParseStage::Input)
    }

    fn get_artifact_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>> {
        self.get_s3_object_bytes(object_key, ParseStage::Output)
    }

    fn put_object_bytes<'a>(
        &'a self,
        object_key: &'a str,
        bytes: &'a [u8],
        content_type: &'a str,
    ) -> ObjectStoreFuture<'a, ()> {
        Box::pin(async move {
            let _put_output = self
                .client
                .put_object()
                .bucket(&self.bucket)
                .key(object_key)
                .content_type(content_type)
                .body(ByteStream::from(bytes.to_vec()))
                .send()
                .await
                .map_err(|error| {
                    s3_error(
                        "put_object",
                        &self.bucket,
                        object_key,
                        ParseStage::Output,
                        Retryability::Retryable,
                        error,
                    )
                })?;
            Ok(())
        })
    }

    fn put_artifact_bytes_if_absent<'a>(
        &'a self,
        object_key: &'a str,
        bytes: &'a [u8],
        content_type: &'a str,
    ) -> ObjectStoreFuture<'a, ArtifactPutOutcome> {
        Box::pin(async move {
            let result = self
                .client
                .put_object()
                .bucket(&self.bucket)
                .key(object_key)
                .content_type(content_type)
                .if_none_match("*")
                .body(ByteStream::from(bytes.to_vec()))
                .send()
                .await;

            match result {
                Ok(_put_output) => Ok(ArtifactPutOutcome::Created),
                Err(error) => classify_conditional_put_error(&error).map_or_else(
                    || {
                        Err(s3_error(
                            "put_object",
                            &self.bucket,
                            object_key,
                            ParseStage::Output,
                            Retryability::Retryable,
                            error,
                        ))
                    },
                    Ok,
                ),
            }
        })
    }
}

impl S3ObjectStore {
    fn get_s3_object_bytes<'a>(
        &'a self,
        object_key: &'a str,
        stage: ParseStage,
    ) -> ObjectStoreFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let response = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(object_key)
                .send()
                .await
                .map_err(|error| {
                    if is_no_such_key(&error) {
                        return object_not_found("get_object", &self.bucket, object_key, stage);
                    }
                    s3_error(
                        "get_object",
                        &self.bucket,
                        object_key,
                        stage,
                        Retryability::Retryable,
                        error,
                    )
                })?;

            let bytes = response.body.collect().await.map_err(|error| {
                s3_error(
                    "get_object_body",
                    &self.bucket,
                    object_key,
                    stage,
                    Retryability::Retryable,
                    error,
                )
            })?;

            Ok(bytes.into_bytes().to_vec())
        })
    }
}

const fn artifact_write(
    bucket: String,
    key: String,
    checksum: SourceChecksum,
    size_bytes: u64,
    reused_existing: bool,
) -> ArtifactWrite {
    ArtifactWrite {
        reference: ArtifactReference { bucket, key },
        checksum,
        size_bytes,
        reused_existing,
    }
}

fn compare_existing_artifact(
    bucket: String,
    key: &str,
    existing_bytes: &[u8],
    new_checksum: &SourceChecksum,
    new_size_bytes: u64,
) -> Result<ArtifactWrite, WorkerError> {
    let existing_checksum = source_checksum_from_bytes(existing_bytes)?;
    let existing_size_bytes = byte_len(existing_bytes)?;
    if existing_size_bytes == new_size_bytes && existing_checksum == *new_checksum {
        return Ok(artifact_write(
            bucket,
            key.to_owned(),
            new_checksum.clone(),
            new_size_bytes,
            true,
        ));
    }

    Err(WorkerFailureKind::ArtifactConflict {
        key: key.to_owned(),
        existing_checksum,
        existing_size_bytes,
        new_checksum: new_checksum.clone(),
        new_size_bytes,
    }
    .into())
}

fn byte_len(bytes: &[u8]) -> Result<u64, WorkerError> {
    u64::try_from(bytes.len()).map_err(|source| {
        WorkerError::ChecksumValidation(format!("object byte length does not fit u64: {source}"))
    })
}

fn is_no_such_key(error: &SdkError<GetObjectError>) -> bool {
    error.as_service_error().is_some_and(GetObjectError::is_no_such_key)
}

fn classify_conditional_put_error(error: &SdkError<PutObjectError>) -> Option<ArtifactPutOutcome> {
    let service_error = error.as_service_error();
    let code = service_error.and_then(ProvideErrorMetadata::code);
    let message = service_error.and_then(ProvideErrorMetadata::message);
    let status = error.raw_response().map(|response| response.status().as_u16());

    if matches!(
        code,
        Some("PreconditionFailed" | "PreconditionFailedException" | "ConditionalRequestConflict")
    ) || matches!(status, Some(409 | 412))
    {
        return Some(ArtifactPutOutcome::AlreadyExists);
    }

    if matches!(code, Some("NotImplemented" | "NotSupported")) || matches!(status, Some(501)) {
        return Some(ArtifactPutOutcome::UnsupportedConditionalWrite);
    }

    if (matches!(code, Some("InvalidRequest")) || matches!(status, Some(400)))
        && message_mentions_if_none_match(message)
    {
        return Some(ArtifactPutOutcome::UnsupportedConditionalWrite);
    }

    None
}

fn message_mentions_if_none_match(message: Option<&str>) -> bool {
    message.is_some_and(|message| {
        let normalized = message.to_ascii_lowercase();
        normalized.contains("if-none-match") || normalized.contains("if none match")
    })
}

fn object_not_found(
    operation: &'static str,
    bucket: &str,
    key: &str,
    stage: ParseStage,
) -> WorkerError {
    WorkerError::ObjectNotFound {
        operation,
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        stage,
        retryability: Retryability::Unknown,
    }
}

fn s3_error(
    operation: &'static str,
    bucket: &str,
    key: &str,
    stage: ParseStage,
    retryability: Retryability,
    source: impl std::error::Error,
) -> WorkerError {
    WorkerError::S3 {
        operation,
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        stage,
        retryability,
        message: source.to_string(),
    }
}
