//! Typed raw document for `ugreencom`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.
//!
//! The brand site renders no product record but Shopify's analytics `meta`; the title, canonical
//! URL, gallery and the "Shop Now" button come from the page, the currency from the PayPal
//! metadata tag.

use analytics::AnalyticsScript;
use ldjson::LdJsonScript;
use media::MediaItem;
use paypal::PayPalMetadata;
use serde::Deserialize;
use shopnow::ShopNow;
pub mod analytics;
pub mod ldjson;
pub mod media;
pub mod paypal;
pub mod shopnow;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenComRawDocument {
    pub paypal: PayPalMetadata,

    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<LdJsonScript>,

    pub analytics: AnalyticsScript,

    pub og_title: OgTitle,

    pub canonical: Canonical,

    /// Missing on `products-us-85185.html`, present on the other 65 pages.
    #[serde(default)]
    pub shop_now: Option<ShopNow>,

    pub media: Vec<MediaItem>,
}

/// `meta[property="og:title"]`: the product title, equal to the analytics variant's `name` and
/// the rendered `h1` on every page. One title spaces its words with `\u{a0}`, as the analytics
/// record does; it is kept verbatim.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OgTitle {
    pub content: String,
}

/// `link[rel="canonical"]`: `https://www.ugreen.com/products/<handle>` on every page.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Canonical {
    pub href: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::from_value;

    use super::analytics::tests::script;
    use super::ldjson::tests::organization;
    use super::media::tests::image;
    use super::paypal::tests::metadata as tests_metadata;
    use super::shopnow::tests::button as tests_button;

    use super::UgreenComRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// `products-usa-15495.html`, trimmed.
    pub(crate) fn document() -> Value {
        json!({
            "paypal": tests_metadata(),
            "ld-json": [
                {"type": "application/ld+json", "data": organization()}
            ],
            "analytics": script(),
            "og_title": {"content": "UGREEN Revodok 105 5-in-1 USB-C Hub"},
            "canonical": {"href": "https://www.ugreen.com/products/usa-15495"},
            "shop_now": tests_button(),
            "media": [image()]
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        let raw =
            from_value::<UgreenComRawDocument>(document()).expect("the raw fixture deserializes");
        assert!(raw.shop_now.is_some());
    }

    #[test]
    fn a_page_without_the_shop_now_button_deserializes() {
        let mut input = document();
        input.as_object_mut().expect("an object").remove("shop_now");

        let raw = from_value::<UgreenComRawDocument>(input).expect("deserializes");
        assert_eq!(raw.shop_now, None);
    }

    #[test]
    fn every_other_structure_is_required() {
        for key in [
            "paypal",
            "ld-json",
            "analytics",
            "og_title",
            "canonical",
            "media",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(key);

            assert!(from_value::<UgreenComRawDocument>(input).is_err(), "{key}");
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

        let error = from_value::<UgreenComRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }

    #[test]
    fn an_unknown_structure_fails() {
        let mut input = document();
        input["price"] = json!({"content": "1300.00"});

        assert!(from_value::<UgreenComRawDocument>(input).is_err());
    }

    #[test]
    fn a_new_key_in_the_title_or_canonical_fails() {
        for key in ["og_title", "canonical"] {
            let mut input = document();
            input[key]["id"] = json!("x");

            assert!(from_value::<UgreenComRawDocument>(input).is_err(), "{key}");
        }
    }
}
