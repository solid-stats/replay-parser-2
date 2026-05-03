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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use std::{
        collections::BTreeMap,
        sync::{
            Mutex,
            atomic::{AtomicBool, Ordering},
        },
    };

    use super::*;

    const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[derive(Debug)]
    struct UnitObjectStore {
        bucket: String,
        objects: Mutex<BTreeMap<String, Vec<u8>>>,
        conditional_outcome: Mutex<Option<ArtifactPutOutcome>>,
        fail_get_with_s3: AtomicBool,
        fail_put_with_s3: AtomicBool,
    }

    impl UnitObjectStore {
        fn new() -> Self {
            Self {
                bucket: "solid-replays".to_owned(),
                objects: Mutex::new(BTreeMap::new()),
                conditional_outcome: Mutex::new(None),
                fail_get_with_s3: AtomicBool::new(false),
                fail_put_with_s3: AtomicBool::new(false),
            }
        }

        fn with_object(key: &str, bytes: &[u8]) -> Self {
            let store = Self::new();
            store.insert(key, bytes);
            store
        }

        fn insert(&self, key: &str, bytes: &[u8]) {
            let _previous = self
                .objects
                .lock()
                .expect("unit object map lock should not be poisoned")
                .insert(key.to_owned(), bytes.to_vec());
        }

        fn stored(&self, key: &str) -> Option<Vec<u8>> {
            self.objects
                .lock()
                .expect("unit object map lock should not be poisoned")
                .get(key)
                .cloned()
        }

        fn set_conditional_outcome(&self, outcome: ArtifactPutOutcome) {
            let _previous = self
                .conditional_outcome
                .lock()
                .expect("unit outcome lock should not be poisoned")
                .replace(outcome);
        }

        fn fail_get_with_s3(&self) {
            self.fail_get_with_s3.store(true, Ordering::SeqCst);
        }

        fn fail_put_with_s3(&self) {
            self.fail_put_with_s3.store(true, Ordering::SeqCst);
        }
    }

    impl ObjectStore for UnitObjectStore {
        fn bucket(&self) -> &str {
            &self.bucket
        }

        fn get_object_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>> {
            Box::pin(async move {
                if self.fail_get_with_s3.load(Ordering::SeqCst) {
                    return Err(s3_error(
                        "get_object",
                        &self.bucket,
                        object_key,
                        ParseStage::Input,
                        Retryability::Retryable,
                        std::io::Error::new(std::io::ErrorKind::Other, "configured get failure"),
                    ));
                }

                self.objects
                    .lock()
                    .expect("unit object map lock should not be poisoned")
                    .get(object_key)
                    .cloned()
                    .ok_or_else(|| {
                        object_not_found("get_object", &self.bucket, object_key, ParseStage::Input)
                    })
            })
        }

        fn put_object_bytes<'a>(
            &'a self,
            object_key: &'a str,
            bytes: &'a [u8],
            _content_type: &'a str,
        ) -> ObjectStoreFuture<'a, ()> {
            Box::pin(async move {
                if self.fail_put_with_s3.load(Ordering::SeqCst) {
                    return Err(s3_error(
                        "put_object",
                        &self.bucket,
                        object_key,
                        ParseStage::Output,
                        Retryability::Retryable,
                        std::io::Error::new(std::io::ErrorKind::Other, "configured put failure"),
                    ));
                }

                self.insert(object_key, bytes);
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
                if let Some(outcome) = self
                    .conditional_outcome
                    .lock()
                    .expect("unit outcome lock should not be poisoned")
                    .as_ref()
                    .copied()
                {
                    return Ok(outcome);
                }

                ObjectStore::put_object_bytes(self, object_key, bytes, content_type).await?;
                Ok(ArtifactPutOutcome::Created)
            })
        }
    }

    #[tokio::test]
    async fn default_object_store_helpers_should_download_put_and_write_created_artifacts() {
        let store = UnitObjectStore::with_object("raw/replay.json", b"abc");

        let artifact_bytes =
            store.get_artifact_bytes("raw/replay.json").await.expect("artifact bytes should read");
        let downloaded =
            store.download_raw("raw/replay.json").await.expect("raw bytes should download");
        let put_outcome = ObjectStore::put_artifact_bytes_if_absent(
            &store,
            "artifacts/direct.json",
            br#"{"ok":true}"#,
            "application/json",
        )
        .await
        .expect("default put should create");
        let write = store
            .write_artifact_if_absent_or_matching("artifacts/write.json", b"[]")
            .await
            .expect("artifact should write");

        assert_eq!(artifact_bytes, b"abc");
        assert_eq!(downloaded.bytes, b"abc");
        assert_eq!(downloaded.checksum.value.as_str(), ABC_SHA256);
        assert_eq!(put_outcome, ArtifactPutOutcome::Created);
        assert_eq!(
            store.stored("artifacts/direct.json").as_deref(),
            Some(br#"{"ok":true}"#.as_slice())
        );
        assert_eq!(store.stored("artifacts/write.json").as_deref(), Some(b"[]".as_slice()));
        assert_eq!(write.reference.bucket, "solid-replays");
        assert_eq!(write.reference.key, "artifacts/write.json");
        assert_eq!(write.size_bytes, 2);
        assert!(!write.reused_existing);
    }

    #[tokio::test]
    async fn artifact_write_helpers_should_cover_existing_fallback_conflict_and_error_paths() {
        let key = "artifacts/existing.json";
        let store = UnitObjectStore::with_object(key, b"same");
        store.set_conditional_outcome(ArtifactPutOutcome::AlreadyExists);

        let reused = store
            .write_artifact_if_absent_or_matching(key, b"same")
            .await
            .expect("same bytes reuse");
        let conflict = store
            .write_artifact_if_absent_or_matching(key, b"different")
            .await
            .expect_err("different bytes should conflict");

        let fallback_key = "artifacts/fallback.json";
        let fallback = UnitObjectStore::new();
        fallback.set_conditional_outcome(ArtifactPutOutcome::UnsupportedConditionalWrite);
        let fallback_write = fallback
            .write_artifact_if_absent_or_matching(fallback_key, b"new")
            .await
            .expect("unsupported conditional write should fall back to put");

        let failing = UnitObjectStore::new();
        failing.set_conditional_outcome(ArtifactPutOutcome::UnsupportedConditionalWrite);
        failing.fail_get_with_s3();
        let fallback_error = failing
            .write_artifact_if_absent_or_matching("artifacts/error.json", b"new")
            .await
            .expect_err("fallback get error should be returned");

        let put_failing = UnitObjectStore::new();
        put_failing.fail_put_with_s3();
        let put_error = put_failing
            .write_artifact_if_absent_or_matching("artifacts/put-error.json", b"new")
            .await
            .expect_err("created write put error should be returned");

        assert!(reused.reused_existing);
        assert_eq!(fallback.stored(fallback_key).as_deref(), Some(b"new".as_slice()));
        assert_eq!(fallback_write.reference.key, fallback_key);
        assert!(matches!(
            conflict,
            WorkerError::Failure(WorkerFailureKind::ArtifactConflict { .. })
        ));
        assert!(matches!(fallback_error, WorkerError::S3 { operation: "get_object", .. }));
        assert!(matches!(put_error, WorkerError::S3 { operation: "put_object", .. }));
    }

    #[test]
    fn storage_private_helpers_should_shape_worker_errors_and_match_condition_messages() {
        let checksum = SourceChecksum::sha256(ABC_SHA256).expect("checksum should parse");
        let write = artifact_write(
            "bucket".to_owned(),
            "artifact.json".to_owned(),
            checksum.clone(),
            3,
            true,
        );
        let same =
            compare_existing_artifact("bucket".to_owned(), "artifact.json", b"abc", &checksum, 3)
                .expect("matching artifact should compare");
        let conflict = compare_existing_artifact(
            "bucket".to_owned(),
            "artifact.json",
            b"different",
            &checksum,
            3,
        )
        .expect_err("different artifact should conflict");
        let missing = object_not_found("get_object", "bucket", "missing.json", ParseStage::Input);
        let storage = s3_error(
            "put_object",
            "bucket",
            "artifact.json",
            ParseStage::Output,
            Retryability::Retryable,
            std::io::Error::new(std::io::ErrorKind::Other, "boom"),
        );
        let len = byte_len(b"abc").expect("byte length should fit u64");

        assert_eq!(write.reference.key, "artifact.json");
        assert!(write.reused_existing);
        assert_eq!(same.size_bytes, 3);
        assert_eq!(len, 3);
        assert!(matches!(
            conflict,
            WorkerError::Failure(WorkerFailureKind::ArtifactConflict { .. })
        ));
        assert!(matches!(
            missing,
            WorkerError::ObjectNotFound {
                operation: "get_object",
                key,
                stage: ParseStage::Input,
                retryability: Retryability::Unknown,
                ..
            } if key == "missing.json"
        ));
        assert!(matches!(
            storage,
            WorkerError::S3 {
                operation: "put_object",
                stage: ParseStage::Output,
                retryability: Retryability::Retryable,
                ..
            }
        ));
        assert!(message_mentions_if_none_match(Some("provider rejects If-None-Match")));
        assert!(message_mentions_if_none_match(Some("provider rejects if none match")));
        assert!(!message_mentions_if_none_match(Some("unrelated")));
        assert!(!message_mentions_if_none_match(None));
    }
}
