//! Queue endpoints: inbox, command, event and outbox batches delegate to
//! Mulac's Poem adapter. The outbox publisher stays application-owned.

pub mod bus_api;
pub mod customer_facing_publisher;
pub mod inbox_api;

pub use bus_api::{CommandingApi, EventingApi, OutboxApi};
pub use inbox_api::InboxApi;
