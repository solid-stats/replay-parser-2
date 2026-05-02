//! RabbitMQ adapter behavior tests.

use lapin::{
    Confirmation,
    message::{BasicReturnMessage, Delivery},
    options::{BasicAckOptions, BasicNackOptions},
};
use parser_contract::{
    presence::{FieldPresence, UnknownReason},
    source_ref::SourceChecksum,
    version::{ContractVersion, ParserInfo},
    worker::{ArtifactReference, ParseCompletedMessage, ParseFailedMessage},
};
use parser_worker::{
    amqp::{
        AckerFuture, DeliveryAcker, DeliveryAction, PublishedOutcome, apply_delivery_action,
        delivery_action_after_publish, ensure_publish_confirmed, prepare_completed_publish,
        prepare_failed_publish,
    },
    config::{
        DEFAULT_COMPLETED_ROUTING_KEY, DEFAULT_FAILED_ROUTING_KEY, DEFAULT_PREFETCH,
        DEFAULT_RESULT_EXCHANGE, WorkerConfig, WorkerConfigOverrides,
    },
    error::{WorkerError, WorkerFailureKind},
};
use serde_json::{Value, json};

fn default_config() -> WorkerConfig {
    WorkerConfig::from_env_and_overrides(
        |_| None,
        WorkerConfigOverrides {
            s3_bucket: Some("solid-stats-replays".to_owned()),
            ..Default::default()
        },
    )
    .expect("default test config should be valid")
}

fn checksum(byte: char) -> SourceChecksum {
    SourceChecksum::sha256(byte.to_string().repeat(64))
        .expect("test checksum should be valid lowercase SHA-256")
}

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn completed_message() -> ParseCompletedMessage {
    ParseCompletedMessage::new(
        "job-1".to_owned(),
        "replay-1".to_owned(),
        ContractVersion::current(),
        checksum('a'),
        ArtifactReference {
            bucket: "solid-stats-replays".to_owned(),
            key: "artifacts/v3/replay-1/source.json".to_owned(),
        },
        checksum('b'),
        128,
        parser_info(),
    )
}

fn failed_message() -> ParseFailedMessage {
    ParseFailedMessage::unsupported_contract_version(
        FieldPresence::Present { value: "job-1".to_owned(), source: None },
        FieldPresence::Present { value: "replay-1".to_owned(), source: None },
        FieldPresence::Present { value: "raw/replay-1.json".to_owned(), source: None },
        FieldPresence::Unknown { reason: UnknownReason::SchemaDrift, source: None },
        FieldPresence::Present { value: checksum('a'), source: None },
        parser_info(),
    )
    .expect("failed message should be valid")
}

fn decoded_body(body: &[u8]) -> Value {
    serde_json::from_slice(body).expect("publish body should be valid JSON")
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FakeAckCall {
    Ack(BasicAckOptions),
    Nack(BasicNackOptions),
}

#[derive(Debug, Default)]
struct FakeAcker {
    calls: Vec<FakeAckCall>,
}

impl DeliveryAcker for FakeAcker {
    fn ack(&mut self, options: BasicAckOptions) -> AckerFuture<'_> {
        self.calls.push(FakeAckCall::Ack(options));
        Box::pin(async { Ok(()) })
    }

    fn nack(&mut self, options: BasicNackOptions) -> AckerFuture<'_> {
        self.calls.push(FakeAckCall::Nack(options));
        Box::pin(async { Ok(()) })
    }
}

#[test]
fn amqp_completed_publish_uses_default_completed_routing_key() {
    let config = default_config();
    let publish = prepare_completed_publish(&config, &completed_message())
        .expect("completed publish should serialize");

    assert_eq!(publish.exchange, DEFAULT_RESULT_EXCHANGE);
    assert_eq!(publish.routing_key, DEFAULT_COMPLETED_ROUTING_KEY);
    assert_eq!(publish.content_type, "application/json");
    assert!(publish.mandatory);
    assert_eq!(decoded_body(&publish.body)["message_type"], "parse.completed");
}

#[test]
fn amqp_failed_publish_uses_default_failed_routing_key() {
    let config = default_config();
    let publish = prepare_failed_publish(&config, &failed_message())
        .expect("failed publish should serialize");

    assert_eq!(publish.exchange, DEFAULT_RESULT_EXCHANGE);
    assert_eq!(publish.routing_key, DEFAULT_FAILED_ROUTING_KEY);
    assert_eq!(publish.content_type, "application/json");
    assert!(publish.mandatory);
    assert_eq!(decoded_body(&publish.body)["message_type"], "parse.failed");
}

#[test]
fn amqp_non_ack_confirm_returns_rabbitmq_publish_error() {
    let error = ensure_publish_confirmed(Confirmation::Nack(None))
        .expect_err("nack confirmation should fail");

    assert!(error.to_string().contains("output.rabbitmq_publish"));
}

#[test]
fn amqp_returned_mandatory_message_is_publish_failure() {
    let returned = BasicReturnMessage {
        delivery: Delivery::mock(
            1,
            DEFAULT_RESULT_EXCHANGE.into(),
            DEFAULT_COMPLETED_ROUTING_KEY.into(),
            false,
            Vec::new(),
        ),
        reply_code: 312,
        reply_text: "NO_ROUTE".into(),
    };

    let error = ensure_publish_confirmed(Confirmation::Ack(Some(returned)))
        .expect_err("mandatory returned message should fail");

    let message = error.to_string();
    assert!(message.contains("output.rabbitmq_publish"));
    assert!(message.contains("returned mandatory message"));
}

#[test]
fn amqp_worker_config_default_prefetch_is_one() {
    let config = default_config();

    assert_eq!(config.prefetch, DEFAULT_PREFETCH);
    assert_eq!(DEFAULT_PREFETCH, 1);
}

#[test]
fn ack_policy_successful_completed_result_confirm_maps_to_ack() {
    let action = delivery_action_after_publish(Ok(PublishedOutcome::Completed));

    assert_eq!(action, DeliveryAction::Ack);
}

#[test]
fn ack_policy_successful_failed_result_confirm_maps_to_ack() {
    let action = delivery_action_after_publish(Ok(PublishedOutcome::Failed));

    assert_eq!(action, DeliveryAction::Ack);
}

#[test]
fn ack_policy_publish_failure_maps_to_nack_requeue() {
    let publish_error = WorkerError::Failure(WorkerFailureKind::RabbitMqPublish {
        message: "broker nacked result publish".to_owned(),
    });
    let action = delivery_action_after_publish(Err(publish_error));

    assert_eq!(action, DeliveryAction::NackRequeue);
}

#[test]
fn ack_policy_invalid_job_json_waits_for_failed_publish_result() {
    let action_after_confirmed_failed_publish =
        delivery_action_after_publish(Ok(PublishedOutcome::Failed));
    let action_after_failed_failed_publish = delivery_action_after_publish(Err(
        WorkerError::Failure(WorkerFailureKind::RabbitMqPublish {
            message: "could not publish parse.failed for invalid job JSON".to_owned(),
        }),
    ));

    assert_eq!(action_after_confirmed_failed_publish, DeliveryAction::Ack);
    assert_eq!(action_after_failed_failed_publish, DeliveryAction::NackRequeue);
}

#[tokio::test]
async fn ack_policy_ack_uses_basic_ack_options() {
    let mut acker = FakeAcker::default();

    apply_delivery_action(&mut acker, DeliveryAction::Ack).await.expect("ack action should apply");

    assert_eq!(acker.calls, vec![FakeAckCall::Ack(BasicAckOptions::default())]);
}

#[tokio::test]
async fn ack_policy_nack_requeue_uses_basic_nack_options() {
    let mut acker = FakeAcker::default();

    apply_delivery_action(&mut acker, DeliveryAction::NackRequeue)
        .await
        .expect("nack action should apply");

    assert_eq!(
        acker.calls,
        vec![FakeAckCall::Nack(BasicNackOptions { multiple: false, requeue: true })]
    );
}
