//! Command, event and outbox runner endpoints delegate to Mulac's Poem adapters.
//! Subscribers and publishers remain configured by the application.

use mulac_poem::io::{CommandingAdapter, EventingAdapter, OutboxAdapter, RunRequest, RunResponse};
use poem::Result;
use poem_openapi::{OpenApi, payload::Json};

pub struct CommandingApi {
    adapter: CommandingAdapter,
}

impl CommandingApi {
    pub fn new(adapter: CommandingAdapter) -> Self {
        Self { adapter }
    }
}

#[OpenApi]
impl CommandingApi {
    /// Execute one batch of due commands; resulting events remain queued.
    #[oai(path = "/commands/run", method = "post")]
    async fn run_commands(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.adapter.run(request).await
    }
}

pub struct EventingApi {
    adapter: EventingAdapter,
}

impl EventingApi {
    pub fn new(adapter: EventingAdapter) -> Self {
        Self { adapter }
    }
}

#[OpenApi]
impl EventingApi {
    /// Dispatch one batch of due events to the registered projectors.
    #[oai(path = "/events/run", method = "post")]
    async fn run_events(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.adapter.run(request).await
    }
}

pub struct OutboxApi {
    adapter: OutboxAdapter,
}

impl OutboxApi {
    pub fn new(adapter: OutboxAdapter) -> Self {
        Self { adapter }
    }
}

#[OpenApi]
impl OutboxApi {
    /// Publish one batch to customer-facing's inbox; returns 503 when delivery is not configured.
    #[oai(path = "/outbox/run", method = "post")]
    async fn run_outbox(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.adapter.run(request).await
    }
}

#[cfg(test)]
mod commanding_tests;

#[cfg(test)]
mod eventing_tests;

#[cfg(test)]
mod outbox_tests;
