//! `RabbitMQ` adapter for parse-job consumption and parse-result publication.

use std::pin::Pin;

use lapin::{
    BasicProperties, Channel, Confirmation, Connection, ConnectionProperties, Consumer,
    message::{BasicReturnMessage, Delivery},
    options::{BasicConsumeOptions, BasicPublishOptions, BasicQosOptions, ConfirmSelectOptions},
    types::FieldTable,
};
use parser_contract::worker::{ParseCompletedMessage, ParseFailedMessage};

use crate::{
    config::WorkerConfig,
    error::{WorkerError, WorkerFailureKind},
};

const RESULT_CONTENT_TYPE: &str = "application/json";

/// Delivery acknowledgement decision after the worker outcome path is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryAction {
    /// Acknowledge the input parse job.
    Ack,
    /// Requeue the input parse job because no durable outcome was published.
    NackRequeue,
}

/// Result kind whose publish confirmation completed successfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishedOutcome {
    /// A confirmed `parse.completed` outcome.
    Completed,
    /// A confirmed `parse.failed` outcome, including malformed jobs represented as failures.
    Failed,
}

/// Maps outcome publication to the AMQP delivery action.
#[must_use]
pub fn delivery_action_after_publish(
    published: Result<PublishedOutcome, WorkerError>,
) -> DeliveryAction {
    match published {
        Ok(PublishedOutcome::Completed | PublishedOutcome::Failed) => DeliveryAction::Ack,
        Err(_error) => DeliveryAction::NackRequeue,
    }
}

/// Boxed acknowledgement future used by testable acker adapters.
pub type AckerFuture<'a> = Pin<Box<dyn Future<Output = Result<(), WorkerError>> + Send + 'a>>;

/// Minimal acknowledgement interface used to test ack policy without a live broker.
pub trait DeliveryAcker: Send {
    /// Acknowledge a delivery.
    fn ack(&mut self, options: lapin::options::BasicAckOptions) -> AckerFuture<'_>;

    /// Negatively acknowledge and optionally requeue a delivery.
    fn nack(&mut self, options: lapin::options::BasicNackOptions) -> AckerFuture<'_>;
}

/// `lapin` delivery-backed acker adapter.
#[derive(Debug)]
pub struct LapinDeliveryAcker<'a> {
    delivery: &'a Delivery,
}

impl<'a> LapinDeliveryAcker<'a> {
    /// Builds an acker around a `lapin` delivery.
    #[must_use]
    pub const fn new(delivery: &'a Delivery) -> Self {
        Self { delivery }
    }
}

impl DeliveryAcker for LapinDeliveryAcker<'_> {
    fn ack(&mut self, options: lapin::options::BasicAckOptions) -> AckerFuture<'_> {
        Box::pin(async move {
            _ = self.delivery.ack(options).await?;
            Ok(())
        })
    }

    fn nack(&mut self, options: lapin::options::BasicNackOptions) -> AckerFuture<'_> {
        Box::pin(async move {
            _ = self.delivery.nack(options).await?;
            Ok(())
        })
    }
}

/// Applies a delivery action through a testable acker adapter.
///
/// # Errors
///
/// Returns [`WorkerError`] when `RabbitMQ` rejects the ack/nack operation.
pub async fn apply_delivery_action(
    acker: &mut impl DeliveryAcker,
    action: DeliveryAction,
) -> Result<(), WorkerError> {
    match action {
        DeliveryAction::Ack => acker.ack(lapin::options::BasicAckOptions::default()).await,
        DeliveryAction::NackRequeue => {
            acker.nack(lapin::options::BasicNackOptions { multiple: false, requeue: true }).await
        }
    }
}

/// Applies a delivery action to a `lapin` delivery.
///
/// # Errors
///
/// Returns [`WorkerError`] when `RabbitMQ` rejects the ack/nack operation.
pub async fn apply_lapin_delivery_action(
    delivery: &Delivery,
    action: DeliveryAction,
) -> Result<(), WorkerError> {
    let mut acker = LapinDeliveryAcker::new(delivery);
    apply_delivery_action(&mut acker, action).await
}

/// `RabbitMQ` client with separate channels for consuming jobs and publishing results.
#[derive(Debug)]
pub struct RabbitMqClient {
    _connection: Connection,
    consume_channel: Channel,
    publish_channel: Channel,
    consumer: Consumer,
    config: WorkerConfig,
}

impl RabbitMqClient {
    /// Connects to `RabbitMQ`, configures manual consumption, and enables publisher confirms.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] when connection, channel, `QoS`, consumer, or confirm setup fails.
    pub async fn connect(config: &WorkerConfig) -> Result<Self, WorkerError> {
        config.validate()?;

        let connection =
            Connection::connect(&config.amqp_url, ConnectionProperties::default()).await?;
        let consume_channel = connection.create_channel().await?;
        consume_channel.basic_qos(config.prefetch, BasicQosOptions { global: false }).await?;

        let publish_channel = connection.create_channel().await?;
        publish_channel.confirm_select(ConfirmSelectOptions::default()).await?;

        let consumer = consume_channel
            .basic_consume(
                config.job_queue.clone().into(),
                "replay-parser-2-worker".into(),
                BasicConsumeOptions { no_ack: false, ..Default::default() },
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            _connection: connection,
            consume_channel,
            publish_channel,
            consumer,
            config: config.clone(),
        })
    }

    /// Returns the job consumer stream.
    #[must_use]
    pub const fn consumer(&self) -> &Consumer {
        &self.consumer
    }

    /// Returns the mutable job consumer stream.
    #[must_use]
    pub const fn consumer_mut(&mut self) -> &mut Consumer {
        &mut self.consumer
    }

    /// Returns the channel used for manual delivery acknowledgements.
    #[must_use]
    pub const fn consume_channel(&self) -> &Channel {
        &self.consume_channel
    }

    /// Publishes a confirmed `parse.completed` result.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] if serialization, publishing, or broker confirmation fails.
    pub async fn publish_completed(
        &self,
        message: &ParseCompletedMessage,
    ) -> Result<(), WorkerError> {
        self.publish_prepared(prepare_completed_publish(&self.config, message)?).await
    }

    /// Publishes a confirmed `parse.failed` result.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] if serialization, publishing, or broker confirmation fails.
    pub async fn publish_failed(&self, message: &ParseFailedMessage) -> Result<(), WorkerError> {
        self.publish_prepared(prepare_failed_publish(&self.config, message)?).await
    }

    async fn publish_prepared(&self, publish: PreparedResultPublish) -> Result<(), WorkerError> {
        let confirm = self
            .publish_channel
            .basic_publish(
                publish.exchange.into(),
                publish.routing_key.into(),
                BasicPublishOptions { mandatory: true, ..Default::default() },
                &publish.body,
                BasicProperties::default().with_content_type(publish.content_type.into()),
            )
            .await
            .map_err(|source| rabbitmq_publish_error(source.to_string()))?;

        let confirmation =
            confirm.await.map_err(|source| rabbitmq_publish_error(source.to_string()))?;
        ensure_publish_confirmed(confirmation)
    }
}

/// Serialized result publication ready for `RabbitMQ`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedResultPublish {
    /// Result exchange name.
    pub exchange: String,
    /// Result routing key.
    pub routing_key: String,
    /// Serialized JSON body.
    pub body: Vec<u8>,
    /// AMQP content type.
    pub content_type: &'static str,
    /// Whether the publish requires routing to a queue.
    pub mandatory: bool,
}

/// Builds a publish request for a `parse.completed` result.
///
/// # Errors
///
/// Returns [`WorkerError`] when JSON serialization fails.
pub fn prepare_completed_publish(
    config: &WorkerConfig,
    message: &ParseCompletedMessage,
) -> Result<PreparedResultPublish, WorkerError> {
    Ok(prepare_result_publish(
        config,
        config.completed_routing_key.clone(),
        serde_json::to_vec(message)?,
    ))
}

/// Builds a publish request for a `parse.failed` result.
///
/// # Errors
///
/// Returns [`WorkerError`] when JSON serialization fails.
pub fn prepare_failed_publish(
    config: &WorkerConfig,
    message: &ParseFailedMessage,
) -> Result<PreparedResultPublish, WorkerError> {
    Ok(prepare_result_publish(
        config,
        config.failed_routing_key.clone(),
        serde_json::to_vec(message)?,
    ))
}

fn prepare_result_publish(
    config: &WorkerConfig,
    routing_key: String,
    body: Vec<u8>,
) -> PreparedResultPublish {
    PreparedResultPublish {
        exchange: config.result_exchange.clone(),
        routing_key,
        body,
        content_type: RESULT_CONTENT_TYPE,
        mandatory: true,
    }
}

/// Validates that `RabbitMQ` accepted a published result and did not return a mandatory message.
///
/// # Errors
///
/// Returns [`WorkerError`] with `output.rabbitmq_publish` when the broker nacks, returns, or
/// cannot provide publisher confirmation.
pub fn ensure_publish_confirmed(confirmation: Confirmation) -> Result<(), WorkerError> {
    match confirmation {
        Confirmation::Ack(None) => Ok(()),
        Confirmation::Ack(Some(returned)) => Err(returned_message_error("ack", &returned)),
        Confirmation::Nack(returned) => Err(returned.map_or_else(
            || rabbitmq_publish_error("broker nacked result publish".to_owned()),
            |returned| returned_message_error("nack", &returned),
        )),
        Confirmation::NotRequested => {
            Err(rabbitmq_publish_error("publisher confirmation was not requested".to_owned()))
        }
    }
}

fn returned_message_error(kind: &str, returned: &BasicReturnMessage) -> WorkerError {
    rabbitmq_publish_error(format!(
        "broker {kind} included returned mandatory message reply_code={} reply_text={} exchange={} routing_key={}",
        returned.reply_code,
        returned.reply_text.as_str(),
        returned.exchange.as_str(),
        returned.routing_key.as_str()
    ))
}

const fn rabbitmq_publish_error(message: String) -> WorkerError {
    WorkerError::Failure(WorkerFailureKind::RabbitMqPublish { message })
}
