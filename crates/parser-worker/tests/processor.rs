//! Job processor behavior tests.

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
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::{ContractVersion, ParserInfo},
    worker::{ParseCompletedMessage, ParseFailedMessage, ParseJobMessage},
};
use parser_core::{ParserInput, ParserOptions, public_parse_replay};
use parser_worker::{
    amqp::{DeliveryAction, PublishedOutcome},
    artifact_key::artifact_key,
    checksum::source_checksum_from_bytes,
    config::{WorkerConfig, WorkerConfigOverrides},
    error::{WorkerError, WorkerFailureKind},
    processor::{PublisherFuture, ResultPublisher, process_job_body},
    storage::{ArtifactPutOutcome, ObjectStore, ObjectStoreFuture},
};
use serde_json::json;

const VALID_REPLAY: &[u8] =
    include_bytes!("../../parser-core/tests/fixtures/valid-minimal.ocap.json");
const INVALID_REPLAY: &[u8] =
    include_bytes!("../../parser-core/tests/fixtures/invalid-json.ocap.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FakeArtifactWriteOutcome {
    Created,
    Reused,
    Conflicted,
}

#[derive(Debug, Default)]
struct FakeObjectStore {
    bucket: String,
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
    get_calls: Mutex<Vec<String>>,
    put_calls: Mutex<Vec<String>>,
    get_failures: Mutex<BTreeSet<String>>,
    put_failures: Mutex<BTreeSet<String>>,
    artifact_write_outcomes: Mutex<Vec<FakeArtifactWriteOutcome>>,
}

impl FakeObjectStore {
    fn new() -> Self {
        Self { bucket: "solid-replays".to_owned(), ..Default::default() }
    }

    fn insert_object(&self, key: &str, bytes: &[u8]) {
        let _previous = self
            .objects
            .lock()
            .expect("fake object map lock should not be poisoned")
            .insert(key.to_owned(), bytes.to_vec());
    }

    fn fail_put(&self, key: &str) {
        let _inserted = self
            .put_failures
            .lock()
            .expect("fake put failure set lock should not be poisoned")
            .insert(key.to_owned());
    }

    fn fail_get(&self, key: &str) {
        let _inserted = self
            .get_failures
            .lock()
            .expect("fake get failure set lock should not be poisoned")
            .insert(key.to_owned());
    }

    fn stored_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.objects.lock().expect("fake object map lock should not be poisoned").get(key).cloned()
    }

    fn get_call_count(&self) -> usize {
        self.get_calls.lock().expect("fake get calls lock should not be poisoned").len()
    }

    fn put_call_count(&self) -> usize {
        self.put_calls.lock().expect("fake put calls lock should not be poisoned").len()
    }

    fn artifact_write_outcomes(&self) -> Vec<FakeArtifactWriteOutcome> {
        self.artifact_write_outcomes
            .lock()
            .expect("fake artifact write outcomes lock should not be poisoned")
            .clone()
    }

    fn record_artifact_write_outcome(&self, outcome: FakeArtifactWriteOutcome) {
        self.artifact_write_outcomes
            .lock()
            .expect("fake artifact write outcomes lock should not be poisoned")
            .push(outcome);
    }
}

impl ObjectStore for FakeObjectStore {
    fn bucket(&self) -> &str {
        &self.bucket
    }

    fn get_object_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>> {
        Box::pin(async move {
            self.get_calls
                .lock()
                .expect("fake get calls lock should not be poisoned")
                .push(object_key.to_owned());
            if self
                .get_failures
                .lock()
                .expect("fake get failure set lock should not be poisoned")
                .contains(object_key)
            {
                return Err(WorkerError::S3 {
                    operation: "get_object",
                    bucket: self.bucket.clone(),
                    key: object_key.to_owned(),
                    stage: ParseStage::Input,
                    retryability: Retryability::Retryable,
                    message: "configured fake get failure".to_owned(),
                });
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
            self.put_calls
                .lock()
                .expect("fake put calls lock should not be poisoned")
                .push(object_key.to_owned());
            if self
                .put_failures
                .lock()
                .expect("fake put failure set lock should not be poisoned")
                .contains(object_key)
            {
                return Err(WorkerError::S3 {
                    operation: "put_object",
                    bucket: self.bucket.clone(),
                    key: object_key.to_owned(),
                    stage: ParseStage::Output,
                    retryability: Retryability::Retryable,
                    message: "configured fake put failure".to_owned(),
                });
            }

            self.insert_object(object_key, bytes);
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
            assert_eq!(content_type, "application/json");
            if self
                .put_failures
                .lock()
                .expect("fake put failure set lock should not be poisoned")
                .contains(object_key)
            {
                return Err(WorkerError::S3 {
                    operation: "put_object",
                    bucket: self.bucket.clone(),
                    key: object_key.to_owned(),
                    stage: ParseStage::Output,
                    retryability: Retryability::Retryable,
                    message: "configured fake put failure".to_owned(),
                });
            }

            let existing = self
                .objects
                .lock()
                .expect("fake object map lock should not be poisoned")
                .get(object_key)
                .cloned();

            match existing {
                Some(existing) if existing == bytes => {
                    self.record_artifact_write_outcome(FakeArtifactWriteOutcome::Reused);
                    Ok(ArtifactPutOutcome::AlreadyExists)
                }
                Some(_different) => {
                    self.record_artifact_write_outcome(FakeArtifactWriteOutcome::Conflicted);
                    Ok(ArtifactPutOutcome::AlreadyExists)
                }
                None => {
                    self.put_calls
                        .lock()
                        .expect("fake put calls lock should not be poisoned")
                        .push(object_key.to_owned());
                    self.insert_object(object_key, bytes);
                    self.record_artifact_write_outcome(FakeArtifactWriteOutcome::Created);
                    Ok(ArtifactPutOutcome::Created)
                }
            }
        })
    }
}

#[derive(Debug, Default)]
struct FakePublisher {
    completed: Mutex<Vec<ParseCompletedMessage>>,
    failed: Mutex<Vec<ParseFailedMessage>>,
    fail_publish: bool,
}

impl FakePublisher {
    fn failing() -> Self {
        Self { fail_publish: true, ..Default::default() }
    }

    fn completed_messages(&self) -> Vec<ParseCompletedMessage> {
        self.completed.lock().expect("completed messages lock should not be poisoned").clone()
    }

    fn failed_messages(&self) -> Vec<ParseFailedMessage> {
        self.failed.lock().expect("failed messages lock should not be poisoned").clone()
    }

    fn publish_error() -> WorkerError {
        WorkerError::Failure(WorkerFailureKind::RabbitMqPublish {
            message: "configured fake publish failure".to_owned(),
        })
    }
}

impl ResultPublisher for FakePublisher {
    fn publish_completed<'a>(&'a self, message: &'a ParseCompletedMessage) -> PublisherFuture<'a> {
        Box::pin(async move {
            if self.fail_publish {
                return Err(Self::publish_error());
            }
            self.completed
                .lock()
                .expect("completed messages lock should not be poisoned")
                .push(message.clone());
            Ok(PublishedOutcome::Completed)
        })
    }

    fn publish_failed<'a>(&'a self, message: &'a ParseFailedMessage) -> PublisherFuture<'a> {
        Box::pin(async move {
            if self.fail_publish {
                return Err(Self::publish_error());
            }
            self.failed
                .lock()
                .expect("failed messages lock should not be poisoned")
                .push(message.clone());
            Ok(PublishedOutcome::Failed)
        })
    }
}

fn config() -> WorkerConfig {
    WorkerConfig::from_env_and_overrides(
        |_| None,
        WorkerConfigOverrides { s3_bucket: Some("solid-replays".to_owned()), ..Default::default() },
    )
    .expect("test worker config should be valid")
}

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn checksum(bytes: &[u8]) -> SourceChecksum {
    source_checksum_from_bytes(bytes).expect("test checksum should be valid")
}

fn job_message(raw_bytes: &[u8]) -> ParseJobMessage {
    ParseJobMessage {
        job_id: "job-1".to_owned(),
        replay_id: "replay-1".to_owned(),
        object_key: "raw/replay-1.ocap.json".to_owned(),
        checksum: checksum(raw_bytes),
        parser_contract_version: ContractVersion::current(),
    }
}

fn job_body(job: &ParseJobMessage) -> Vec<u8> {
    serde_json::to_vec(job).expect("job should serialize")
}

fn public_artifact_bytes(job: &ParseJobMessage, raw_bytes: &[u8]) -> Vec<u8> {
    let artifact = public_parse_replay(ParserInput {
        bytes: raw_bytes,
        source: ReplaySource {
            replay_id: Some(job.replay_id.clone()),
            source_file: job.object_key.clone(),
            checksum: FieldPresence::Present { value: job.checksum.clone(), source: None },
        },
        parser: parser_info(),
        options: ParserOptions::default(),
    });
    let mut bytes = serde_json::to_vec(&artifact).expect("artifact should serialize");
    bytes.push(b'\n');
    bytes
}

async fn process(
    body: &[u8],
    store: &FakeObjectStore,
    publisher: &FakePublisher,
) -> DeliveryAction {
    process_job_body(body, &config(), store, publisher, parser_info())
        .await
        .expect("processor should return a delivery action")
}

fn failure_code(message: &ParseFailedMessage) -> &str {
    message.failure.error_code.as_str()
}

#[tokio::test]
async fn processor_valid_job_should_write_artifact_and_publish_completed() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let key =
        artifact_key(&config().artifact_prefix, &job.replay_id, &job.checksum).expect("key valid");
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, VALID_REPLAY);
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let completed = publisher.completed_messages();
    let stored = store.stored_bytes(&key).expect("artifact should be stored");
    let stored_checksum = checksum(&stored);

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.put_call_count(), 1);
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].artifact.bucket, "solid-replays");
    assert_eq!(completed[0].artifact.key, key);
    assert_eq!(completed[0].artifact_checksum, stored_checksum);
    assert_eq!(
        completed[0].artifact_size_bytes,
        u64::try_from(stored.len()).expect("stored bytes length should fit u64")
    );
    assert_eq!(completed[0].source_checksum, job.checksum);
    assert_eq!(store.artifact_write_outcomes(), vec![FakeArtifactWriteOutcome::Created]);
    assert!(publisher.failed_messages().is_empty());
}

#[tokio::test]
async fn processor_duplicate_redelivery_should_reuse_matching_artifact_and_publish_completed() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, VALID_REPLAY);
    let publisher = FakePublisher::default();

    // Act
    let first_action = process(&body, &store, &publisher).await;
    let second_action = process(&body, &store, &publisher).await;
    let completed = publisher.completed_messages();

    // Assert
    assert_eq!(first_action, DeliveryAction::Ack);
    assert_eq!(second_action, DeliveryAction::Ack);
    assert_eq!(
        store.artifact_write_outcomes(),
        vec![FakeArtifactWriteOutcome::Created, FakeArtifactWriteOutcome::Reused]
    );
    assert_eq!(completed.len(), 2);
    assert_eq!(completed[0].artifact.key, completed[1].artifact.key);
    assert_eq!(completed[0].artifact_checksum, completed[1].artifact_checksum);
    assert_eq!(completed[0].artifact_size_bytes, completed[1].artifact_size_bytes);
    assert!(publisher.failed_messages().is_empty());
}

#[tokio::test]
async fn processor_worker_artifact_bytes_should_match_cli_default_minified_bytes() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let key =
        artifact_key(&config().artifact_prefix, &job.replay_id, &job.checksum).expect("key valid");
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, VALID_REPLAY);
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let stored = store.stored_bytes(&key).expect("artifact should be stored");

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(stored, public_artifact_bytes(&job, VALID_REPLAY));
    assert!(stored.ends_with(b"\n"));
    assert_eq!(
        String::from_utf8(stored).expect("artifact should be UTF-8").trim_end().lines().count(),
        1
    );
}

#[tokio::test]
async fn processor_unsupported_contract_version_should_publish_failed_without_storage() {
    // Arrange
    let mut job = job_message(VALID_REPLAY);
    job.parser_contract_version =
        ContractVersion::parse("2.0.0").expect("unsupported version should parse");
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.get_call_count(), 0);
    assert_eq!(store.put_call_count(), 0);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "unsupported.contract_version");
    assert!(publisher.completed_messages().is_empty());
}

#[tokio::test]
async fn processor_malformed_job_json_should_publish_failed() {
    // Arrange
    let store = FakeObjectStore::new();
    let publisher = FakePublisher::default();

    // Act
    let action = process(b"{not-json", &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "json.decode");
    assert!(matches!(failed[0].job_id, FieldPresence::Unknown { .. }));
    assert_eq!(store.get_call_count(), 0);
}

#[tokio::test]
async fn processor_missing_job_field_should_publish_failed_with_unknown_presence() {
    // Arrange
    let body = br#"{"job_id":"job-1","replay_id":"replay-1"}"#;
    let store = FakeObjectStore::new();
    let publisher = FakePublisher::default();

    // Act
    let action = process(body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "schema.parse_job");
    assert!(matches!(failed[0].object_key, FieldPresence::Unknown { .. }));
    assert_eq!(store.get_call_count(), 0);
}

#[tokio::test]
async fn processor_empty_job_field_should_publish_failed_without_storage() {
    // Arrange
    let mut job = job_message(VALID_REPLAY);
    job.object_key.clear();
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.get_call_count(), 0);
    assert_eq!(store.put_call_count(), 0);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "schema.parse_job");
    assert_eq!(failed[0].failure.stage, ParseStage::Schema);
    assert_eq!(failed[0].failure.retryability, Retryability::NotRetryable);
}

#[tokio::test]
async fn processor_checksum_mismatch_should_publish_failed_without_artifact_write() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, b"different raw bytes");
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.put_call_count(), 0);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "checksum.mismatch");
    assert_eq!(failed[0].failure.stage, ParseStage::Checksum);
}

#[tokio::test]
async fn processor_s3_raw_get_failure_should_publish_input_read_failure() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    store.fail_get(&job.object_key);
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.get_call_count(), 1);
    assert_eq!(store.put_call_count(), 0);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "io.s3_read");
    assert_eq!(failed[0].failure.stage, ParseStage::Input);
    assert_eq!(failed[0].failure.retryability, Retryability::Retryable);
    assert!(publisher.completed_messages().is_empty());
}

#[tokio::test]
async fn processor_parser_failure_should_publish_failed_without_artifact_write() {
    // Arrange
    let job = job_message(INVALID_REPLAY);
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, INVALID_REPLAY);
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.put_call_count(), 0);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "json.decode");
    assert!(publisher.completed_messages().is_empty());
}

#[tokio::test]
async fn processor_artifact_conflict_should_publish_failed_and_ack() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let key =
        artifact_key(&config().artifact_prefix, &job.replay_id, &job.checksum).expect("key valid");
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, VALID_REPLAY);
    store.insert_object(&key, b"conflicting artifact bytes");
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.put_call_count(), 0);
    assert_eq!(store.artifact_write_outcomes(), vec![FakeArtifactWriteOutcome::Conflicted]);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "output.artifact_conflict");
    assert_eq!(failed[0].failure.stage, ParseStage::Output);
    assert!(publisher.completed_messages().is_empty());
}

#[tokio::test]
async fn processor_s3_artifact_write_failure_should_publish_failed() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let key =
        artifact_key(&config().artifact_prefix, &job.replay_id, &job.checksum).expect("key valid");
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, VALID_REPLAY);
    store.fail_put(&key);
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "output.s3_write");
    assert!(publisher.completed_messages().is_empty());
}

#[tokio::test]
async fn processor_invalid_artifact_key_should_publish_failed() {
    // Arrange
    let mut job = job_message(VALID_REPLAY);
    job.replay_id = ".".to_owned();
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, VALID_REPLAY);
    let publisher = FakePublisher::default();

    // Act
    let action = process(&body, &store, &publisher).await;
    let failed = publisher.failed_messages();

    // Assert
    assert_eq!(action, DeliveryAction::Ack);
    assert_eq!(store.put_call_count(), 0);
    assert_eq!(failed.len(), 1);
    assert_eq!(failure_code(&failed[0]), "output.artifact_key");
    assert!(publisher.completed_messages().is_empty());
}

#[tokio::test]
async fn processor_completed_publish_failure_should_return_nack_requeue() {
    // Arrange
    let job = job_message(VALID_REPLAY);
    let body = job_body(&job);
    let store = FakeObjectStore::new();
    store.insert_object(&job.object_key, VALID_REPLAY);
    let publisher = FakePublisher::failing();

    // Act
    let action = process(&body, &store, &publisher).await;

    // Assert
    assert_eq!(action, DeliveryAction::NackRequeue);
}

#[tokio::test]
async fn processor_failed_publish_failure_should_return_nack_requeue() {
    // Arrange
    let store = FakeObjectStore::new();
    let publisher = FakePublisher::failing();

    // Act
    let action = process(b"{not-json", &store, &publisher).await;

    // Assert
    assert_eq!(action, DeliveryAction::NackRequeue);
    assert_eq!(store.get_call_count(), 0);
}
