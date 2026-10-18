mod ankerau;
mod ankerca;
mod ankercom;
mod ankerde;
mod ankereu;
mod ankerfr;
mod ankeritalycom;
mod ankerjapancom;
mod ankerkr;
mod ankernordicscom;
mod ankernz;
mod ankerpl;
mod ankeruk;
mod ankervn;

use bodul_shared::link::LinkKind;

/// Classifies Anker storefront URL paths. Most Anker stores use Shopify-style
/// plural paths; Korea also uses singular `/product/` paths.
pub fn anker_from_location(url: &str) -> LinkKind {
    let path = url.to_lowercase();
    if path.contains("/products/") || path.contains("/product/") {
        LinkKind::Product
    } else if path.contains("/collections/") || path.contains("/collection/") {
        LinkKind::Catalog
    } else if path.contains("/pages/") || path.contains("/blogs/") || path.contains("/blog/") {
        LinkKind::Content
    } else {
        LinkKind::Unknown
    }
}

mod registry;
mod shared;

pub mod prelude;
pub mod shopify;

#[cfg(test)]
mod tests {
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_anker");
    }
}
