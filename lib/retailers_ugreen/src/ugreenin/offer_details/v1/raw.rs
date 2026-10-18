//! Typed raw document for `ugreenin`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.
//!
//! The product is read from Liquid's `product | json`, which the Bitespeed app renders on every
//! page. It names no currency, so the Open Graph price-currency tag is read for that; Shopify's
//! analytics `meta` and the theme's schema.org island restate its ids, prices, SKUs and
//! availability, and the island is the only statement of the page's own URL, which carries the
//! locale.

pub mod analytics;
pub mod ldjson;
pub mod ogcurrency;
pub mod productjson;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenInRawDocument {
    pub og_currency: ogcurrency::OgCurrency,

    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjson::LdJsonScript>,

    pub analytics: analytics::AnalyticsScript,

    pub product_json: productjson::ProductJsonScript,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenInRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    fn script(data: Value) -> Value {
        json!({"type": "application/ld+json", "data": data})
    }

    /// A three-variant English page: every island the corpus renders, with the product as a
    /// `ProductGroup` (index 2) and Judge.me's reviews last.
    pub(crate) fn document() -> Value {
        json!({
            "og_currency": super::ogcurrency::tests::og_currency(),
            "ld-json": [
                script(super::ldjson::tests::organization()),
                script(super::ldjson::tests::breadcrumbs()),
                script(super::ldjson::tests::product_group()),
                script(super::ldjson::tests::faq()),
                {
                    "class": "jdgm-server-jld",
                    "type": "application/ld+json",
                    "data": super::ldjson::tests::reviews()
                }
            ],
            "analytics": {"meta": super::analytics::tests::meta()},
            "product_json": super::productjson::tests::script()
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<UgreenInRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn the_currency_the_islands_the_analytics_and_the_record_are_required() {
        for key in ["og_currency", "ld-json", "analytics", "product_json"] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(key);

            assert!(
                serde_json::from_value::<UgreenInRawDocument>(input).is_err(),
                "{key}"
            );
        }
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

        let error = serde_json::from_value::<UgreenInRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }

    #[test]
    fn an_unknown_top_level_key_fails() {
        let mut input = document();
        input["paypal"] = json!({});

        assert!(serde_json::from_value::<UgreenInRawDocument>(input).is_err());
    }
}
