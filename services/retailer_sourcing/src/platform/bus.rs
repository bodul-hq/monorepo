//! Command and event bus on top of the Mulac kernel.
//!
//! Features register handlers and reactions here and never touch Mulac
//! directly. Mulac stores every command and event in Postgres, delivers each one
//! at least once, and retries a failed command with backoff.

use std::fmt::Display;
use std::sync::Arc;

use kernel::io::{
    CommandGateway, NewCommand, NewCommandEnvelope, NewCommandMetadata, run_command_worker,
    run_event_worker,
};
use kernel::{
    ApplicationEvent, CommandError, CommandHandlerPort, EventError, EventSubscriberPort,
    KernelBuilder, KernelConfig, KernelError, NewEventEnvelope, PersistentKernelHandle,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::db::DbPool;

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

/// Handles one command type and returns the events it produced. Handlers
/// don't publish events themselves.
pub trait Handle: Send + Sync + 'static {
    type Command: Command;
    type Event: Event;
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

    /// Starts the command and event workers.
    pub fn start(self, pool: DbPool) -> Result<Running, KernelError> {
        let kernel = self
            .registrations
            .into_iter()
            .fold(kernel::boot(KernelConfig::default()), |kernel, register| {
                register(kernel)
            });
        let handle = kernel.start_persistent(pool, 0)?;
        let workers = vec![
            tokio::spawn(run_command_worker(
                handle.command_consumer(),
                handle.child_token(),
            )),
            tokio::spawn(run_event_worker(
                handle.event_consumer(),
                handle.child_token(),
            )),
        ];

        Ok(Running { handle, workers })
    }
}

/// The running workers.
pub struct Running {
    handle: PersistentKernelHandle,
    workers: Vec<JoinHandle<()>>,
}

impl Running {
    /// Stops polling for work and waits for the batch in progress to finish.
    pub async fn stop(self) {
        self.handle.shutdown();
        for worker in self.workers {
            if let Err(error) = worker.await {
                tracing::error!("bus worker failed: {error}");
            }
        }
    }
}

/// Serializes an event exactly as the feature defined it, tagged with its name.
#[derive(Serialize)]
#[serde(transparent)]
struct Published<E>(E);

impl<E: Event> ApplicationEvent for Published<E> {
    fn event_type(&self) -> &'static str {
        E::NAME
    }
}

struct HandlerPort<H>(H);

impl<H: Handle> CommandHandlerPort<H::Command, Published<H::Event>> for HandlerPort<H> {
    fn execute(&self, command: H::Command) -> Result<Vec<Published<H::Event>>, CommandError> {
        // Every failure is reported as a storage error, so Mulac retries all of
        // them. Splitting retryable from permanent errors is a follow-up.
        self.0
            .handle(command)
            .map(|events| events.into_iter().map(Published).collect())
            .map_err(|error| CommandError::Storage(error.to_string()))
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
