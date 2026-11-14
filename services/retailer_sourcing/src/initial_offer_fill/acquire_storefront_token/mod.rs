//! Acquire a storefront token.
//!
//! The `AcquireStorefrontToken` command downloads the storefront HTML and
//! extracts its token, then emits `StorefrontTokenAcquired` with that token.

pub mod prelude;

pub(super) mod command;
mod entity;
pub(super) mod event;
pub(super) mod handler;
