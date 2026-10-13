//! Offer discovery.
//!
//! Turns the product URLs of each consolidated sitemap into offers and downloads
//! their pages. Two feature slices run in sequence:
//!
//! 1. [`discover_offers`] records an offer for every product URL of a
//!    consolidated sitemap.
//! 2. [`download_offer_page`] downloads each offer's page, unless it was
//!    fetched in the last few hours.
//!
//! This module holds the domain shared by the slices. Other contexts may only
//! use [`prelude`].

pub mod prelude;

mod schema;

mod discover_offers;
mod download_offer_page;
