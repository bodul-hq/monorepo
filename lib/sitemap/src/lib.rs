//! Sitemaps: parsing sitemap XML and assembling the files of one crawl into a
//! single tree.

mod build;
mod document;
mod file_name;
mod links;
mod parse;

pub use build::{BuildError, build_tree, build_tree_with};
pub use document::{
    ChangeFrequency, RawSitemapDocument, SitemapDocument, SitemapImage, SitemapUrl,
};
pub use file_name::{FileNameError, file_name};
pub use links::GroupedLinks;
pub use parse::{ChildRef, Parsed, SitemapParseError, parse};

/// How deep sitemap indexes may nest before a branch is cut off.
pub const MAX_SITEMAP_DEPTH: usize = 32;

/// The `source` given to URLs that don't come from a known sitemap file.
pub const ROOT_SITEMAP_SOURCE: &str = "main";
