//! Retailer sourcing v2.
//!
//! Layered as domain, application and infrastructure. The domain holds the
//! business rules and has no dependencies on the other two layers; the
//! application orchestrates the domain through use cases; the infrastructure
//! adapts the outside world (database, object storage) to them; the ui
//! exposes the use cases over HTTP. `incoming` holds sourcing's own policy over
//! types received from the retailer libraries.

mod application;
mod domain;
mod incoming;
mod infrastructure;
pub mod prelude;
mod ui;
