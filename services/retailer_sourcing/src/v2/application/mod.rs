//! Retailer sourcing v2 application layer: use cases that coordinate the
//! domain through the ports that infrastructure implements.
//!
//! Only [`prelude`] is public; other contexts reach this layer through it.

mod model;
mod service;
mod steps;

pub mod prelude;
