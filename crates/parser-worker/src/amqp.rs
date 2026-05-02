//! RabbitMQ adapter for parse-job consumption and parse-result publication.

use lapin::{
    BasicProperties, Channel, Confirmation, Connection, ConnectionProperties, Consumer,
    message::BasicReturnMessage,
    options::{BasicConsumeOptions, BasicPublishOptions, BasicQosOptions, ConfirmSelectOptions},
    types::FieldTable,
};
use parser_contract::worker::{ParseCompletedMessage, ParseFailedMessage};

use crate::{
    config::WorkerConfig,
    error::{WorkerError, WorkerFailureKind},
};

const RESULT_CONTENT_TYPE: &str = "application/json";

/// RabbitMQ client with separate channels for consuming jobs and publishing results.
#[derive(Debug)]
pub struct RabbitMqClient {
    _connection: Connection,
    consume_channel: Channel,
    publish_channel: Channel,
    consumer: Consumer,
    config: WorkerConfig,
}

impl RabbitMqClient {
    /// Connects to RabbitMQ, configures manual consumption, and enables publisher confirms.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] when connection, channel, QoS, consumer, or confirm setup fails.
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
    pub fn consumer(&self) -> &Consumer {
        &self.consumer
    }

    /// Returns the channel used for manual delivery acknowledgements.
    #[must_use]
    pub fn consume_channel(&self) -> &Channel {
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

/// Serialized result publication ready for RabbitMQ.
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
    prepare_result_publish(
        config,
        config.completed_routing_key.clone(),
        serde_json::to_vec(message)?,
    )
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
    prepare_result_publish(config, config.failed_routing_key.clone(), serde_json::to_vec(message)?)
}

fn prepare_result_publish(
    config: &WorkerConfig,
    routing_key: String,
    body: Vec<u8>,
) -> Result<PreparedResultPublish, WorkerError> {
    Ok(PreparedResultPublish {
        exchange: config.result_exchange.clone(),
        routing_key,
        body,
        content_type: RESULT_CONTENT_TYPE,
        mandatory: true,
    })
}

/// Validates that RabbitMQ accepted a published result and did not return a mandatory message.
///
/// # Errors
///
/// Returns [`WorkerError`] with `output.rabbitmq_publish` when the broker nacks, returns, or
/// cannot provide publisher confirmation.
pub fn ensure_publish_confirmed(confirmation: Confirmation) -> Result<(), WorkerError> {
    match confirmation {
        Confirmation::Ack(None) => Ok(()),
        Confirmation::Ack(Some(returned)) => Err(returned_message_error("ack", &returned)),
        Confirmation::Nack(returned) => Err(match returned {
            Some(returned) => returned_message_error("nack", &returned),
            None => rabbitmq_publish_error("broker nacked result publish".to_owned()),
        }),
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

fn rabbitmq_publish_error(message: String) -> WorkerError {
    WorkerError::Failure(WorkerFailureKind::RabbitMqPublish { message })
}
