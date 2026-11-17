//! Retailer sourcing v2 infrastructure: adapters for the database and
//! object storage that the application layer depends on.
//!
//! Only [`prelude`] is public; other contexts reach this layer through it.

pub mod prelude;

mod storefront_token;

mod offer_http;
