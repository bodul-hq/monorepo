//! Identifier types used by the retailer modules in this crate. They live in `retailer_sauce`, so
//! storefronts outside this crate build the same ones.

pub use retailer_sauce::identifier::*;

/// Old AU/US crawls also saved pages from other regions. Their App Router JSON-LD is an
/// array and their payload is not readable by these Pages Router integrations. Skip only
/// documents whose product URLs positively identify another known storefront; malformed or
/// missing product data must still fail normalization.
pub(crate) fn is_other_storefront(
    document: &html_extractor::serde_json::Value,
    region: &str,
) -> bool {
    use html_extractor::serde_json::Value;

    if document.get("next_data").is_some() {
        return false;
    }
    let Some(items) = document.get("ld-json").and_then(Value::as_array) else {
        return false;
    };
    let products: Vec<_> = items
        .iter()
        .flat_map(|item| match &item["data"] {
            Value::Array(items) => items.iter().collect::<Vec<_>>(),
            item => vec![item],
        })
        .filter(|item| item["@type"] == "Product")
        .collect();
    !products.is_empty()
        && products.iter().all(|product| {
            product["url"]
                .as_str()
                .and_then(|url| {
                    url.strip_prefix("https://www.anker.com/")
                        .or_else(|| url.strip_prefix("https://anker.com/"))
                })
                .and_then(|path| path.split_once("/products/"))
                .is_some_and(|(locale, handle)| {
                    matches!(locale, "au" | "ca" | "ca-fr" | "de" | "uk" | "mx")
                        && locale != region
                        && !handle.is_empty()
                })
        })
}

#[cfg(test)]
mod tests {
    use super::is_other_storefront;
    use html_extractor::serde_json::json;

    #[test]
    fn only_identifiable_foreign_products_are_skipped() {
        for locale in ["ca", "ca-fr", "de", "uk", "mx"] {
            let document = json!({"ld-json": [{"data": [{"@type": "Product", "url": format!("https://www.anker.com/{locale}/products/test")}]}]});
            assert!(is_other_storefront(&document, "au"));
            assert!(is_other_storefront(&document, "us"));
            assert!(!is_other_storefront(&document, locale));
        }
        for document in [
            json!({}),
            json!({"ld-json": []}),
            json!({"ld-json": [{"data": {"@type":"Product"}}]}),
            json!({"ld-json": [{"data": [{"@type":"Product", "url":"https://other.example/de/products/test"}]}]}),
            json!({"next_data": {}, "ld-json": [{"data": [{"@type":"Product", "url":"https://www.anker.com/de/products/test"}]}]}),
        ] {
            assert!(!is_other_storefront(&document, "au"));
        }
    }
}
