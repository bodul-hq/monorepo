//! Retailer sourcing v2 application steps: one module per step of an initial
//! offer fill.

pub(crate) mod acquire_storefront_token;
pub(crate) mod offer_details;
pub(crate) mod product_catalog_ids;

#[cfg(test)]
mod live_shopify_tests;
#[cfg(test)]
mod postgres_tests;
#[cfg(test)]
mod tests;
