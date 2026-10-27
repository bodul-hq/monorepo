//! Offer delivery.
//!
//! Hands completed offer exports to other services. One slice so far:
//!
//! 1. [`record_export_messages`] records one outbox message per exported
//!    variation when offer discovery reports a completed export.
//!
//! Delivery itself runs through the outbox publisher on request. Other
//! contexts may only use [`prelude`].

pub mod prelude;

mod record_export_messages;
