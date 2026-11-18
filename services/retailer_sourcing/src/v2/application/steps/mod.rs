//! Retailer sourcing v2 application steps: one module per step of an initial
//! offer fill.

pub(crate) mod acquire_storefront_token;
pub(crate) mod extract_offer_details;
pub(crate) mod offer_details;
pub(crate) mod prepare_offer_imports;
pub(crate) mod product_catalog_ids;
pub(crate) mod structuralize_offer_details;

#[cfg(test)]
mod live_shopify_tests;
#[cfg(test)]
mod postgres_tests;
#[cfg(test)]
mod tests;
