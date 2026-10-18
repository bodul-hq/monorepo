//! Sorts every URL of a processed sitemap by kind of page (product, catalog,
//! content and so on) using the retailer's link rules, and stores the groups.

pub mod prelude;

mod command;
mod event;
mod handler;
mod reacts_to;
mod store;
