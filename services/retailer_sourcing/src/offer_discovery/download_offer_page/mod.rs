//! Downloads an offer's page and stores it, unless the same URL was downloaded
//! in the last few hours.

pub mod prelude;

mod command;
mod event;
mod handler;
mod reacts_to;
mod store;
