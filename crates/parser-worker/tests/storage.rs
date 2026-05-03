//! Object storage behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Credentials, Region},
};
use parser_contract::{
    failure::{ParseStage, Retryability},
    source_ref::SourceChecksum,
    worker::ArtifactReference,
};
use parser_worker::{
    checksum::verify_source_checksum,
    error::{WorkerError, WorkerFailureKind},
    logging::{WORKER_ARTIFACT_REUSED, WORKER_ARTIFACT_WRITTEN},
    storage::{
        ArtifactPutOutcome, ArtifactWrite, DownloadedObject, ObjectStore, ObjectStoreFuture,
        S3ObjectStore,
    },
};

const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[derive(Debug, Default)]
struct FakeObjectStore {
    bucket: String,
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
    get_failures: Mutex<BTreeSet<String>>,
    put_failures: Mutex<BTreeSet<String>>,
    conditional_put_outcomes: Mutex<BTreeMap<String, ArtifactPutOutcome>>,
    conditional_put_attempts: Mutex<Vec<String>>,
}

impl FakeObjectStore {
    fn new() -> Self {
        Self {
            bucket: "solid-replays".to_owned(),
            objects: Mutex::new(BTreeMap::new()),
            get_failures: Mutex::new(BTreeSet::new()),
            put_failures: Mutex::new(BTreeSet::new()),
            conditional_put_outcomes: Mutex::new(BTreeMap::new()),
            conditional_put_attempts: Mutex::new(Vec::new()),
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

    fn set_conditional_put_outcome(&self, key: &str, outcome: ArtifactPutOutcome) {
        let _previous = self
            .conditional_put_outcomes
            .lock()
            .expect("fake conditional put outcome map lock should not be poisoned")
            .insert(key.to_owned(), outcome);
    }

    fn conditional_put_attempt_count(&self, key: &str) -> usize {
        self.conditional_put_attempts
            .lock()
            .expect("fake conditional put attempt list lock should not be poisoned")
            .iter()
            .filter(|attempted_key| attempted_key.as_str() == key)
            .count()
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
                return Err(storage_error(
                    "get_object",
                    &self.bucket,
                    object_key,
                    ParseStage::Input,
                ));
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
                return Err(storage_error(
                    "put_object",
                    &self.bucket,
                    object_key,
                    ParseStage::Output,
                ));
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
            self.conditional_put_attempts
                .lock()
                .expect("fake conditional put attempt list lock should not be poisoned")
                .push(object_key.to_owned());

            if self
                .put_failures
                .lock()
                .expect("fake put failure set lock should not be poisoned")
                .contains(object_key)
            {
                return Err(storage_error(
                    "put_object",
                    &self.bucket,
                    object_key,
                    ParseStage::Output,
                ));
            }

            let configured_outcome = self
                .conditional_put_outcomes
                .lock()
                .expect("fake conditional put outcome map lock should not be poisoned")
                .get(object_key)
                .copied();

            match configured_outcome {
                Some(ArtifactPutOutcome::Created) => {
                    self.insert_object(object_key, bytes);
                    Ok(ArtifactPutOutcome::Created)
                }
                Some(ArtifactPutOutcome::AlreadyExists) => Ok(ArtifactPutOutcome::AlreadyExists),
                Some(ArtifactPutOutcome::UnsupportedConditionalWrite) => {
                    Ok(ArtifactPutOutcome::UnsupportedConditionalWrite)
                }
                None => {
                    let object_exists = self
                        .objects
                        .lock()
                        .expect("fake object map lock should not be poisoned")
                        .contains_key(object_key);
                    if object_exists {
                        Ok(ArtifactPutOutcome::AlreadyExists)
                    } else {
                        self.insert_object(object_key, bytes);
                        Ok(ArtifactPutOutcome::Created)
                    }
                }
            }
        })
    }
}

#[derive(Debug, Default)]
struct DirectPutObjectStore {
    bucket: String,
    objects: Mutex<BTreeMap<String, Vec<u8>>>,
    put_content_types: Mutex<Vec<String>>,
}

impl DirectPutObjectStore {
    fn new() -> Self {
        Self { bucket: "solid-replays".to_owned(), ..Default::default() }
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
            .expect("direct object map lock should not be poisoned")
            .insert(key.to_owned(), bytes.to_vec());
    }

    fn stored_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.objects
            .lock()
            .expect("direct object map lock should not be poisoned")
            .get(key)
            .cloned()
    }
}

impl ObjectStore for DirectPutObjectStore {
    fn bucket(&self) -> &str {
        &self.bucket
    }

    fn get_object_bytes<'a>(&'a self, object_key: &'a str) -> ObjectStoreFuture<'a, Vec<u8>> {
        Box::pin(async move {
            self.objects
                .lock()
                .expect("direct object map lock should not be poisoned")
                .get(object_key)
                .cloned()
                .ok_or_else(|| WorkerError::ObjectNotFound {
                    operation: "get_object",
                    bucket: self.bucket.clone(),
                    key: object_key.to_owned(),
                    stage: ParseStage::Output,
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
            self.put_content_types
                .lock()
                .expect("direct put content types lock should not be poisoned")
                .push(content_type.to_owned());
            self.insert_object(object_key, bytes);
            Ok(())
        })
    }
}

fn storage_error(
    operation: &'static str,
    bucket: &str,
    key: &str,
    stage: ParseStage,
) -> WorkerError {
    WorkerError::S3 {
        operation,
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        stage,
        retryability: Retryability::Retryable,
        message: "configured fake storage failure".to_owned(),
    }
}

fn checksum(value: &str) -> SourceChecksum {
    SourceChecksum::sha256(value).expect("test checksum should be valid SHA-256")
}

#[derive(Debug, Clone)]
struct RecordedHttpRequest {
    method: String,
    path: String,
    headers: String,
    body: Vec<u8>,
}

impl RecordedHttpRequest {
    fn path_without_query(&self) -> &str {
        self.path.split_once('?').map_or(self.path.as_str(), |(path, _query)| path)
    }
}

#[derive(Debug)]
struct FakeS3HttpServer {
    endpoint: String,
    requests: Arc<Mutex<Vec<RecordedHttpRequest>>>,
    handle: thread::JoinHandle<()>,
}

impl FakeS3HttpServer {
    fn spawn(responses: Vec<HttpResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("fake S3 listener should bind");
        listener.set_nonblocking(false).expect("fake S3 listener should stay blocking");
        let endpoint = format!(
            "http://{}",
            listener.local_addr().expect("fake S3 listener should have address")
        );
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _peer) =
                    listener.accept().expect("fake S3 request should connect");
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .expect("fake S3 read timeout should set");
                let request = read_http_request(&mut stream);
                captured
                    .lock()
                    .expect("fake S3 request list lock should not be poisoned")
                    .push(request);
                stream
                    .write_all(response.to_bytes().as_slice())
                    .expect("fake S3 response should write");
            }
        });

        Self { endpoint, requests, handle }
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn finish(self) -> Vec<RecordedHttpRequest> {
        self.handle.join().expect("fake S3 server thread should finish");
        Arc::try_unwrap(self.requests)
            .expect("fake S3 request list should have one owner")
            .into_inner()
            .expect("fake S3 request list lock should not be poisoned")
    }
}

#[derive(Debug, Clone)]
struct HttpResponse {
    status: &'static str,
    body: Vec<u8>,
}

impl HttpResponse {
    const fn empty(status: &'static str) -> Self {
        Self { status, body: Vec::new() }
    }

    fn xml_error(status: &'static str, code: &str) -> Self {
        Self::xml_error_message(status, code, code)
    }

    fn xml_error_message(status: &'static str, code: &str, message: &str) -> Self {
        Self {
            status,
            body: format!("<Error><Code>{code}</Code><Message>{message}</Message></Error>")
                .into_bytes(),
        }
    }

    fn bytes(status: &'static str, body: &[u8]) -> Self {
        Self { status, body: body.to_vec() }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let content_type = if self.body.starts_with(b"<Error>") {
            "application/xml"
        } else {
            "application/octet-stream"
        };
        let mut response = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\nConnection: close\r\n\r\n",
            self.status,
            self.body.len()
        )
        .into_bytes();
        response.extend_from_slice(&self.body);
        response
    }
}

fn read_http_request(stream: &mut impl Read) -> RecordedHttpRequest {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        let read = stream.read(&mut buffer).expect("fake S3 request should read");
        assert!(read > 0, "fake S3 request ended before headers");
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let header_end = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("fake S3 request should contain header terminator")
        + 4;
    let headers = String::from_utf8(bytes[..header_end].to_vec())
        .expect("fake S3 request headers should be UTF-8");
    let content_length = headers
        .lines()
        .find_map(|line| {
            line.split_once(':').and_then(|(name, value)| {
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("content-length should parse"))
            })
        })
        .unwrap_or(0);
    let mut body = bytes[header_end..].to_vec();
    while body.len() < content_length {
        let read = stream.read(&mut buffer).expect("fake S3 request body should read");
        assert!(read > 0, "fake S3 request body ended early");
        body.extend_from_slice(&buffer[..read]);
    }
    body.truncate(content_length);

    let request_line = headers.lines().next().expect("request line should exist");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().expect("request method should exist").to_owned();
    let path = parts.next().expect("request path should exist").to_owned();
    RecordedHttpRequest { method, path, headers, body }
}

fn s3_store(endpoint: &str) -> S3ObjectStore {
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::new("test", "test", None, None, "test"))
        .endpoint_url(endpoint)
        .force_path_style(true)
        .build();
    S3ObjectStore::new(Client::from_conf(config), "solid-replays")
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

fn artifact_write(reused_existing: bool) -> ArtifactWrite {
    ArtifactWrite {
        reference: ArtifactReference {
            bucket: "solid-replays".to_owned(),
            key: "artifacts/v3/replay-1/source.json".to_owned(),
        },
        checksum: checksum(ABC_SHA256),
        size_bytes: 3,
        reused_existing,
    }
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

#[test]
fn storage_artifact_write_log_event_should_distinguish_created_and_reused() {
    assert_eq!(artifact_write(false).log_event_name(), WORKER_ARTIFACT_WRITTEN);
    assert_eq!(artifact_write(true).log_event_name(), WORKER_ARTIFACT_REUSED);
}

#[tokio::test]
async fn storage_default_get_artifact_should_read_object_bytes() {
    // Arrange
    let key = "artifacts/v3/replay-1/source.json";
    let store = DirectPutObjectStore::with_object(key, br#"{"ok":true}"#);

    // Act
    let bytes = store.get_artifact_bytes(key).await.expect("artifact bytes should read");

    // Assert
    assert_eq!(bytes, br#"{"ok":true}"#);
}

#[tokio::test]
async fn storage_default_conditional_put_should_create_via_plain_put() {
    // Arrange
    let store = DirectPutObjectStore::new();
    let key = "artifacts/v3/replay-1/source.json";
    let bytes = br#"{"ok":true}"#;

    // Act
    let outcome = store
        .put_artifact_bytes_if_absent(key, bytes, "application/json")
        .await
        .expect("default conditional put should create");

    // Assert
    assert_eq!(outcome, ArtifactPutOutcome::Created);
    assert_eq!(store.stored_bytes(key).as_deref(), Some(bytes.as_slice()));
    assert_eq!(
        store
            .put_content_types
            .lock()
            .expect("direct content type lock should not be poisoned")
            .as_slice(),
        ["application/json"]
    );
}

#[tokio::test]
async fn storage_write_artifact_should_use_conditional_create_for_new_object_conditional_put() {
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
    assert_eq!(store.conditional_put_attempt_count(key), 1);
}

#[tokio::test]
async fn storage_write_artifact_should_reuse_matching_object_after_conditional_race_artifact_write_existing_match()
 {
    // Arrange
    let key = "artifacts/v3/replay-1/source.json";
    let bytes = br#"{"ok":true}"#;
    let store = FakeObjectStore::with_object(key, bytes);
    store.set_conditional_put_outcome(key, ArtifactPutOutcome::AlreadyExists);

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
    assert_eq!(store.conditional_put_attempt_count(key), 1);
}

#[tokio::test]
async fn storage_write_artifact_should_conflict_after_conditional_race_with_different_bytes_artifact_write_existing_conflict()
 {
    // Arrange
    let key = "artifacts/v3/replay-1/source.json";
    let store = FakeObjectStore::with_object(key, br#"{"ok":false}"#);
    store.set_conditional_put_outcome(key, ArtifactPutOutcome::AlreadyExists);

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
    assert_eq!(store.conditional_put_attempt_count(key), 1);
}

#[tokio::test]
async fn storage_write_artifact_should_fallback_to_get_then_put_when_conditional_write_unsupported()
{
    // Arrange
    let store = FakeObjectStore::new();
    let key = "artifacts/v3/replay-1/source.json";
    let bytes = br#"{"ok":true}"#;
    store.set_conditional_put_outcome(key, ArtifactPutOutcome::UnsupportedConditionalWrite);

    // Act
    let write = store
        .write_artifact_if_absent_or_matching(key, bytes)
        .await
        .expect("unsupported conditional write should fall back to normal put");

    // Assert
    assert_artifact_write(
        &write,
        key,
        "4062edaf750fb8074e7e83e0c9028c94e32468a8b6f1614774328ef045150f93",
        11,
        false,
    );
    assert_eq!(store.stored_bytes(key).as_deref(), Some(bytes.as_slice()));
    assert_eq!(store.conditional_put_attempt_count(key), 1);
}

#[tokio::test]
async fn storage_write_artifact_should_return_get_error_when_conditional_write_fallback_get_fails()
{
    // Arrange
    let store = FakeObjectStore::new();
    let key = "artifacts/v3/replay-1/source.json";
    store.set_conditional_put_outcome(key, ArtifactPutOutcome::UnsupportedConditionalWrite);
    store.fail_get(key);

    // Act
    let error = store
        .write_artifact_if_absent_or_matching(key, br#"{"ok":true}"#)
        .await
        .expect_err("fallback get failure should be returned");

    // Assert
    match error {
        WorkerError::S3 { operation, stage, retryability, .. } => {
            assert_eq!(operation, "get_object");
            assert_eq!(stage, ParseStage::Input);
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => assert!(other.to_string().contains("S3 operation failed"), "unexpected: {other}"),
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
            assert!(other.to_string().contains("S3 object not found"), "unexpected error: {other}");
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
        WorkerError::S3 { operation, stage, retryability, .. } => {
            assert_eq!(operation, "get_object");
            assert_eq!(stage, ParseStage::Input);
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => {
            assert!(other.to_string().contains("S3 operation failed"), "unexpected error: {other}");
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
        WorkerError::S3 { operation, stage, retryability, .. } => {
            assert_eq!(operation, "put_object");
            assert_eq!(stage, ParseStage::Output);
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => {
            assert!(other.to_string().contains("S3 operation failed"), "unexpected error: {other}");
        }
    }
}

#[tokio::test]
async fn s3_object_store_should_check_ready_download_and_put_against_http_boundary() {
    // Arrange
    let server = FakeS3HttpServer::spawn(vec![
        HttpResponse::empty("200 OK"),
        HttpResponse::bytes("200 OK", b"abc"),
        HttpResponse::empty("200 OK"),
    ]);
    let store = s3_store(server.endpoint());

    // Act
    store.check_ready().await.expect("head bucket should succeed");
    let downloaded =
        store.download_raw("raw/replay.json").await.expect("raw object should download");
    store
        .put_object_bytes("artifacts/v3/replay/source.json", br#"{"ok":true}"#, "application/json")
        .await
        .expect("artifact put should succeed");
    let requests = server.finish();

    // Assert
    assert_eq!(downloaded.bytes, b"abc");
    assert_eq!(downloaded.checksum.value.as_str(), ABC_SHA256);
    assert_eq!(requests[0].method, "HEAD");
    assert_eq!(requests[0].path_without_query(), "/solid-replays/");
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path_without_query(), "/solid-replays/raw/replay.json");
    assert_eq!(requests[2].method, "PUT");
    assert_eq!(requests[2].path_without_query(), "/solid-replays/artifacts/v3/replay/source.json");
    assert!(requests[2].headers.to_ascii_lowercase().contains("content-type: application/json"));
    assert_eq!(requests[2].body, br#"{"ok":true}"#);
}

#[tokio::test]
async fn s3_object_store_should_map_head_bucket_failure_to_retryable_input_error() {
    // Arrange
    let server = FakeS3HttpServer::spawn(vec![HttpResponse::xml_error(
        "500 Internal Server Error",
        "InternalError",
    )]);
    let store = s3_store(server.endpoint());

    // Act
    let error = store.check_ready().await.expect_err("head bucket failure should map");
    let requests = server.finish();

    // Assert
    assert_eq!(requests[0].method, "HEAD");
    match error {
        WorkerError::S3 { operation, bucket, key, stage, retryability, .. } => {
            assert_eq!(operation, "head_bucket");
            assert_eq!(bucket, "solid-replays");
            assert_eq!(key, "");
            assert_eq!(stage, ParseStage::Input);
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => assert!(other.to_string().contains("S3 operation failed"), "unexpected: {other}"),
    }
}

#[tokio::test]
async fn s3_object_store_should_map_put_failure_to_retryable_output_error() {
    // Arrange
    let server = FakeS3HttpServer::spawn(vec![HttpResponse::xml_error(
        "503 Service Unavailable",
        "SlowDown",
    )]);
    let store = s3_store(server.endpoint());

    // Act
    let error = store
        .put_object_bytes("artifacts/v3/replay/source.json", br#"{"ok":true}"#, "application/json")
        .await
        .expect_err("put object failure should map");
    let requests = server.finish();

    // Assert
    assert_eq!(requests[0].method, "PUT");
    match error {
        WorkerError::S3 { operation, key, stage, retryability, .. } => {
            assert_eq!(operation, "put_object");
            assert_eq!(key, "artifacts/v3/replay/source.json");
            assert_eq!(stage, ParseStage::Output);
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => assert!(other.to_string().contains("S3 operation failed"), "unexpected: {other}"),
    }
}

#[tokio::test]
async fn s3_object_store_should_classify_conditional_put_responses() {
    // Arrange
    let server = FakeS3HttpServer::spawn(vec![
        HttpResponse::empty("200 OK"),
        HttpResponse::xml_error("412 Precondition Failed", "PreconditionFailed"),
        HttpResponse::xml_error("501 Not Implemented", "NotImplemented"),
        HttpResponse::xml_error_message(
            "400 Bad Request",
            "InvalidRequest",
            "If-None-Match conditional writes are not supported",
        ),
    ]);
    let store = s3_store(server.endpoint());

    // Act
    let created = store
        .put_artifact_bytes_if_absent("artifacts/created.json", b"{}", "application/json")
        .await
        .expect("created conditional put should succeed");
    let already_exists = store
        .put_artifact_bytes_if_absent("artifacts/existing.json", b"{}", "application/json")
        .await
        .expect("precondition failure should map to existing object");
    let unsupported = store
        .put_artifact_bytes_if_absent("artifacts/unsupported.json", b"{}", "application/json")
        .await
        .expect("not implemented should map to unsupported conditional write");
    let unsupported_invalid_request = store
        .put_artifact_bytes_if_absent(
            "artifacts/unsupported-invalid-request.json",
            b"{}",
            "application/json",
        )
        .await
        .expect("invalid request mentioning if-none-match should map to unsupported write");
    let requests = server.finish();

    // Assert
    assert_eq!(created, ArtifactPutOutcome::Created);
    assert_eq!(already_exists, ArtifactPutOutcome::AlreadyExists);
    assert_eq!(unsupported, ArtifactPutOutcome::UnsupportedConditionalWrite);
    assert_eq!(unsupported_invalid_request, ArtifactPutOutcome::UnsupportedConditionalWrite);
    assert_eq!(requests.iter().filter(|request| request.method == "PUT").count(), 4);
    assert!(
        requests
            .iter()
            .all(|request| { request.headers.to_ascii_lowercase().contains("if-none-match: *") })
    );
}

#[tokio::test]
async fn s3_object_store_should_return_retryable_error_for_unclassified_conditional_put_failure() {
    // Arrange
    let server = FakeS3HttpServer::spawn(vec![HttpResponse::xml_error(
        "500 Internal Server Error",
        "InternalError",
    )]);
    let store = s3_store(server.endpoint());

    // Act
    let error = store
        .put_artifact_bytes_if_absent("artifacts/unclassified.json", b"{}", "application/json")
        .await
        .expect_err("unclassified conditional put failure should map to S3 error");
    let requests = server.finish();

    // Assert
    assert_eq!(requests[0].method, "PUT");
    match error {
        WorkerError::S3 { operation, key, stage, retryability, .. } => {
            assert_eq!(operation, "put_object");
            assert_eq!(key, "artifacts/unclassified.json");
            assert_eq!(stage, ParseStage::Output);
            assert_eq!(retryability, Retryability::Retryable);
        }
        other => assert!(other.to_string().contains("S3 operation failed"), "unexpected: {other}"),
    }
}

#[tokio::test]
async fn s3_object_store_should_map_no_such_key_to_object_not_found() {
    // Arrange
    let server =
        FakeS3HttpServer::spawn(vec![HttpResponse::xml_error("404 Not Found", "NoSuchKey")]);
    let store = s3_store(server.endpoint());

    // Act
    let error = store
        .get_object_bytes("raw/missing.json")
        .await
        .expect_err("missing object should be mapped");
    let requests = server.finish();

    // Assert
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path_without_query(), "/solid-replays/raw/missing.json");
    match error {
        WorkerError::ObjectNotFound { operation, key, stage, retryability, .. } => {
            assert_eq!(operation, "get_object");
            assert_eq!(key, "raw/missing.json");
            assert_eq!(stage, ParseStage::Input);
            assert_eq!(retryability, Retryability::Unknown);
        }
        other => {
            assert!(other.to_string().contains("S3 object not found"), "unexpected error: {other}");
        }
    }
}

#[tokio::test]
async fn s3_object_store_should_map_missing_artifact_to_output_stage() {
    // Arrange
    let server =
        FakeS3HttpServer::spawn(vec![HttpResponse::xml_error("404 Not Found", "NoSuchKey")]);
    let store = s3_store(server.endpoint());

    // Act
    let error = store
        .get_artifact_bytes("artifacts/missing.json")
        .await
        .expect_err("missing artifact should be mapped");
    let requests = server.finish();

    // Assert
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path_without_query(), "/solid-replays/artifacts/missing.json");
    match error {
        WorkerError::ObjectNotFound { operation, key, stage, retryability, .. } => {
            assert_eq!(operation, "get_object");
            assert_eq!(key, "artifacts/missing.json");
            assert_eq!(stage, ParseStage::Output);
            assert_eq!(retryability, Retryability::Unknown);
        }
        other => assert!(other.to_string().contains("S3 object not found"), "unexpected: {other}"),
    }
}

#[test]
fn s3_object_store_debug_should_include_bucket_without_client_details() {
    // Arrange
    let server = FakeS3HttpServer::spawn(Vec::new());
    let store = s3_store(server.endpoint());

    // Act
    let debug = format!("{store:?}");
    let requests = server.finish();

    // Assert
    assert!(requests.is_empty());
    assert!(debug.contains("S3ObjectStore"));
    assert!(debug.contains("solid-replays"));
    assert!(debug.contains(".."));
}
