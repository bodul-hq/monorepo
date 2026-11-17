//! Typed raw document for `minisforumus`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The storefront is a Shopify Horizon theme serving several markets (`/en-ca`, `/en-os`, ...).
//! The theme's schema.org islands state the market's currency and address; an app's copy of
//! Liquid's `product | json` states the product itself.

use ldjsonitem::LdJsonItem;
use productjson::ProductJsonItem;
use serde::Deserialize;
pub mod ldjsonitem;
pub mod productjson;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinisforumUsRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<LdJsonItem>,

    pub product_json: ProductJsonItem,
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::from_str;
    use serde_json::from_value;

    use super::ldjsonitem::tests::breadcrumbs;
    use super::ldjsonitem::tests::organization;
    use super::ldjsonitem::tests::product as tests_product;
    use super::ldjsonitem::tests::website;
    use super::productjson::tests::product as productjson_tests_product;

    use super::MinisforumUsRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant `en-ca` page, trimmed from
    /// `en-ca-products-minisforum-ms-01-se-workstation.html`.
    pub(crate) fn document() -> Value {
        json!({
            "ld-json": [
                {"type": "application/ld+json", "data": breadcrumbs()},
                {"type": "application/ld+json", "data": website()},
                {"type": "application/ld+json", "data": organization()},
                {"type": "application/ld+json", "data": tests_product()}
            ],
            "product_json": {"data": productjson_tests_product()}
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        from_value::<MinisforumUsRawDocument>(document()).expect("the raw fixture deserializes");
    }

    #[test]
    fn both_islands_are_required() {
        assert!(from_str::<MinisforumUsRawDocument>(r#"{}"#).is_err());
        assert!(from_str::<MinisforumUsRawDocument>(r#"{"ld-json":[]}"#).is_err());
    }

    #[test]
    fn the_hyphenated_key_is_what_the_architecture_emits() {
        let mut input = document();
        let islands = input
            .as_object_mut()
            .expect("an object")
            .remove("ld-json")
            .expect("islands");
        input["ld_json"] = islands;

        let error = from_value::<MinisforumUsRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
