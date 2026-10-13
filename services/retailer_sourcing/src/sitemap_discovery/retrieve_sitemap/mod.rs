//! Fetches a retailer's root sitemaps and every child sitemap they list, and
//! stores the files as fetched.

pub mod prelude;

mod command;
mod crawl;
mod event;
mod handler;
mod store;
