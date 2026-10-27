//! Explicit queue batches. Mulac owns validation, blocking work, and retries.

use commanding_api::io::CommandingService;
use eventing_api::io::EventingService;
use inbox_api::io::InboxService;
use kernel::{KernelConfig, KernelError, PersistentKernelHandle, io::DbPool};
use mulac_poem::io::{
    CommandingAdapter, EventingAdapter, InboxAdapter, InboxSubmission, OutboxAdapter, RunRequest,
    RunResponse, SubmitInboxRequest,
};
use poem::Result;
use poem_openapi::{OpenApi, payload::Json};

#[derive(Clone)]
pub struct AppState {
    pub inbox: InboxAdapter,
    pub commands: CommandingAdapter,
    pub events: EventingAdapter,
    pub outbox: OutboxAdapter,
}

impl AppState {
    pub fn from_kernel(pool: DbPool, kernel: &PersistentKernelHandle) -> Self {
        Self {
            inbox: InboxAdapter::new(InboxService::from_postgres(pool)),
            commands: CommandingAdapter::new(CommandingService::new(kernel.command_consumer())),
            events: EventingAdapter::new(EventingService::new(kernel.event_consumer())),
            outbox: OutboxAdapter::unconfigured(),
        }
    }
}

pub fn start_mulac(pool: DbPool) -> Result<(PersistentKernelHandle, AppState), KernelError> {
    // No production handlers are registered yet. Unknown commands fail visibly
    // through Mulac; accepting an inbox message never pretends to project offers.
    let kernel = kernel::boot(KernelConfig::default()).start_persistent(pool.clone(), 0)?;
    let state = AppState::from_kernel(pool, &kernel);
    Ok((kernel, state))
}

pub struct QueueApi(pub AppState);

#[OpenApi]
impl QueueApi {
    /// Durably store a message without processing it; unknown routing keys are accepted.
    #[oai(path = "/inbox", method = "post")]
    async fn submit(&self, request: Json<SubmitInboxRequest>) -> Result<InboxSubmission> {
        self.0.inbox.submit(request).await
    }

    /// Queue commands from one batch of stored inbox messages.
    #[oai(path = "/inbox/run", method = "post")]
    async fn run_inbox(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.0.inbox.run(request).await
    }

    /// Execute one batch of commands; resulting events remain queued.
    #[oai(path = "/commands/run", method = "post")]
    async fn run_commands(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.0.commands.run(request).await
    }

    /// Dispatch one batch of events to registered subscribers.
    #[oai(path = "/events/run", method = "post")]
    async fn run_events(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.0.events.run(request).await
    }

    /// Delivery is unconfigured; valid requests return 503 without touching the queue.
    #[oai(path = "/outbox/run", method = "post")]
    async fn run_outbox(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.0.outbox.run(request).await
    }
}
