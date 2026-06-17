//! Golden container e2e regression oracle for the parser worker.

// Boots ephemeral RabbitMQ + MinIO via testcontainers, drives the REAL worker through
// the `run_until_cancelled` + `CancellationToken` seam, and pins the full observable
// contract: the worker-written S3 artifact bytes equal the committed golden baseline
// byte-for-byte, plus the `parse.completed`/`parse.failed` message shape, artifact key
// + checksum + size, duplicate-redelivery idempotency, a checksum-mismatch failure,
// and an artifact-conflict failure.
//
// `#[ignore]`: requires a Docker daemon. It additionally skips cleanly (early `Ok`)
// when Docker, the seed fixture, or the MinIO credentials are absent, so
// `cargo test --workspace` stays green with no Docker. It is excluded from the coverage
// gate (which never passes `--ignored`) and runs only as a master-only pre-deploy CI job.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "container e2e uses expect/assert as infra diagnostics and eprintln for skip notices"
)]

mod common;

use std::path::Path;

use aws_sdk_s3::Client;
use lapin::Channel;
use parser_contract::{
    failure::{ParseStage, Retryability},
    source_ref::SourceChecksum,
};
use parser_worker::{
    artifact_key::artifact_key, checksum::source_checksum_from_bytes, config::WorkerConfig,
    config::WorkerConfigOverrides,
};
use testcontainers_modules::{
    minio::MinIO,
    rabbitmq::RabbitMq,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};
use tokio_util::sync::CancellationToken;

// Shared, single-source-of-truth pinned identity (same file the parser-core fast golden
// test includes). Textually included so both crates agree on every embedded value.
include!("../../parser-core/tests/common/golden_identity.rs");

const SEED_FIXTURE: &str = "../parser-core/tests/fixtures/valid-minimal.ocap.json";
const EXPECTED_BASELINE: &[u8] =
    include_bytes!("../../parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json");
const BUCKET: &str = "solid-replays";

type BoxError = Box<dyn std::error::Error>;

// Booted container handles + endpoints, held for the test lifetime (drop = stop).
struct Infra {
    _minio: ContainerAsync<MinIO>,
    _rabbit: ContainerAsync<RabbitMq>,
    s3_endpoint: String,
    amqp_url: String,
}

// Enforces the version coupling LOUDLY and WITHOUT Docker. The worker embeds its own
// `env!("CARGO_PKG_VERSION")` (parser-worker's) into the artifact it writes to S3, while
// the committed baseline is pinned to `GOLDEN_PARSER_VERSION`. The byte-exact e2e assert
// would catch a divergence, but only when Docker is present and only as a confusing
// "parser drift" byte-diff. This fast guard runs under plain `cargo test --workspace`
// and turns a future parser-worker version bump into an explicit, self-explaining
// failure instead.
#[test]
fn worker_package_version_must_match_pinned_golden_baseline_version() {
    assert_eq!(
        env!("CARGO_PKG_VERSION"),
        GOLDEN_PARSER_VERSION,
        "parser-worker package version diverged from the pinned golden baseline version \
         (GOLDEN_PARSER_VERSION in tests/common/golden_identity.rs). The worker embeds its \
         own CARGO_PKG_VERSION into the artifact, so this divergence would make the golden \
         container e2e fail byte-for-byte and read as spurious parser drift. Either align \
         the crate version or regenerate valid-minimal.expected.json and bump \
         GOLDEN_PARSER_VERSION together."
    );
}

// Boots MinIO + RabbitMQ, returning `Ok(None)` (skip) when Docker is unavailable.
async fn boot_infra() -> Result<Option<Infra>, BoxError> {
    let minio = match MinIO::default().start().await {
        Ok(container) => container,
        Err(error) => {
            eprintln!("SKIP golden_container_e2e: Docker unavailable ({error})");
            return Ok(None);
        }
    };
    let rabbit = RabbitMq::default().start().await?;
    let s3_endpoint =
        format!("http://{}:{}", minio.get_host().await?, minio.get_host_port_ipv4(9000).await?);
    let amqp_url = format!(
        "amqp://guest:guest@{}:{}/%2f",
        rabbit.get_host().await?,
        rabbit.get_host_port_ipv4(5672).await?
    );
    Ok(Some(Infra { _minio: minio, _rabbit: rabbit, s3_endpoint, amqp_url }))
}

fn build_config(infra: &Infra) -> Result<WorkerConfig, BoxError> {
    Ok(WorkerConfig::from_env_and_overrides(
        |_| None,
        WorkerConfigOverrides {
            amqp_url: Some(infra.amqp_url.clone()),
            s3_bucket: Some(BUCKET.to_owned()),
            s3_endpoint: Some(infra.s3_endpoint.clone()),
            s3_force_path_style: Some(true),
            s3_region: Some("us-east-1".to_owned()),
            probes_enabled: Some(false),
            prefetch: Some(1),
            worker_id: Some("e2e-oracle".to_owned()),
            ..Default::default()
        },
    )?)
}

// SUCCESS: byte-exact artifact + full `parse.completed` message contract.
async fn assert_success_contract(
    channel: &Channel,
    s3: &Client,
    config: &WorkerConfig,
    checksum: &SourceChecksum,
) -> Result<String, BoxError> {
    common::publish_job(
        channel,
        &config.job_queue,
        "job-golden-success",
        GOLDEN_REPLAY_ID,
        GOLDEN_OBJECT_KEY,
        checksum,
    )
    .await;
    let completed = common::wait_for_completed(channel).await;
    let fetched = common::fetch_artifact_bytes(s3, &completed).await;

    assert_eq!(fetched, EXPECTED_BASELINE, "S3 artifact bytes must equal the committed baseline");
    let expected_key = artifact_key(&config.artifact_prefix, GOLDEN_REPLAY_ID, checksum)?;
    assert_eq!(completed.artifact.key, expected_key, "artifact key must be deterministic");
    assert_eq!(completed.artifact.bucket, config.s3_bucket);
    assert_eq!(
        completed.artifact_checksum,
        source_checksum_from_bytes(&fetched),
        "completed.artifact_checksum must match the fetched bytes"
    );
    assert_eq!(
        completed.artifact_size_bytes,
        u64::try_from(fetched.len()).expect("artifact length should fit u64"),
        "completed.artifact_size_bytes must match the fetched bytes"
    );
    assert_eq!(completed.job_id, "job-golden-success");
    assert_eq!(completed.replay_id, GOLDEN_REPLAY_ID);
    assert_eq!(completed.source_checksum, *checksum);
    common::assert_queue_empty(channel, &config.job_queue).await;
    Ok(expected_key)
}

// IDEMPOTENCY: duplicate redelivery → both completed, single artifact at the key.
async fn assert_idempotency(
    channel: &Channel,
    s3: &Client,
    config: &WorkerConfig,
    checksum: &SourceChecksum,
    expected_key: &str,
) {
    for _ in 0..2 {
        common::publish_job(
            channel,
            &config.job_queue,
            "job-golden-dup",
            GOLDEN_REPLAY_ID,
            GOLDEN_OBJECT_KEY,
            checksum,
        )
        .await;
    }
    let first = common::wait_for_completed(channel).await;
    let second = common::wait_for_completed(channel).await;
    assert_eq!(first.artifact.key, second.artifact.key);
    assert_eq!(first.artifact_checksum, second.artifact_checksum);
    assert_eq!(first.artifact_size_bytes, second.artifact_size_bytes);
    assert_eq!(first.artifact.key, expected_key);
    assert_eq!(
        common::count_artifact_keys(s3, &first).await,
        1,
        "duplicate redelivery must leave exactly one artifact at the deterministic key"
    );
    common::assert_queue_empty(channel, &config.job_queue).await;
}

// CHECKSUM-MISMATCH → parse.failed (checksum.mismatch / Checksum / NotRetryable).
async fn assert_checksum_mismatch_failure(
    channel: &Channel,
    config: &WorkerConfig,
) -> Result<(), BoxError> {
    let bad_checksum = SourceChecksum::sha256("0".repeat(64))?;
    common::publish_job(
        channel,
        &config.job_queue,
        "job-golden-bad-checksum",
        "replay-golden-bad-checksum",
        GOLDEN_OBJECT_KEY,
        &bad_checksum,
    )
    .await;
    let failed = common::wait_for_failed(channel).await;
    assert_eq!(failed.failure.error_code.as_str(), "checksum.mismatch");
    assert_eq!(failed.failure.stage, ParseStage::Checksum);
    assert_eq!(failed.failure.retryability, Retryability::NotRetryable);
    common::assert_queue_empty(channel, &config.job_queue).await;
    Ok(())
}

// ARTIFACT-CONFLICT → parse.failed (output.artifact_conflict / Output).
//
// Uses a DISTINCT replay_id so the conflicting pre-seed cannot poison the byte-exact
// success assertion.
async fn assert_artifact_conflict_failure(
    channel: &Channel,
    s3: &Client,
    config: &WorkerConfig,
    checksum: &SourceChecksum,
) -> Result<(), BoxError> {
    let conflict_replay_id = "replay-golden-conflict";
    let conflict_key =
        common::put_conflicting_artifact(s3, config, conflict_replay_id, checksum).await;
    assert_eq!(
        conflict_key,
        artifact_key(&config.artifact_prefix, conflict_replay_id, checksum)?,
        "conflict pre-seed must land on the deterministic artifact key"
    );
    common::publish_job(
        channel,
        &config.job_queue,
        "job-golden-conflict",
        conflict_replay_id,
        GOLDEN_OBJECT_KEY,
        checksum,
    )
    .await;
    let failed = common::wait_for_failed(channel).await;
    assert_eq!(failed.failure.error_code.as_str(), "output.artifact_conflict");
    assert_eq!(failed.failure.stage, ParseStage::Output);
    common::assert_queue_empty(channel, &config.job_queue).await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires Docker; boots ephemeral RabbitMQ + MinIO via testcontainers"]
async fn golden_container_e2e_should_pin_full_worker_contract_byte_for_byte()
-> Result<(), BoxError> {
    // Skip-guard: missing seed fixture must skip cleanly, never false-fail.
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SEED_FIXTURE);
    if !fixture_path.is_file() {
        eprintln!("SKIP golden_container_e2e: seed fixture absent at {}", fixture_path.display());
        return Ok(());
    }
    let seed_bytes = std::fs::read(&fixture_path)?;

    // Skip-guard: MinIO needs static credentials in the process env (the AWS SDK reads
    // its standard chain for BOTH this test's client AND the worker's own client). We do
    // NOT mutate the environment here because the workspace forbids `unsafe`, and
    // `std::env::set_var` is `unsafe` on edition 2024. The runner / CI job exports
    // AWS_ACCESS_KEY_ID=minioadmin and AWS_SECRET_ACCESS_KEY=minioadmin (MinIO defaults,
    // resolved from testcontainers-modules 0.15.0). If absent, skip cleanly.
    if std::env::var_os("AWS_ACCESS_KEY_ID").is_none()
        || std::env::var_os("AWS_SECRET_ACCESS_KEY").is_none()
    {
        eprintln!(
            "SKIP golden_container_e2e: set AWS_ACCESS_KEY_ID=minioadmin and \
             AWS_SECRET_ACCESS_KEY=minioadmin before running this e2e"
        );
        return Ok(());
    }

    // Skip-guard: no Docker daemon → skip cleanly.
    let Some(infra) = boot_infra().await? else {
        return Ok(());
    };
    let config = build_config(&infra)?;

    let s3 = common::s3_client(&config).await;
    common::ensure_bucket(&s3, &config.s3_bucket).await;
    common::put_raw_object(&s3, &config.s3_bucket, GOLDEN_OBJECT_KEY, &seed_bytes).await;

    let amqp =
        lapin::Connection::connect(&config.amqp_url, lapin::ConnectionProperties::default()).await?;
    let channel = amqp.create_channel().await?;
    common::prepare_broker(&channel, &config).await;

    let shutdown = CancellationToken::new();
    let worker = common::spawn_worker(config.clone(), shutdown.clone());

    let checksum = SourceChecksum::sha256(GOLDEN_SOURCE_CHECKSUM_HEX)?;

    let expected_key = assert_success_contract(&channel, &s3, &config, &checksum).await?;
    assert_idempotency(&channel, &s3, &config, &checksum, &expected_key).await;
    assert_checksum_mismatch_failure(&channel, &config).await?;
    assert_artifact_conflict_failure(&channel, &s3, &config, &checksum).await?;

    common::stop_worker(&shutdown, worker).await;
    Ok(())
}
