//! Command and event bus on top of the Mulac kernel.
//!
//! Features register handlers and reactions here and never touch Mulac
//! directly. Mulac stores every command and event in Postgres, delivers each one
//! at least once, and retries a failed command with backoff.
//!
//! Nothing processes the queues in the background. The HTTP routes delegate
//! bounded batches to Mulac's command and event adapters.

use std::fmt::Display;
use std::sync::Arc;
use std::time::Instant;

use commanding_api::io::CommandingService;
use eventing_api::io::EventingService;
use kernel::io::{CommandGateway, NewCommand, NewCommandEnvelope, NewCommandMetadata};
use kernel::{
    ApplicationEvent, CommandError, CommandHandlerPort, EventError, EventSubscriberPort,
    KernelBuilder, KernelConfig, KernelError, NewEventEnvelope, PersistentKernelHandle,
};
use mulac_axum::io::{CommandingAdapter, EventingAdapter};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use super::db::DbPool;
use super::logging::TIMING_TARGET;

/// A request to change state, handled by exactly one handler.
pub trait Command: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// The stored command type. Changing it orphans queued commands.
    const NAME: &'static str;
}

/// A fact that something happened, which any number of reactions may follow.
pub trait Event: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// The stored event type. Changing it orphans queued events.
    const NAME: &'static str;
}

/// What a handler returns for the bus to store as an event: any [`Event`], or
/// an enum of events when a handler can produce more than one kind. The enum
/// must serialize as its variant's event alone (`#[serde(untagged)]`), so
/// reactions read it as that event.
pub trait Publish: Serialize + Send + Sync + 'static {
    fn event_type(&self) -> &'static str;
}

impl<E: Event> Publish for E {
    fn event_type(&self) -> &'static str {
        E::NAME
    }
}

/// Handles one command type and returns the events it produced. Handlers
/// don't publish events themselves.
pub trait Handle: Send + Sync + 'static {
    type Command: Command;
    type Event: Publish;
    type Error: Display;

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}

type Registration = Box<dyn FnOnce(KernelBuilder) -> KernelBuilder>;

/// Collects handlers and reactions until [`Bus::start`] runs them.
#[derive(Default)]
pub struct Bus {
    registrations: Vec<Registration>,
}

impl Bus {
    /// Registers the handler for its command type.
    pub fn handle<H: Handle>(&mut self, handler: H) {
        let port: Arc<dyn CommandHandlerPort<H::Command, Published<H::Event>>> =
            Arc::new(HandlerPort(handler));
        self.registrations.push(Box::new(move |kernel| {
            kernel.command_handler(<H::Command as Command>::NAME, port)
        }));
    }

    /// Registers a reaction that turns each `E` event into a `C` command.
    pub fn on<E, C>(&mut self, reaction: fn(&E) -> Option<C>)
    where
        E: Event,
        C: Command,
    {
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber_with_command_gateway(E::NAME, C::NAME, move |gateway| {
                Arc::new(Reaction { gateway, reaction }) as Arc<dyn EventSubscriberPort>
            })
        }));
    }

    /// Builds the bus over `pool`. Queued work runs only through the returned
    /// handle.
    pub fn start(self, pool: DbPool) -> Result<BusHandle, KernelError> {
        let kernel = self
            .registrations
            .into_iter()
            .fold(kernel::boot(KernelConfig::default()), |kernel, register| {
                register(kernel)
            });
        let handle = kernel.start_persistent(pool, 0)?;

        Ok(BusHandle {
            dispatcher: Dispatcher {
                gateway: handle.state().command_gateway(),
            },
            commands: CommandingAdapter::new(CommandingService::new(handle.command_consumer())),
            events: EventingAdapter::new(EventingService::new(handle.event_consumer())),
            kernel: Arc::new(handle),
        })
    }
}

/// A started bus: queues commands and processes queued work on request.
#[derive(Clone)]
pub struct BusHandle {
    dispatcher: Dispatcher,
    pub(super) commands: CommandingAdapter,
    pub(super) events: EventingAdapter,
    kernel: Arc<PersistentKernelHandle>,
}

impl BusHandle {
    pub fn dispatcher(&self) -> &Dispatcher {
        &self.dispatcher
    }

    /// Cancel the kernel after the HTTP server has drained in-flight requests.
    pub fn shutdown(&self) {
        self.kernel.shutdown();
    }
}

/// Queues commands from outside the bus, such as from an HTTP request.
#[derive(Clone)]
pub struct Dispatcher {
    gateway: Arc<CommandGateway>,
}

/// A command that couldn't be queued.
#[derive(Debug, thiserror::Error)]
#[error("failed to queue command: {0}")]
pub struct DispatchError(String);

impl Dispatcher {
    /// Stores `command` for the workers to handle. `source` says where it came
    /// from, such as `api`. Blocks on a database insert, so call it off the
    /// async runtime.
    pub fn dispatch<C: Command>(&self, command: &C, source: &str) -> Result<(), DispatchError> {
        let envelope = command_envelope(
            command,
            NewCommandMetadata {
                command_id: Uuid::now_v7(),
                correlation_id: None,
                causation_id: None,
                source: Some(source.to_string()),
            },
        )
        .map_err(|error| DispatchError(error.to_string()))?;

        self.gateway
            .dispatch(envelope)
            .map_err(|error| DispatchError(error.to_string()))
    }
}

/// Serializes an event exactly as the feature defined it, tagged with its name.
#[derive(Serialize)]
#[serde(transparent)]
struct Published<E>(E);

impl<E: Publish> ApplicationEvent for Published<E> {
    fn event_type(&self) -> &'static str {
        self.0.event_type()
    }
}

struct HandlerPort<H>(H);

impl<H: Handle> CommandHandlerPort<H::Command, Published<H::Event>> for HandlerPort<H> {
    fn execute(&self, command: H::Command) -> Result<Vec<Published<H::Event>>, CommandError> {
        let name = <H::Command as Command>::NAME;
        let start = Instant::now();
        let result = self.0.handle(command);
        let elapsed_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(events) => {
                tracing::info!(target: TIMING_TARGET, kind = "command", name, elapsed_ms, status = "ok");
                Ok(events.into_iter().map(Published).collect())
            }
            Err(error) => {
                tracing::info!(target: TIMING_TARGET, kind = "command", name, elapsed_ms, status = "error");
                tracing::error!(command = name, "command failed: {error}");
                // Every failure is reported as a storage error, so Mulac
                // retries all of them. Splitting retryable from permanent
                // errors is a follow-up.
                Err(CommandError::Storage(error.to_string()))
            }
        }
    }
}

struct Reaction<E, C> {
    gateway: Arc<CommandGateway>,
    reaction: fn(&E) -> Option<C>,
}

impl<E, C> EventSubscriberPort for Reaction<E, C>
where
    E: Event,
    C: Command,
{
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let start = Instant::now();
        let result = self.react(envelope);
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let status = if result.is_ok() { "ok" } else { "error" };
        tracing::info!(
            target: TIMING_TARGET,
            kind = "event",
            name = E::NAME,
            reaction = C::NAME,
            elapsed_ms,
            status
        );
        if let Err(error) = &result {
            tracing::error!(
                event = E::NAME,
                reaction = C::NAME,
                "reaction failed: {error}"
            );
        }
        result
    }
}

impl<E, C> Reaction<E, C>
where
    E: Event,
    C: Command,
{
    fn react(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let event: E = serde_json::from_str(&envelope.payload).map_err(subscriber_error)?;
        let Some(command) = (self.reaction)(&event) else {
            return Ok(());
        };

        let metadata = envelope.metadata.as_ref();
        let command = command_envelope(
            &command,
            NewCommandMetadata {
                command_id: Uuid::now_v7(),
                correlation_id: metadata.and_then(|metadata| metadata.correlation_id),
                causation_id: metadata.map(|metadata| metadata.event_id),
                source: Some(format!("event:{}", E::NAME)),
            },
        )
        .map_err(subscriber_error)?;

        self.gateway.dispatch(command).map_err(subscriber_error)
    }
}

fn command_envelope<C: Command>(
    command: &C,
    metadata: NewCommandMetadata,
) -> Result<NewCommandEnvelope, serde_json::Error> {
    Ok(NewCommandEnvelope {
        command: NewCommand {
            command_type: C::NAME.to_string(),
            payload: serde_json::to_string(command)?,
        },
        metadata: Some(metadata),
    })
}

fn subscriber_error(error: impl Display) -> EventError {
    EventError::SubscriberExecution(error.to_string())
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Ping {
        id: u32,
    }

    impl Command for Ping {
        const NAME: &'static str = "Ping";
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Ponged {
        id: u32,
    }

    impl Event for Ponged {
        const NAME: &'static str = "Ponged";
    }

    struct Echo;

    impl Handle for Echo {
        type Command = Ping;
        type Event = Ponged;
        type Error = String;

        fn handle(&self, command: Ping) -> Result<Vec<Ponged>, String> {
            if command.id == 0 {
                return Err("id must not be zero".to_string());
            }
            Ok(vec![Ponged { id: command.id }])
        }
    }

    #[test]
    fn publishes_events_as_the_feature_serializes_them() {
        let events = HandlerPort(Echo).execute(Ping { id: 7 }).unwrap();

        assert_eq!(events[0].event_type(), "Ponged");
        assert_eq!(serde_json::to_string(&events[0]).unwrap(), r#"{"id":7}"#);
    }

    #[test]
    fn reports_handler_failures_as_storage_errors() {
        let error = HandlerPort(Echo).execute(Ping { id: 0 }).err().unwrap();

        assert!(
            matches!(&error, CommandError::Storage(message) if message == "id must not be zero"),
            "{error:?}"
        );
    }
}
