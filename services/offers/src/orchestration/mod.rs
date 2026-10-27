//! Queue endpoints: inbox, command, event and outbox batches delegate to
//! Mulac's Poem adapter. The outbox publisher stays application-owned.

pub mod commanding_api;
pub mod customer_facing_publisher;
pub mod eventing_api;
pub mod inbox_api;
pub mod outbox_api;

pub use commanding_api::CommandingApi;
pub use eventing_api::EventingApi;
pub use inbox_api::InboxApi;
pub use outbox_api::OutboxApi;
