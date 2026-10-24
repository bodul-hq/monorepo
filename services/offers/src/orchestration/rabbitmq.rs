//! RabbitMQ configuration for the durable outbox. Publication itself is
//! Mulac's `ConfirmedAmqpPublisher`: confirmed, mandatory and persistent.

use std::{env, io};

use outbox::io::ConfirmedAmqpPublisher;

/// Builds the outbox publisher from `AMQP_URL` and `OUTBOX_EXCHANGE`.
///
/// Returns `None` when `AMQP_URL` is unset. `OUTBOX_EXCHANGE` defaults to the
/// empty (default) exchange.
pub fn publisher_from_env() -> io::Result<Option<ConfirmedAmqpPublisher>> {
    let url = match env::var("AMQP_URL") {
        Ok(url) => url,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(_) => return Err(err_configuration()),
    };
    let exchange = match env::var("OUTBOX_EXCHANGE") {
        Ok(exchange) => exchange,
        Err(env::VarError::NotPresent) => String::new(),
        Err(_) => return Err(err_configuration()),
    };
    publisher(url, exchange).map(Some)
}

/// Builds the outbox publisher after checking that `url` is an `amqp` or
/// `amqps` URL with a host. Mulac's publisher leaves this policy to us.
pub fn publisher(url: String, exchange: String) -> io::Result<ConfirmedAmqpPublisher> {
    let parsed = url::Url::parse(&url).map_err(|_| err_configuration())?;
    if !matches!(parsed.scheme(), "amqp" | "amqps") || parsed.host_str().is_none() {
        return Err(err_configuration());
    }
    Ok(ConfirmedAmqpPublisher::new(url, exchange))
}

fn err_configuration() -> io::Error {
    io::Error::other(
        "AMQP_URL must be an amqp(s) URL with a host; RabbitMQ configuration must be valid Unicode",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use outbox::io::{OutboundMessageEnvelope, OutboxEntryMetadata, OutboxPublisherPort};
    use std::{ffi::OsStr, sync::Arc};
    use uuid::Uuid;

    const CONFIGURATION_ERROR: &str =
        "AMQP_URL must be an amqp(s) URL with a host; RabbitMQ configuration must be valid Unicode";
    const FROM_ENV_PROBE: &str = "OFFERS_RABBITMQ_FROM_ENV_PROBE";

    /// Runs `publisher_from_env` in a child test process so that tests never
    /// mutate this process's environment.
    fn from_env_in_child(vars: &[(&str, &OsStr)]) -> String {
        let mut command = std::process::Command::new(env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "orchestration::rabbitmq::tests::from_env_probe",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(FROM_ENV_PROBE, "1")
            .env_remove("AMQP_URL")
            .env_remove("OUTBOX_EXCHANGE");
        for (name, value) in vars {
            command.env(name, value);
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.split_once("FROM_ENV:").map(|(_, result)| result))
            .unwrap_or_else(|| panic!("probe printed no result: {output:?}"))
            .to_owned()
    }

    #[test]
    fn from_env_probe() {
        if env::var_os(FROM_ENV_PROBE).is_none() {
            return;
        }
        match publisher_from_env() {
            Ok(None) => println!("FROM_ENV:none"),
            Ok(Some(publisher)) => println!("FROM_ENV:exchange={}", publisher.exchange()),
            Err(err) => println!("FROM_ENV:error={err}"),
        }
    }

    #[test]
    fn from_env_reads_url_and_exchange_and_validates_them() {
        let config_error = format!("error={CONFIGURATION_ERROR}");
        assert_eq!(from_env_in_child(&[]), "none");
        assert_eq!(
            from_env_in_child(&[("OUTBOX_EXCHANGE", OsStr::new("offers"))]),
            "none"
        );
        assert_eq!(
            from_env_in_child(&[("AMQP_URL", OsStr::new("amqp://localhost"))]),
            "exchange="
        );
        assert_eq!(
            from_env_in_child(&[
                (
                    "AMQP_URL",
                    OsStr::new("amqps://user:secret@broker:5671/%2f")
                ),
                ("OUTBOX_EXCHANGE", OsStr::new("offers")),
            ]),
            "exchange=offers"
        );
        for url in ["", "https://example.com", "not a url", "amqp:no-host"] {
            assert_eq!(
                from_env_in_child(&[("AMQP_URL", OsStr::new(url))]),
                config_error,
                "{url:?}"
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let invalid = OsStr::from_bytes(&[0xff]);
            assert_eq!(from_env_in_child(&[("AMQP_URL", invalid)]), config_error);
            assert_eq!(
                from_env_in_child(&[
                    ("AMQP_URL", OsStr::new("amqp://localhost")),
                    ("OUTBOX_EXCHANGE", invalid),
                ]),
                config_error
            );
        }
    }

    #[test]
    fn accepts_only_amqp_and_amqps_urls_with_a_host() {
        for url in [
            "amqp://localhost",
            "amqp://offers:secret@127.0.0.1:5672/%2f",
            "amqps://broker.example.com",
        ] {
            let publisher = publisher(url.into(), "offers".into()).unwrap();
            assert_eq!(publisher.exchange(), "offers", "{url}");
            assert_eq!(
                publisher.timeout(),
                std::time::Duration::from_secs(10),
                "{url}"
            );
        }
        for url in ["", "https://example.com", "not a url", "amqp:no-host"] {
            let err = publisher(url.into(), String::new()).err().unwrap();
            assert_eq!(err.to_string(), CONFIGURATION_ERROR, "{url:?}");
        }
    }

    fn envelope(routing_key: String) -> OutboundMessageEnvelope {
        let event_id = Uuid::new_v4();
        OutboundMessageEnvelope {
            payload: br#"{"title":"Charger"}"#.to_vec(),
            metadata: OutboxEntryMetadata {
                event_id,
                message_id: event_id,
                correlation_id: Some(Uuid::new_v4()),
                causation_id: Some(Uuid::new_v4()),
                event_type: "OfferCreated".into(),
                routing_key,
                source: Some("offers".into()),
                content_type: None,
            },
        }
    }

    /// Smoke test for `make test-rabbitmq`; Mulac's own broker test covers
    /// the publisher in detail.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "requires AMQP_URL_TEST"]
    async fn rabbitmq_confirms_delivery_rejects_unroutable_and_recovers() {
        use lapin::{
            Connection, ConnectionProperties,
            options::{BasicGetOptions, QueueDeclareOptions, QueueDeleteOptions},
            types::FieldTable,
        };
        let url = env::var("AMQP_URL_TEST").expect("AMQP_URL_TEST is required");
        let connection = Connection::connect(&url, ConnectionProperties::default())
            .await
            .unwrap();
        let channel = connection.create_channel().await.unwrap();
        let queue = channel
            .queue_declare(
                "".into(),
                QueueDeclareOptions {
                    exclusive: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .unwrap();
        let publisher = Arc::new(publisher(url, String::new()).unwrap());
        // Publish on the blocking pool, as the outbox endpoint does.
        let publish = |message: OutboundMessageEnvelope| {
            let publisher = publisher.clone();
            tokio::task::spawn_blocking(move || publisher.publish(message))
        };
        let message = envelope(queue.name().to_string());

        publish(message.clone()).await.unwrap().unwrap();
        let received = channel
            .basic_get(queue.name().clone(), BasicGetOptions { no_ack: true })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(received.data, message.payload);
        assert_eq!(*received.properties.delivery_mode(), Some(2));
        assert_eq!(
            received
                .properties
                .content_type()
                .as_ref()
                .unwrap()
                .as_str(),
            "application/json"
        );

        let mut unroutable = message.clone();
        unroutable.metadata.routing_key = format!("offers-missing-{}", Uuid::new_v4());
        assert_eq!(
            publish(unroutable).await.unwrap().unwrap_err().to_string(),
            "routing error: RabbitMQ returned an unroutable message"
        );

        publish(message).await.unwrap().unwrap();
        assert!(
            channel
                .basic_get(queue.name().clone(), BasicGetOptions { no_ack: true })
                .await
                .unwrap()
                .is_some()
        );
        channel
            .queue_delete(queue.name().clone(), QueueDeleteOptions::default())
            .await
            .unwrap();
        connection.close(200, "test complete".into()).await.unwrap();
    }
}
