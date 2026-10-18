//! Records an offer for every product URL of a consolidated sitemap.

pub mod prelude;

mod command;
mod entity;
mod event;
mod handler;
#[cfg(test)]
mod postgres_tests;
mod reacts_to;
mod store;
