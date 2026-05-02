//! Live worker smoke test against `RabbitMQ` and `S3`-compatible storage.

#![allow(
    clippy::expect_used,
    reason = "live smoke tests use expect messages as infrastructure diagnostics"
)]

use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Region, primitives::ByteStream};
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{
        BasicAckOptions, BasicGetOptions, BasicPublishOptions, ConfirmSelectOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions, QueuePurgeOptions,
    },
    types::FieldTable,
};
use parser_contract::{
    failure::{ParseStage, Retryability},
    source_ref::SourceChecksum,
    version::ContractVersion,
    worker::{ParseCompletedMessage, ParseFailedMessage, ParseJobMessage},
};
use parser_worker::{
    artifact_key::artifact_key, checksum::source_checksum_from_bytes, config::WorkerConfig,
    runner::run_until_cancelled,
};
use tokio::{task::JoinHandle, time::timeout};
use tokio_util::sync::CancellationToken;

const VALID_REPLAY: &[u8] =
    include_bytes!("../../parser-core/tests/fixtures/valid-minimal.ocap.json");
const RAW_KEY: &str = "raw/replay-smoke.ocap.json";
const COMPLETED_QUEUE: &str = "parse.completed.smoke";
const FAILED_QUEUE: &str = "parse.failed.smoke";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires local RabbitMQ and S3-compatible storage; run scripts/worker-smoke.sh"]
async fn live_worker_should_process_completed_and_failed_jobs_through_broker_and_s3() {
    assert_eq!(
        env::var("REPLAY_PARSER_LIVE_SMOKE").unwrap_or_default(),
        "1",
        "set REPLAY_PARSER_LIVE_SMOKE=1 or use scripts/worker-smoke.sh"
    );

    let config = WorkerConfig::from_env().expect("live smoke worker config should be valid");
    let container_smoke = container_smoke_enabled();
    let setup_only =
        env::var("REPLAY_PARSER_CONTAINER_SMOKE_SETUP_ONLY").unwrap_or_default() == "1";
    let checksum = source_checksum_from_bytes(VALID_REPLAY)
        .expect("fixture checksum should be internally valid");
    let s3 = s3_client(&config).await;
    ensure_bucket(&s3, &config.s3_bucket).await;
    delete_smoke_objects(&s3, &config, &checksum).await;
    let _put_object = s3
        .put_object()
        .bucket(&config.s3_bucket)
        .key(RAW_KEY)
        .body(ByteStream::from(VALID_REPLAY.to_vec()))
        .send()
        .await
        .expect("raw replay fixture should upload to S3-compatible storage");

    let amqp = Connection::connect(&config.amqp_url, ConnectionProperties::default())
        .await
        .expect("RabbitMQ should accept smoke setup connection");
    let channel = amqp.create_channel().await.expect("RabbitMQ setup channel should open");
    prepare_broker(&channel, &config).await;
    if setup_only {
        return;
    }

    let shutdown = CancellationToken::new();
    let worker = (!container_smoke).then(|| spawn_worker(config.clone(), shutdown.clone()));
    if container_smoke {
        assert_container_worker_probes().await;
    }

    publish_job(
        &channel,
        &config.job_queue,
        "job-smoke-duplicate",
        "replay-smoke-duplicate",
        &checksum,
    )
    .await;
    publish_job(
        &channel,
        &config.job_queue,
        "job-smoke-duplicate",
        "replay-smoke-duplicate",
        &checksum,
    )
    .await;
    let first_completed = wait_for_completed(&channel).await;
    let second_completed = wait_for_completed(&channel).await;
    assert_duplicate_completed_results(&config, &checksum, &first_completed, &second_completed);
    assert_artifact_exists(&s3, &first_completed).await;
    assert_single_artifact_key(&s3, &first_completed).await;
    assert_queue_empty(&channel, &config.job_queue).await;

    let bad_checksum =
        SourceChecksum::sha256("0".repeat(64)).expect("test checksum should be valid SHA-256");
    publish_job(&channel, &config.job_queue, "job-smoke-failed", "replay-smoke-002", &bad_checksum)
        .await;
    let failed = wait_for_failed(&channel).await;
    assert_eq!(failed.failure.error_code.as_str(), "checksum.mismatch");
    assert_eq!(failed.failure.stage, ParseStage::Checksum);
    assert_eq!(failed.failure.retryability, Retryability::NotRetryable);
    assert_queue_empty(&channel, &config.job_queue).await;

    let conflict_key = put_conflicting_artifact(&s3, &config, &checksum).await;
    publish_job(
        &channel,
        &config.job_queue,
        "job-smoke-conflict",
        "replay-smoke-conflict",
        &checksum,
    )
    .await;
    let failed = wait_for_failed(&channel).await;
    assert_eq!(failed.failure.error_code.as_str(), "output.artifact_conflict");
    assert_eq!(failed.failure.stage, ParseStage::Output);
    assert_eq!(
        conflict_key,
        artifact_key(&config.artifact_prefix, "replay-smoke-conflict", &checksum)
            .expect("artifact key should be deterministic")
    );
    assert_queue_empty(&channel, &config.job_queue).await;

    if container_smoke {
        assert_container_worker_probes().await;
    }
    if let Some(worker) = worker {
        shutdown.cancel();
        timeout(Duration::from_secs(10), worker)
            .await
            .expect("worker should stop after cancellation")
            .expect("worker task should not panic")
            .expect("worker should exit cleanly");
    }
}

async fn s3_client(config: &WorkerConfig) -> aws_sdk_s3::Client {
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

fn container_smoke_enabled() -> bool {
    env::var("REPLAY_PARSER_CONTAINER_SMOKE").unwrap_or_default() == "1"
}

async fn ensure_bucket(client: &aws_sdk_s3::Client, bucket: &str) {
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

async fn delete_smoke_objects(
    client: &aws_sdk_s3::Client,
    config: &WorkerConfig,
    checksum: &SourceChecksum,
) {
    let duplicate_key = artifact_key(&config.artifact_prefix, "replay-smoke-duplicate", checksum)
        .expect("duplicate artifact key should be deterministic");
    let conflict_key = artifact_key(&config.artifact_prefix, "replay-smoke-conflict", checksum)
        .expect("conflict artifact key should be deterministic");
    for key in [RAW_KEY.to_owned(), duplicate_key, conflict_key] {
        let _delete_result = client.delete_object().bucket(&config.s3_bucket).key(key).send().await;
    }
}

async fn prepare_broker(channel: &Channel, config: &WorkerConfig) {
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
        .expect("smoke publish channel should enable confirms");

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
        let _purged_count = channel
            .queue_purge(queue.into(), QueuePurgeOptions::default())
            .await
            .expect("smoke queues should purge");
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

fn spawn_worker(
    config: WorkerConfig,
    shutdown: CancellationToken,
) -> JoinHandle<Result<(), parser_worker::error::WorkerError>> {
    tokio::spawn(async move { run_until_cancelled(config, shutdown).await })
}

async fn publish_job(
    channel: &Channel,
    queue: &str,
    job_id: &str,
    replay_id: &str,
    checksum: &SourceChecksum,
) {
    let body = serde_json::to_vec(&ParseJobMessage {
        job_id: job_id.to_owned(),
        replay_id: replay_id.to_owned(),
        object_key: RAW_KEY.to_owned(),
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

fn assert_duplicate_completed_results(
    config: &WorkerConfig,
    checksum: &SourceChecksum,
    first: &ParseCompletedMessage,
    second: &ParseCompletedMessage,
) {
    let expected_key = artifact_key(&config.artifact_prefix, "replay-smoke-duplicate", checksum)
        .expect("artifact key should be deterministic");
    for completed in [first, second] {
        assert_eq!(completed.job_id, "job-smoke-duplicate");
        assert_eq!(completed.replay_id, "replay-smoke-duplicate");
        assert_eq!(&completed.source_checksum, checksum);
        assert_eq!(completed.artifact.key, expected_key);
    }
    assert_eq!(first.artifact.key, second.artifact.key);
    assert_eq!(first.artifact_checksum, second.artifact_checksum);
    assert_eq!(first.artifact_size_bytes, second.artifact_size_bytes);
}

async fn put_conflicting_artifact(
    client: &aws_sdk_s3::Client,
    config: &WorkerConfig,
    checksum: &SourceChecksum,
) -> String {
    let key = artifact_key(&config.artifact_prefix, "replay-smoke-conflict", checksum)
        .expect("conflict artifact key should be deterministic");
    let _put_object = client
        .put_object()
        .bucket(&config.s3_bucket)
        .key(&key)
        .content_type("application/json")
        .body(ByteStream::from(br#"{"phase_07":"conflict"}"#.to_vec()))
        .send()
        .await
        .expect("conflicting artifact should pre-seed");
    key
}

async fn wait_for_completed(channel: &Channel) -> ParseCompletedMessage {
    let delivery = wait_for_delivery(channel, COMPLETED_QUEUE).await;
    let message: ParseCompletedMessage =
        serde_json::from_slice(&delivery.data).expect("completed result should deserialize");
    assert!(
        delivery.ack(BasicAckOptions::default()).await.expect("completed result should ack"),
        "completed result ack should be accepted"
    );
    message
}

async fn wait_for_failed(channel: &Channel) -> ParseFailedMessage {
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
    timeout(Duration::from_secs(45), async {
        loop {
            if let Some(delivery) = channel
                .basic_get(queue.into(), BasicGetOptions { no_ack: false })
                .await
                .expect("result queue should support basic.get")
            {
                return delivery.delivery;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    })
    .await
    .expect("result message should arrive before timeout")
}

async fn assert_artifact_exists(client: &aws_sdk_s3::Client, completed: &ParseCompletedMessage) {
    let object = client
        .get_object()
        .bucket(&completed.artifact.bucket)
        .key(&completed.artifact.key)
        .send()
        .await
        .expect("completed artifact should exist in S3-compatible storage");
    let bytes = object.body.collect().await.expect("artifact body should collect").into_bytes();
    let checksum = source_checksum_from_bytes(&bytes).expect("artifact checksum should validate");
    assert_eq!(checksum, completed.artifact_checksum);
    assert_eq!(
        u64::try_from(bytes.len()).expect("artifact length should fit u64"),
        completed.artifact_size_bytes
    );
}

async fn assert_single_artifact_key(
    client: &aws_sdk_s3::Client,
    completed: &ParseCompletedMessage,
) {
    let listed = client
        .list_objects_v2()
        .bucket(&completed.artifact.bucket)
        .prefix(&completed.artifact.key)
        .send()
        .await
        .expect("artifact prefix should list");
    let matching_count = listed
        .contents()
        .iter()
        .filter(|object| object.key() == Some(completed.artifact.key.as_str()))
        .count();
    assert_eq!(matching_count, 1, "exact deterministic artifact key should exist once");
}

async fn assert_container_worker_probes() {
    for port in [
        probe_port_from_env("WORKER_A_PROBE_PORT", 18_081),
        probe_port_from_env("WORKER_B_PROBE_PORT", 18_082),
    ] {
        wait_for_probe(port, "/livez").await;
        wait_for_probe(port, "/readyz").await;
    }
}

fn probe_port_from_env(name: &str, default: u16) -> u16 {
    env::var(name).ok().and_then(|value| value.parse::<u16>().ok()).unwrap_or(default)
}

async fn wait_for_probe(port: u16, path: &str) {
    let result = timeout(Duration::from_secs(45), async {
        loop {
            if probe_http_ok(port, path) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    })
    .await;
    assert!(result.is_ok(), "probe {path} on port {port} should return HTTP 200");
}

fn probe_http_ok(port: u16, path: &str) -> bool {
    let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) else {
        return false;
    };
    let timeout = Some(Duration::from_secs(2));
    if stream.set_read_timeout(timeout).and_then(|()| stream.set_write_timeout(timeout)).is_err() {
        return false;
    }
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut response = [0_u8; 128];
    let Ok(read) = stream.read(&mut response) else {
        return false;
    };
    response[..read].starts_with(b"HTTP/1.1 200") || response[..read].starts_with(b"HTTP/1.0 200")
}

async fn assert_queue_empty(channel: &Channel, queue: &str) {
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
