//! Object storage behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};

use parser_contract::{
    failure::{ParseStage, Retryability},
    source_ref::SourceChecksum,
};
use parser_worker::{
    checksum::verify_source_checksum,
    error::{WorkerError, WorkerFailureKind},
    storage::{ArtifactWrite, DownloadedObject, ObjectStore, ObjectStoreFuture},
};

const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[derive(Debug, Default)]
struct FakeObjectStore {
    bucket: String,
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
    get_failures: Mutex<BTreeSet<String>>,
    put_failures: Mutex<BTreeSet<String>>,
}

impl FakeObjectStore {
    fn new() -> Self {
        Self {
            bucket: "solid-replays".to_owned(),
            objects: Mutex::new(BTreeMap::new()),
            get_failures: Mutex::new(BTreeSet::new()),
            put_failures: Mutex::new(BTreeSet::new()),
        }
    }

    fn with_object(key: &str, bytes: &[u8]) -> Self {
        let store = Self::new();
        store.insert_object(key, bytes);
        store
    }

    fn insert_object(&self, key: &str, bytes: &[u8]) {
        let _previous = self
            .objects
            .lock()
            .expect("fake object map lock should not be poisoned")
            .insert(key.to_owned(), bytes.to_vec());
    }

    fn stored_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.objects.lock().expect("fake object map lock should not be poisoned").get(key).cloned()
    }

    fn fail_get(&self, key: &str) {
        let _inserted = self
            .get_failures
            .lock()
            .expect("fake get failure set lock should not be poisoned")
            .insert(key.to_owned());
    }

    fn fail_put(&self, key: &str) {
        let _inserted = self
            .put_failures
            .lock()
            .expect("fake put failure set lock should not be poisoned")
            .insert(key.to_owned());
    }
}

impl ObjectStore for FakeObjectStore {
    fn bucket(&self) -> &str {
        &self.bucket
    }

    fn get_object_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>> {
        Box::pin(async move {
            if self
                .get_failures
                .lock()
                .expect("fake get failure set lock should not be poisoned")
                .contains(object_key)
            {
                return Err(storage_error("get_object", &self.bucket, object_key));
            }

            self.objects
                .lock()
                .expect("fake object map lock should not be poisoned")
                .get(object_key)
                .cloned()
                .ok_or_else(|| WorkerError::ObjectNotFound {
                    operation: "get_object",
                    bucket: self.bucket.clone(),
                    key: object_key.to_owned(),
                    stage: ParseStage::Input,
                    retryability: Retryability::Unknown,
                })
        })
    }

    fn put_object_bytes<'a>(
        &'a self,
        object_key: &'a str,
        bytes: &'a [u8],
        content_type: &'a str,
    ) -> ObjectStoreFuture<'a, ()> {
        Box::pin(async move {
            assert_eq!(content_type, "application/json");
            if self
                .put_failures
                .lock()
                .expect("fake put failure set lock should not be poisoned")
                .contains(object_key)
            {
                return Err(storage_error("put_object", &self.bucket, object_key));
            }

            self.insert_object(object_key, bytes);
            Ok(())
        })
    }
}

fn storage_error(operation: &'static str, bucket: &str, key: &str) -> WorkerError {
    WorkerError::S3 {
        operation,
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        stage: ParseStage::Output,
        retryability: Retryability::Retryable,
        message: "configured fake storage failure".to_owned(),
    }
}

fn checksum(value: &str) -> SourceChecksum {
    SourceChecksum::sha256(value).expect("test checksum should be valid SHA-256")
}

fn assert_artifact_write(
    write: &ArtifactWrite,
    key: &str,
    checksum: &str,
    size_bytes: u64,
    reused_existing: bool,
) {
    assert_eq!(write.reference.bucket, "solid-replays");
    assert_eq!(write.reference.key, key);
    assert_eq!(write.checksum.value.as_str(), checksum);
    assert_eq!(write.size_bytes, size_bytes);
    assert_eq!(write.reused_existing, reused_existing);
}

#[tokio::test]
async fn storage_download_raw_should_return_bytes_and_local_sha256() {
    // Arrange
    let store = FakeObjectStore::with_object("raw/replay.json", b"abc");

    // Act
    let downloaded: DownloadedObject =
        store.download_raw("raw/replay.json").await.expect("raw object should download");

    // Assert
    assert_eq!(downloaded.bytes, b"abc");
    assert_eq!(downloaded.checksum.value.as_str(), ABC_SHA256);
}

#[tokio::test]
async fn storage_downloaded_raw_should_be_verifiable_against_expected_checksum() {
    // Arrange
    let store = FakeObjectStore::with_object("raw/replay.json", b"different");
    let downloaded =
        store.download_raw("raw/replay.json").await.expect("raw object should download");

    // Act
    let error =
        verify_source_checksum(&downloaded.bytes, &checksum(ABC_SHA256)).expect_err("mismatch");

    // Assert
    assert_eq!(error.error_code(), "checksum.mismatch");
}

#[tokio::test]
async fn storage_write_artifact_should_put_new_object_with_checksum_and_size() {
    // Arrange
    let store = FakeObjectStore::new();
    let key = "artifacts/v3/replay-1/source.json";
    let bytes = br#"{"ok":true}"#;

    // Act
    let write = store
        .write_artifact_if_absent_or_matching(key, bytes)
        .await
        .expect("new artifact should write");

    // Assert
    assert_artifact_write(
        &write,
        key,
        "4062edaf750fb8074e7e83e0c9028c94e32468a8b6f1614774328ef045150f93",
        11,
        false,
    );
    assert_eq!(store.stored_bytes(key).as_deref(), Some(bytes.as_slice()));
}

#[tokio::test]
async fn storage_write_artifact_should_reuse_matching_existing_object() {
    // Arrange
    let key = "artifacts/v3/replay-1/source.json";
    let bytes = br#"{"ok":true}"#;
    let store = FakeObjectStore::with_object(key, bytes);

    // Act
    let write = store
        .write_artifact_if_absent_or_matching(key, bytes)
        .await
        .expect("matching artifact should be reused");

    // Assert
    assert_artifact_write(
        &write,
        key,
        "4062edaf750fb8074e7e83e0c9028c94e32468a8b6f1614774328ef045150f93",
        11,
        true,
    );
}

#[tokio::test]
async fn storage_write_artifact_should_return_conflict_for_different_existing_object() {
    // Arrange
    let key = "artifacts/v3/replay-1/source.json";
    let store = FakeObjectStore::with_object(key, br#"{"ok":false}"#);

    // Act
    let error = store
        .write_artifact_if_absent_or_matching(key, br#"{"ok":true}"#)
        .await
        .expect_err("different existing artifact should conflict");

    // Assert
    match error {
        WorkerError::Failure(WorkerFailureKind::ArtifactConflict { .. }) => {}
        other => assert!(
            other.to_string().contains("output.artifact_conflict"),
            "unexpected error: {other}"
        ),
    }
}

#[tokio::test]
async fn storage_download_raw_should_return_unknown_input_failure_when_object_is_missing() {
    // Arrange
    let store = FakeObjectStore::new();

    // Act
    let error =
        store.download_raw("raw/missing.json").await.expect_err("missing object should fail");

    // Assert
    match error {
        WorkerError::ObjectNotFound { key, stage, retryability, .. } => {
            assert_eq!(key, "raw/missing.json");
            assert_eq!(stage, ParseStage::Input);
            assert_eq!(retryability, Retryability::Unknown);
        }
        other => {
            assert!(other.to_string().contains("S3 object not found"), "unexpected error: {other}")
        }
    }
}

#[tokio::test]
async fn storage_download_raw_should_return_retryable_get_failure_without_panicking() {
    // Arrange
    let store = FakeObjectStore::new();
    store.fail_get("raw/replay.json");

    // Act
    let error =
        store.download_raw("raw/replay.json").await.expect_err("configured get should fail");

    // Assert
    match error {
        WorkerError::S3 { operation, retryability, .. } => {
            assert_eq!(operation, "get_object");
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => {
            assert!(other.to_string().contains("S3 operation failed"), "unexpected error: {other}")
        }
    }
}

#[tokio::test]
async fn storage_write_artifact_should_return_retryable_put_failure_without_panicking() {
    // Arrange
    let store = FakeObjectStore::new();
    let key = "artifacts/v3/replay-1/source.json";
    store.fail_put(key);

    // Act
    let error = store
        .write_artifact_if_absent_or_matching(key, b"{}")
        .await
        .expect_err("configured put should fail");

    // Assert
    match error {
        WorkerError::S3 { operation, retryability, .. } => {
            assert_eq!(operation, "put_object");
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => {
            assert!(other.to_string().contains("S3 operation failed"), "unexpected error: {other}")
        }
    }
}
