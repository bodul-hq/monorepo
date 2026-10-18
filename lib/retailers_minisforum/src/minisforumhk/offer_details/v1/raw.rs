//! Typed raw document for `minisforumhk`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The storefront is a Shopify Liquid theme that does not render the `xcotton_pp_variants`
//! record the other Minisforum shops carry. The product is assembled from what each page does
//! render, all of it on every one of the 29 saved pages:
//!
//! - the schema.org `Product` island: name, path, brand, category, description, primary image,
//!   and one offer per variant;
//! - the theme's `bm_product_variants` script: every variant, in Liquid's variant shape;
//! - `bm_product_selected_or_first_available_variant`: the variant the page opens on;
//! - `seal_product`: the product id and title;
//! - the PayPal metadata tag: the shop currency, which the variant scripts do not name;
//! - the variant selector's option names, the one DOM read: nothing else names the options.
//!
//! What the pages do not carry, and the document therefore cannot: the handle (it is the last
//! segment of the product path), the vendor and product type, tags, a gallery beyond the primary
//! image, and the publish dates.

pub mod ldjson;
pub mod paypal;
pub mod sealproduct;
pub mod variants;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinisforumHkRawDocument {
    pub paypal: paypal::PayPalMetadata,

    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjson::LdJsonScript>,

    pub variants: variants::VariantsScript,

    pub selected_variant: variants::SelectedVariantScript,

    pub seal_product: sealproduct::SealProductScript,

    /// The variant selector's labels, such as `處理器:`, in the order of the variants' option
    /// slots. Empty on the 3 saved pages whose product has no options to choose.
    pub option_names: Vec<OptionName>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionName {
    pub text: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::MinisforumHkRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `products-minisforum-ms-s1-max.html`.
    pub(crate) fn document() -> Value {
        json!({
            "paypal": super::paypal::tests::metadata(),
            "ld-json": [
                {"type": "application/ld+json", "data": super::ldjson::tests::product()},
                {"type": "application/ld+json", "data": super::ldjson::tests::breadcrumbs()},
                {"type": "application/ld+json", "data": super::ldjson::tests::organization()}
            ],
            "variants": {
                "id": "bm_product_variants",
                "type": "application/json",
                "data": super::variants::tests::variants()
            },
            "selected_variant": {
                "id": "bm_product_selected_or_first_available_variant",
                "type": "application/json",
                "data": super::variants::tests::variants()[0]
            },
            "seal_product": {
                "id": "seal_product",
                "type": "application/json",
                "data": super::sealproduct::tests::seal_product()
            },
            "option_names": [{"text": "CPU:"}, {"text": "規格:"}]
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<MinisforumHkRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn every_island_is_required() {
        for key in [
            "paypal",
            "ld-json",
            "variants",
            "selected_variant",
            "seal_product",
            "option_names",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(key);

            assert!(
                serde_json::from_value::<MinisforumHkRawDocument>(input).is_err(),
                "{key}"
            );
        }
    }

    #[test]
    fn a_page_without_options_has_no_option_names() {
        let mut input = document();
        input["option_names"] = json!([]);

        let raw = serde_json::from_value::<MinisforumHkRawDocument>(input)
            .expect("the option names may be empty");
        assert!(raw.option_names.is_empty());
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

        let error = serde_json::from_value::<MinisforumHkRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = document();
        input["product_json"] = json!({});

        assert!(serde_json::from_value::<MinisforumHkRawDocument>(input).is_err());
    }
}
