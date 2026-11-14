//! Initial offer fill.
mod discover_markets;

mod acquire_storefront_token;
mod offer_details;
mod product_catalog_ids;

mod api;
pub mod prelude;
mod transport;

#[cfg(test)]
mod live_shopify_tests;
#[cfg(test)]
mod postgres_tests;
#[cfg(test)]
mod tests;
