//! Monitoring.
//!
//! Watches the pipeline's health: queued and failed commands and events, and
//! how each step is doing.
//!
//! It has no slices yet. Other contexts may only use [`prelude`].

pub mod prelude;
