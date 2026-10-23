//! # Module Organization
//!
//! - Public modules go at the top
//! - Private modules go below, separated by two empty lines
//! - `prelude` is always public; other modules are private by default

pub mod event_sourced_repository;
pub mod prelude;

mod aggregate;
mod error;
mod event_envelope;
mod event_publisher;
mod event_store;
mod metadata;
mod query_bus;
mod query_handler;
