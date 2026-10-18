//! Typed raw document for `minisforumuk`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The product is read from Liquid's `product | json`, which a storefront app renders into
//! `script#xcotton_pp_variants` on every page. The schema.org island restates its offers but is
//! missing where the product has no price, and the record names no currency, so the PayPal
//! metadata tag is read for that.

pub mod ldjson;
pub mod paypal;
pub mod productjson;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinisforumUkRawDocument {
    pub paypal: paypal::PayPalMetadata,

    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjson::LdJsonScript>,

    pub product_json: productjson::ProductJsonScript,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::MinisforumUkRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page: the Organization island, a ProductGroup island and the record.
    pub(crate) fn document() -> Value {
        json!({
            "paypal": super::paypal::tests::metadata(),
            "ld-json": [
                {"type": "application/ld+json", "data": super::ldjson::tests::organization()},
                {"type": "application/ld+json", "data": super::ldjson::tests::product_group()}
            ],
            "product_json": {
                "data-cmp-ab": "1",
                "data-cookieconsent": "ignore",
                "data-ot-ignore": "",
                "data-ta-type": "ignore",
                "data-uc-allowed": "true",
                "id": "xcotton_pp_variants",
                "type": "application/json",
                "data": super::productjson::tests::product()
            }
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<MinisforumUkRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn the_metadata_the_islands_and_the_record_are_required() {
        for key in ["paypal", "ld-json", "product_json"] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(key);

            assert!(
                serde_json::from_value::<MinisforumUkRawDocument>(input).is_err(),
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

        let error = serde_json::from_value::<MinisforumUkRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
