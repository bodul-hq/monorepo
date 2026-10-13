use bodul_shared::link::LinkKind;

/// Classifies Shopify storefront URLs by their path conventions, ignoring case.
pub fn shopify_from_location(url: &str) -> LinkKind {
    let path = url.to_lowercase();
    if path.contains("/products/") {
        LinkKind::Product
    } else if path.contains("/collections/") {
        LinkKind::Catalog
    } else if path.contains("/pages/") || path.contains("/blogs/") {
        LinkKind::Content
    } else {
        LinkKind::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_path_ignoring_case() {
        let cases = [
            ("https://shop.example/products/charger", LinkKind::Product),
            ("https://shop.example/Collections/All", LinkKind::Catalog),
            ("https://shop.example/pages/about", LinkKind::Content),
            ("https://shop.example/blogs/news/post", LinkKind::Content),
            ("https://shop.example/cart", LinkKind::Unknown),
        ];
        for (url, kind) in cases {
            assert_eq!(shopify_from_location(url), kind, "{url}");
        }
    }
}
