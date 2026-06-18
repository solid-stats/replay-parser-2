//! Shared test helpers for the worker integration tests.
//!
//! Promoted from `live_smoke.rs` so the container e2e (`golden_container_e2e.rs`) and any
//! future live test reuse ONE definition of the broker/bucket wiring rather than
//! duplicating it. These helpers are boundary-diagnostic only — `expect` is permitted
//! here as infrastructure diagnostics, mirroring `live_smoke.rs:3-6`.

#![allow(
    clippy::expect_used,
    dead_code,
    unreachable_pub,
    reason = "shared worker test helpers; expect messages are infra diagnostics, helpers are \
              `pub` for cross-test-file reuse, and not every helper is used by every target"
)]

use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Region, primitives::ByteStream};
use lapin::{
    BasicProperties, Channel, ExchangeKind,
    options::{
        BasicAckOptions, BasicGetOptions, BasicPublishOptions, ConfirmSelectOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions, QueuePurgeOptions,
    },
    types::FieldTable,
};
use parser_contract::{
    source_ref::SourceChecksum,
    version::ContractVersion,
    worker::{ParseCompletedMessage, ParseFailedMessage, ParseJobMessage},
};
use parser_worker::{config::WorkerConfig, runner::run_until_cancelled};
use std::time::Duration;
use tokio::{task::JoinHandle, time::timeout};
use tokio_util::sync::CancellationToken;

/// Result queue bound on `completed_routing_key`.
pub const COMPLETED_QUEUE: &str = "parse.completed.e2e";
/// Result queue bound on `failed_routing_key`.
pub const FAILED_QUEUE: &str = "parse.failed.e2e";

/// Builds an `aws-sdk-s3` client honoring the config endpoint + path-style, exactly as
/// `S3ObjectStore::from_config` does.
pub async fn s3_client(config: &WorkerConfig) -> aws_sdk_s3::Client {
    let mut loader = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(config.s3_region.clone()));
    if let Some(endpoint) = &config.s3_endpoint {
        loader = loader.endpoint_url(endpoint);
    }
    let shared = loader.load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&shared)
        .force_path_style(config.s3_force_path_style)
        .build();
    aws_sdk_s3::Client::from_conf(s3_config)
}

/// Creates the bucket if absent (idempotent). The worker `head_bucket`-gates readiness,
/// so this must run before spawning the worker.
pub async fn ensure_bucket(client: &aws_sdk_s3::Client, bucket: &str) {
    let result = client.create_bucket().bucket(bucket).send().await;
    if let Err(error) = result {
        if client.head_bucket().bucket(bucket).send().await.is_ok() {
            return;
        }
        let message = error.to_string();
        assert!(
            message.contains("BucketAlreadyOwnedByYou")
                || message.contains("BucketAlreadyExists")
                || message.contains("bucket already exists")
                || message.contains("Your previous request to create the named bucket succeeded"),
            "bucket creation failed unexpectedly: {message}"
        );
    }
}

/// Uploads raw replay bytes at `object_key` so the worker download succeeds.
pub async fn put_raw_object(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    object_key: &str,
    bytes: &[u8],
) {
    let _put = client
        .put_object()
        .bucket(bucket)
        .key(object_key)
        .body(ByteStream::from(bytes.to_vec()))
        .send()
        .await
        .expect("raw replay fixture should upload to S3-compatible storage");
}

/// Declares the topology the worker assumes already exists: job queue, result exchange,
/// and the two bound result queues. The worker declares NONE of these.
pub async fn prepare_broker(channel: &Channel, config: &WorkerConfig) {
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
        .expect("setup publish channel should enable confirms");

    let _job_queue = channel
        .queue_declare(
            config.job_queue.as_str().into(),
            QueueDeclareOptions { durable: true, ..Default::default() },
            FieldTable::default(),
        )
        .await
        .expect("job queue should declare");
    channel
        .exchange_declare(
            config.result_exchange.as_str().into(),
            ExchangeKind::Direct,
            ExchangeDeclareOptions { durable: true, ..Default::default() },
            FieldTable::default(),
        )
        .await
        .expect("result exchange should declare");

    declare_result_queue(channel, COMPLETED_QUEUE, config, &config.completed_routing_key).await;
    declare_result_queue(channel, FAILED_QUEUE, config, &config.failed_routing_key).await;

    for queue in [config.job_queue.as_str(), COMPLETED_QUEUE, FAILED_QUEUE] {
        let _purged = channel
            .queue_purge(queue.into(), QueuePurgeOptions::default())
            .await
            .expect("setup queues should purge");
    }
}

async fn declare_result_queue(
    channel: &Channel,
    queue: &str,
    config: &WorkerConfig,
    routing_key: &str,
) {
    let _result_queue = channel
        .queue_declare(
            queue.into(),
            QueueDeclareOptions { durable: true, ..Default::default() },
            FieldTable::default(),
        )
        .await
        .expect("result queue should declare");
    channel
        .queue_bind(
            queue.into(),
            config.result_exchange.as_str().into(),
            routing_key.into(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("result queue should bind");
}

/// Spawns the real worker driven only by the cancellation token (no signal handler,
/// `listen_for_ctrl_c=false` inside `run_until_cancelled`).
pub fn spawn_worker(
    config: WorkerConfig,
    shutdown: CancellationToken,
) -> JoinHandle<Result<(), parser_worker::error::WorkerError>> {
    tokio::spawn(async move { run_until_cancelled(config, shutdown).await })
}

/// Cancels the token and joins the worker, asserting a clean exit within the timeout.
pub async fn stop_worker(
    shutdown: &CancellationToken,
    worker: JoinHandle<Result<(), parser_worker::error::WorkerError>>,
) {
    shutdown.cancel();
    timeout(Duration::from_secs(10), worker)
        .await
        .expect("worker should stop after cancellation")
        .expect("worker task should not panic")
        .expect("worker should exit cleanly");
}

/// Publishes a parse job to the job queue with publisher confirms.
pub async fn publish_job(
    channel: &Channel,
    queue: &str,
    job_id: &str,
    replay_id: &str,
    object_key: &str,
    checksum: &SourceChecksum,
) {
    let body = serde_json::to_vec(&ParseJobMessage {
        job_id: job_id.to_owned(),
        replay_id: replay_id.to_owned(),
        object_key: object_key.to_owned(),
        checksum: checksum.clone(),
        parser_contract_version: ContractVersion::current(),
    })
    .expect("parse job should serialize");

    let confirm = channel
        .basic_publish(
            "".into(),
            queue.into(),
            BasicPublishOptions::default(),
            &body,
            BasicProperties::default()
                .with_content_type("application/json".into())
                .with_delivery_mode(2),
        )
        .await
        .expect("parse job should publish")
        .await
        .expect("parse job publish should confirm");
    assert!(confirm.is_ack(), "parse job publish should be acked by RabbitMQ");
}

/// Polls the completed result queue until a message arrives, acks it, and decodes it.
pub async fn wait_for_completed(channel: &Channel) -> ParseCompletedMessage {
    let delivery = wait_for_delivery(channel, COMPLETED_QUEUE).await;
    let message: ParseCompletedMessage =
        serde_json::from_slice(&delivery.data).expect("completed result should deserialize");
    assert!(
        delivery.ack(BasicAckOptions::default()).await.expect("completed result should ack"),
        "completed result ack should be accepted"
    );
    message
}

/// Polls the failed result queue until a message arrives, acks it, and decodes it.
pub async fn wait_for_failed(channel: &Channel) -> ParseFailedMessage {
    let delivery = wait_for_delivery(channel, FAILED_QUEUE).await;
    let message: ParseFailedMessage =
        serde_json::from_slice(&delivery.data).expect("failed result should deserialize");
    assert!(
        delivery.ack(BasicAckOptions::default()).await.expect("failed result should ack"),
        "failed result ack should be accepted"
    );
    message
}

async fn wait_for_delivery(channel: &Channel, queue: &str) -> lapin::message::Delivery {
    timeout(Duration::from_mins(1), async {
        loop {
            if let Some(delivery) = channel
                .basic_get(queue.into(), BasicGetOptions { no_ack: false })
                .await
                .expect("result queue should support basic.get")
            {
                return delivery.delivery;
            }
            // Test-side result polling — NOT a worker timer (the worker path uses none).
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    })
    .await
    .expect("result message should arrive before timeout")
}

/// GETs the artifact at `completed.artifact.key` and returns the raw bytes for a
/// byte-exact comparison against the committed baseline.
pub async fn fetch_artifact_bytes(
    client: &aws_sdk_s3::Client,
    completed: &ParseCompletedMessage,
) -> Vec<u8> {
    let object = client
        .get_object()
        .bucket(&completed.artifact.bucket)
        .key(&completed.artifact.key)
        .send()
        .await
        .expect("completed artifact should exist in S3-compatible storage");
    object
        .body
        .collect()
        .await
        .expect("artifact body should collect")
        .into_bytes()
        .to_vec()
}

/// Counts objects whose key exactly equals `completed.artifact.key` (idempotency check).
pub async fn count_artifact_keys(
    client: &aws_sdk_s3::Client,
    completed: &ParseCompletedMessage,
) -> usize {
    let listed = client
        .list_objects_v2()
        .bucket(&completed.artifact.bucket)
        .prefix(&completed.artifact.key)
        .send()
        .await
        .expect("artifact prefix should list");
    listed
        .contents()
        .iter()
        .filter(|object| object.key() == Some(completed.artifact.key.as_str()))
        .count()
}

/// Pre-seeds a CONFLICTING artifact (bytes differing from the worker's output) at the
/// deterministic artifact key for `replay_id`, returning that key. Triggers the
/// conditional-PUT conflict path on the next matching job.
pub async fn put_conflicting_artifact(
    client: &aws_sdk_s3::Client,
    config: &WorkerConfig,
    replay_id: &str,
    checksum: &SourceChecksum,
) -> String {
    let key = parser_worker::artifact_key::artifact_key(&config.artifact_prefix, replay_id, checksum)
        .expect("conflict artifact key should be deterministic");
    let _put = client
        .put_object()
        .bucket(&config.s3_bucket)
        .key(&key)
        .content_type("application/json")
        .body(ByteStream::from(br#"{"conflict":"differing-bytes"}"#.to_vec()))
        .send()
        .await
        .expect("conflicting artifact should pre-seed");
    key
}

/// Passively declares the queue and asserts it is empty (all jobs acked).
pub async fn assert_queue_empty(channel: &Channel, queue: &str) {
    let declared = channel
        .queue_declare(
            queue.into(),
            QueueDeclareOptions { passive: true, durable: true, ..Default::default() },
            FieldTable::default(),
        )
        .await
        .expect("queue should passively declare");
    assert_eq!(declared.message_count(), 0, "queue {queue} should be empty after worker ack");
}
