//! Typed raw document for `ugreeneu`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.
//!
//! The product comes from Shopify's ld+json, which parses on every page, and its analytics
//! `meta`; the main product section adds the selected variant's displayed prices, the picker's
//! record of that variant and the gallery.

pub mod analytics;
pub mod ldjson;
pub mod variant;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenEuRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjson::LdJsonScript>,
    pub analytics: analytics::AnalyticsScript,
    pub price: PriceBlock,
    pub selected_variant_id: SelectedVariantId,
    /// The variant picker's record of the selected variant. Only multi-variant products render
    /// the picker, so it is missing on the 10,639 `Product` pages.
    #[serde(default)]
    pub selected_variant: Option<variant::VariantScript>,
    pub images: Vec<Image>,
}

/// The main `product-price` element's data attributes, all strings. They describe the selected
/// variant as the theme displays it, after the shop's automatic discount.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceBlock {
    pub product_id: String,
    /// The displayed price in hundredths: the Shopify price, or below it where the automatic
    /// discount applies (2,237 pages).
    pub base_price_cents: String,
    /// The struck-through price in hundredths, never below the Shopify price: the variant's
    /// compare-at price, or its Shopify price where it has none.
    pub base_compare_cents: String,
    pub currency: String,
}

/// The main product form's hidden `id` input.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariantId {
    pub value: String,
}

/// An image slide of the main media gallery; video slides are left out.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    /// Protocol-relative, with the display `width` the theme asks the CDN for.
    pub src: String,
    pub alt: Option<String>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenEuRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `de-at-products-pb561.html`.
    pub(crate) fn document() -> Value {
        json!({
            "ld-json": [
                {"type": "application/ld+json", "data": super::ldjson::tests::organization()},
                {"type": "application/ld+json", "data": super::ldjson::tests::product_group()}
            ],
            "analytics": {"meta": super::analytics::tests::meta()},
            "price": {
                "product_id": "8625471816028",
                "base_price_cents": "5599",
                "base_compare_cents": "6999",
                "currency": "EUR"
            },
            "selected_variant_id": {"value": "48981450031452"},
            "selected_variant": {
                "type": "application/json",
                "data": super::variant::tests::variant()
            },
            "images": [
                {
                    "src": "//eu.ugreen.com/cdn/shop/files/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-451511.png?v=1719994480&width=3840",
                    "alt": "UGREEN Wireless Powerbank 10000mAh Magnetische Power Bank - UGREEN EU"
                },
                {
                    "src": "//eu.ugreen.com/cdn/shop/products/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-685866.png?v=1719994480&width=3840",
                    "alt": "UGREEN Wireless Powerbank 10000mAh Magnetische Power Bank - UGREEN"
                }
            ]
        })
    }

    /// The same product with only its first variant: a `Product` island and no picker.
    pub(crate) fn single_variant_document() -> Value {
        let mut document = document();
        document["ld-json"][1]["data"] = super::ldjson::tests::product();
        document["analytics"]["meta"]["product"]["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        document
            .as_object_mut()
            .expect("an object")
            .remove("selected_variant");
        document
    }

    #[test]
    fn the_fixtures_deserialize() {
        for document in [document(), single_variant_document()] {
            serde_json::from_value::<UgreenEuRawDocument>(document)
                .expect("the raw fixture deserializes");
        }
    }

    #[test]
    fn every_island_but_the_picker_record_is_required() {
        for island in [
            "ld-json",
            "analytics",
            "price",
            "selected_variant_id",
            "images",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(island);

            assert!(
                serde_json::from_value::<UgreenEuRawDocument>(input).is_err(),
                "{island}"
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

        let error = serde_json::from_value::<UgreenEuRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }

    #[test]
    fn an_unexpected_script_attribute_fails() {
        let mut ld_json = document();
        ld_json["ld-json"][0]["id"] = json!("organization");
        assert!(serde_json::from_value::<UgreenEuRawDocument>(ld_json).is_err());

        let mut record = document();
        record["selected_variant"]["data-section"] = json!("main");
        assert!(serde_json::from_value::<UgreenEuRawDocument>(record).is_err());

        let mut analytics = document();
        analytics["analytics"]["nonce"] = json!("x");
        assert!(serde_json::from_value::<UgreenEuRawDocument>(analytics).is_err());
    }

    #[test]
    fn an_unknown_price_attribute_fails() {
        let mut input = document();
        input["price"]["sale_price_cents"] = json!("1");

        let error =
            serde_json::from_value::<UgreenEuRawDocument>(input).expect_err("not in the corpus");
        assert!(
            error
                .to_string()
                .contains("unknown field `sale_price_cents`")
        );
    }
}
