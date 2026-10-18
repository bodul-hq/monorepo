//! Typed raw document for `minisforumkr`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The storefront is a Shopify Horizon theme. It renders a schema.org `ProductGroup` (or, for a
//! product without options, a `Product`) island, Shopify's analytics `meta`, and the variant
//! picker's record of the selected variant; the picker renders no record of the other variants.

pub mod analytics;
pub mod ldjsonitem;
pub mod variant;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinisforumKrRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonScript>,
    pub analytics: analytics::AnalyticsScript,
    /// Absent where the product has no options to pick, and on landing pages.
    pub variant_picker: Option<VariantPicker>,
    pub option_names: Vec<OptionName>,
    /// Absent wherever the variant picker is.
    pub selected_variant: Option<variant::VariantScript>,
    pub images: Vec<Image>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantPicker {
    pub product_id: String,
    /// Percent-encoded where the handle is not ASCII.
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
    use super::MinisforumKrRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `products-minisforum-um890-pro.html`.
    pub(crate) fn document() -> Value {
        json!({
            "ld-json": [
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::organization()},
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::product_group()}
            ],
            "analytics": {"meta": super::analytics::tests::meta()},
            "variant_picker": {
                "product_id": "7128100143150",
                "product_url": "/products/minisforum-um890-pro"
            },
            "option_names": [{"text": "CPU:"}, {"text": "Type:"}, {"text": "사양:"}],
            "selected_variant": {
                "type": "application/json",
                "data": super::variant::tests::variant()
            },
            "images": [{
                "src": "//www.minisforum.kr/cdn/shop/files/UM890Pro1.jpg?v=1776075578&width=1946",
                "alt": "MINISFORUM UM890 Pro 고성능 미니PC"
            }]
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<MinisforumKrRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn a_page_without_a_picker_renders_no_selected_variant() {
        let mut input = document();
        let object = input.as_object_mut().expect("an object");
        object.remove("variant_picker");
        object.remove("selected_variant");
        object.insert("option_names".to_string(), json!([]));

        let raw = serde_json::from_value::<MinisforumKrRawDocument>(input)
            .expect("the picker islands are optional");
        assert!(raw.variant_picker.is_none() && raw.selected_variant.is_none());
    }

    #[test]
    fn the_analytics_and_the_ld_json_are_required() {
        for island in ["ld-json", "analytics", "option_names", "images"] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(island);

            assert!(
                serde_json::from_value::<MinisforumKrRawDocument>(input).is_err(),
                "{island}"
            );
        }
    }

    #[test]
    fn the_hyphenated_key_is_what_the_architecture_emits() {
        let mut input = document();
        let object = input.as_object_mut().expect("an object");
        let ld_json = object.remove("ld-json").expect("ld-json");
        object.insert("ld_json".to_string(), ld_json);

        let error = serde_json::from_value::<MinisforumKrRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
