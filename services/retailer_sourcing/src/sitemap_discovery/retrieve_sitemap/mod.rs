//! Fetches a retailer's root sitemaps and every child sitemap they list, and
//! stores the files as fetched.

pub mod prelude;

mod api;
mod command;
mod crawl;
mod entity;
mod event;
mod handler;
#[cfg(test)]
mod live_tests;
#[cfg(test)]
mod postgres_tests;
mod store;
