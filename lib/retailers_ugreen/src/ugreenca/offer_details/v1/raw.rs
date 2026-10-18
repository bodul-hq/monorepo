//! Typed raw document for `ugreenca`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.
//!
//! The storefront is a Shopify Kalles (`t4s`) theme serving English and `/fr/` French pages. It
//! renders a `Product` ld+json island, but that island states one image and, on some
//! multi-variant pages, a single offer; the variants are read from the theme's and Shopify's own
//! islands instead.

pub mod analytics;
pub mod featured;
pub mod ldjsonitem;
pub mod shop;
pub mod variant;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenCaRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,
    pub shop: shop::ShopScript,
    pub analytics: analytics::AnalyticsScript,
    pub title: Title,
    pub featured: featured::FeaturedAttribute,
    /// Rendered only where the product has a variant picker: not on a `Default Title` product.
    pub variants: Option<variant::VariantsScript>,
    /// Rendered alongside `variants`.
    pub options: Option<variant::OptionsScript>,
    pub media: Vec<Media>,
}

/// `h1.t4s-product__title`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Title {
    pub text: String,
}

/// A slide of the main product gallery.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Media {
    pub id: String,
    pub media_type: String,
    /// The image's `data-master`: the file at its version, with no display size.
    pub src: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenCaRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `products-100w-3c1a-gan-fast-charger.html`.
    pub(crate) fn document() -> Value {
        json!({
            "ld-json": super::ldjsonitem::tests::islands(),
            "shop": {
                "id": "apple-pay-shop-capabilities",
                "type": "application/json",
                "data": super::shop::tests::data()
            },
            "analytics": {"meta": super::analytics::tests::meta()},
            "title": {"text": "UGREEN Nexode 100W USB C Charger"},
            "featured": {"data": super::featured::tests::data()},
            "variants": {
                "type": "application/json",
                "class": "pr_variants_json",
                "data": [
                    super::variant::tests::variant(44061363634464, 5999, "Black", "6957303813346"),
                    super::variant::tests::variant(44061396533536, 6499, "White", "")
                ]
            },
            "options": {
                "type": "application/json",
                "class": "pr_options_json",
                "data": [{"name": "Color", "position": 1, "values": ["Black", "White"]}]
            },
            "media": [
                {"id": "35687432159520", "media_type": "image",
                    "src": "//ca.ugreen.com/cdn/shop/files/charger.png?v=1746739856"}
            ]
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<UgreenCaRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn a_default_title_page_renders_no_picker_islands() {
        let mut input = document();
        let object = input.as_object_mut().expect("an object");
        object.remove("variants");
        object.remove("options");

        let raw = serde_json::from_value::<UgreenCaRawDocument>(input)
            .expect("the picker islands are optional");
        assert!(raw.variants.is_none() && raw.options.is_none());
    }

    #[test]
    fn the_other_islands_are_required() {
        for island in ["ld-json", "shop", "analytics", "title", "featured", "media"] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(island);

            assert!(
                serde_json::from_value::<UgreenCaRawDocument>(input).is_err(),
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

        let error = serde_json::from_value::<UgreenCaRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
