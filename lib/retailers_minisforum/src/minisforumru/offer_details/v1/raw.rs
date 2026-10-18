//! Typed raw document for `minisforumru`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The storefront is a Shopify Horizon theme that renders no `Product` ld+json island, so the
//! product is read from the islands the theme and Shopify's own scripts leave on the page.

pub mod analytics;
pub mod shop;
pub mod variant;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinisforumRuRawDocument {
    pub shop: shop::ShopScript,
    pub analytics: analytics::AnalyticsScript,
    pub title: Title,
    pub variant_picker: VariantPicker,
    pub option_names: Vec<OptionName>,
    pub selected_variant: variant::VariantScript,
    /// Only rendered when the product has more than one variant.
    pub variants: Option<variant::VariantsScript>,
    pub images: Vec<Image>,
}

/// The product page's only `h1`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Title {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantPicker {
    pub product_id: String,
    pub product_url: String,
}

/// A picker fieldset's legend, such as `CPU:`, in the order of the variants' option slots.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionName {
    pub text: String,
}

/// A slide of the main media gallery.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::MinisforumRuRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `products-minisforum-ms-s1-max.html`.
    pub(crate) fn document() -> Value {
        json!({
            "shop": {
                "id": "apple-pay-shop-capabilities",
                "type": "application/json",
                "data": super::shop::tests::data()
            },
            "analytics": {"meta": super::analytics::tests::meta()},
            "title": {"text": "MINISFORUM MS-S1 MAX"},
            "variant_picker": {
                "product_id": "8640842891442",
                "product_url": "/products/minisforum-ms-s1-max"
            },
            "option_names": [{"text": "CPU:"}, {"text": "Type:"}, {"text": "Adapter:"}],
            "selected_variant": {
                "type": "application/json",
                "data": super::variant::tests::variant(47567232336050, 259900, "64GB RAM+2TB SSD")
            },
            "variants": {
                "type": "application/json",
                "data-variants-json": "",
                "data": [
                    super::variant::tests::variant(47567232336050, 259900, "64GB RAM+2TB SSD"),
                    super::variant::tests::variant(47567232368818, 299900, "128GB RAM+2TB SSD")
                ]
            },
            "images": [{
                "src": "//minisforum.ru/cdn/shop/files/S1MAX.png?v=1781059869&width=1946",
                "alt": "MINISFORUM MS-S1 MAX"
            }]
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<MinisforumRuRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn a_single_variant_page_renders_no_variants_island() {
        let mut input = document();
        input.as_object_mut().expect("an object").remove("variants");

        let raw = serde_json::from_value::<MinisforumRuRawDocument>(input)
            .expect("the variants island is optional");
        assert!(raw.variants.is_none());
    }

    #[test]
    fn the_other_islands_are_required() {
        for island in [
            "shop",
            "analytics",
            "title",
            "variant_picker",
            "selected_variant",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(island);

            assert!(
                serde_json::from_value::<MinisforumRuRawDocument>(input).is_err(),
                "{island}"
            );
        }
    }

    #[test]
    fn an_unknown_island_fails() {
        let mut input = document();
        input["ld-json"] = json!([]);

        let error = serde_json::from_value::<MinisforumRuRawDocument>(input)
            .expect_err("this storefront renders no product ld+json");
        assert!(error.to_string().contains("unknown field `ld-json`"));
    }
}
