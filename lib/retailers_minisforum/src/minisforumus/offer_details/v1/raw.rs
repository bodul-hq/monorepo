//! Typed raw document for `minisforumus`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The storefront is a Shopify Horizon theme serving several markets (`/en-ca`, `/en-os`, ...).
//! The theme's schema.org islands state the market's currency and address; an app's copy of
//! Liquid's `product | json` states the product itself.

pub mod ldjsonitem;
pub mod productjson;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinisforumUsRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,

    pub product_json: productjson::ProductJsonItem,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::MinisforumUsRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant `en-ca` page, trimmed from
    /// `en-ca-products-minisforum-ms-01-se-workstation.html`.
    pub(crate) fn document() -> Value {
        json!({
            "ld-json": [
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::breadcrumbs()},
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::website()},
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::organization()},
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::product()}
            ],
            "product_json": {"data": super::productjson::tests::product()}
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<MinisforumUsRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn both_islands_are_required() {
        assert!(serde_json::from_str::<MinisforumUsRawDocument>(r#"{}"#).is_err());
        assert!(serde_json::from_str::<MinisforumUsRawDocument>(r#"{"ld-json":[]}"#).is_err());
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

        let error = serde_json::from_value::<MinisforumUsRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
