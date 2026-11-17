//! Typed raw document for `minisforumeu`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The product is read from Liquid's `product | json`, which a storefront app renders into
//! `script#xcotton_pp_variants` on every page. The record names no currency, so the PayPal
//! metadata tag is read for that, and for the page's locale. The page layout is
//! `minisforumau`'s, with 3 to 5 schema.org islands instead of 1 to 2 (see [`ldjson::LdJson`]),
//! of which the theme's `ProductGroup` or `Product` restates the record's offers and is checked
//! against it; the others are carried so that a change in their shape is seen, and read by
//! nothing. The theme's island is missing where the product has no price (2 of the 274 saved pages).
//! Pages come in pairs, `products-*.html` and `de-products-*.html` (137 each), the same product
//! in the storefront's two locales.

use ldjson::LdJsonScript;
use paypal::PayPalMetadata;
use productjson::ProductJsonScript;
use serde::Deserialize;
pub mod ldjson;
mod metadata;
pub mod paypal;
pub mod productjson;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinisforumEuRawDocument {
    pub paypal: PayPalMetadata,

    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<LdJsonScript>,

    pub product_json: ProductJsonScript,
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::from_value;

    use super::ldjson::tests::feed_product;
    use super::ldjson::tests::graph;
    use super::ldjson::tests::organization;
    use super::ldjson::tests::product_group;
    use super::ldjson::tests::reviewed_product;
    use super::paypal::tests::metadata as tests_metadata;
    use super::productjson::tests::product as tests_product;

    use super::MinisforumEuRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page: the merchant feed app's island, the SEO app's envelope, the
    /// Organization island, the theme's ProductGroup island and the record.
    pub(crate) fn document() -> Value {
        json!({
            "paypal": tests_metadata(),
            "ld-json": [
                {"type": "application/ld+json", "data": feed_product()},
                {
                    "type": "application/ld+json",
                    "data-seoant": "1",
                    "data-seoant-version": "v2",
                    "data": graph()
                },
                {"type": "application/ld+json", "data": organization()},
                {"type": "application/ld+json", "data": product_group()}
            ],
            "product_json": {
                "data-cmp-ab": "1",
                "data-cookieconsent": "ignore",
                "data-ot-ignore": "",
                "data-ta-type": "ignore",
                "data-uc-allowed": "true",
                "id": "xcotton_pp_variants",
                "type": "application/json",
                "data": tests_product()
            }
        })
    }

    /// The reviews app's island, as the script that carries it.
    pub(crate) fn reviews_script() -> Value {
        json!({
            "type": "application/ld+json",
            "id": "a2r-03hu34xd-uhj7389-x",
            "data": reviewed_product()
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        from_value::<MinisforumEuRawDocument>(document()).expect("the raw fixture deserializes");
    }

    #[test]
    fn the_reviews_app_island_deserializes() {
        let mut input = document();
        input["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(reviews_script());

        from_value::<MinisforumEuRawDocument>(input)
            .expect("the raw fixture with reviews deserializes");
    }

    #[test]
    fn the_metadata_the_islands_and_the_record_are_required() {
        for key in ["paypal", "ld-json", "product_json"] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(key);

            assert!(
                from_value::<MinisforumEuRawDocument>(input).is_err(),
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

        let error = from_value::<MinisforumEuRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
