//! Typed raw document for `ugreennasde`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.
//!
//! The theme renders no product ld+json, so the product is read from Shopify's analytics `meta`
//! (every variant, its SKU and price), the picker's `product.variants | json` (barcode,
//! compare-at price and availability per variant, where the product renders a picker) and the
//! main section's title, cart form, price block and gallery. The PayPal metadata tag names the
//! currency.

pub mod analytics;
pub mod ldjson;
pub mod paypal;
pub mod variants;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenNasDeRawDocument {
    pub paypal: paypal::PayPalMetadata,

    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjson::LdJsonScript>,

    pub analytics: analytics::AnalyticsScript,

    pub title: Title,

    /// The picker's records. Missing on the 3 saved pages whose single-variant product renders
    /// no picker (`ugreen-nas-10gbps-m2-nvme-ssd-enclosure`,
    /// `ugreen-nas-72w-dc-uninterruptible-power-supply`, `ugreen-nas-backup-power-120w-12000mah`).
    #[serde(default)]
    pub variants: Option<variants::VariantsScript>,

    pub selected_variant: SelectedVariant,

    pub price: PriceBlock,

    pub images: Vec<Image>,
}

/// The main section's `h1`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Title {
    pub text: String,
}

/// The main cart form's hidden `id` input: the variant the page opens on.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariant {
    /// The variant's numeric id, as a string.
    pub value: String,
    /// Present, and empty, exactly where the variant is sold out.
    #[serde(default)]
    pub disabled: Option<String>,
}

/// Dawn's price snippet for the selected variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceBlock {
    /// Space-separated: `price price--large`, plus `price--on-sale` where a compare-at price
    /// above the price is struck through, `price--sold-out` where the variant is sold out, and
    /// `price--show-badge` on the add-on.
    pub class: String,
    /// The price, rendered in the shop's locale (`589,99€`).
    pub regular: Text,
    /// The price again, in the on-sale layout.
    pub sale: Text,
    /// The struck-through price: the compare-at price on sale, empty or the price otherwise.
    pub compare_at: Text,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub text: String,
}

/// A gallery slide's `img`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    /// Protocol-relative, with the display `width` the theme asks the CDN for.
    pub src: String,
    pub alt: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenNasDeRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page with a picker, on sale, trimmed from
    /// `products-ugreen-nasync-dxp4800-gt-4-bay-nas-storage.html`.
    pub(crate) fn document() -> Value {
        json!({
            "paypal": super::paypal::tests::metadata(),
            "ld-json": super::ldjson::tests::scripts(),
            "analytics": {"meta": super::analytics::tests::meta()},
            "title": {"text": "UGREEN NASync DXP4800 GT 4-Bay 144TB NAS"},
            "variants": super::variants::tests::script(),
            "selected_variant": {"value": "52118906241255"},
            "price": {
                "class": "price price--large price--on-sale",
                "regular": {"text": "589,99€"},
                "sale": {"text": "589,99€"},
                "compare_at": {"text": "659,99€"}
            },
            "images": [
                {"src": "//ai-de.ugreen.com/cdn/shop/files/DXP4800_GT_16058.webp?v=1780565809&width=1445",
                    "alt": "UGREEN NASync DXP4800 GT 4-Bay 144TB NAS"},
                {"src": "//ai-de.ugreen.com/cdn/shop/files/07_813b72a9-9cac-4d55-b7c0-8c1be5a945ad.jpg?v=1785465917&width=1445",
                    "alt": "UGREEN NASync DXP4800 GT 4-Bay 144TB NAS"}
            ]
        })
    }

    /// The fixture as a single-variant product without a picker, as on `ups2000`.
    pub(crate) fn document_without_picker() -> Value {
        let mut input = document();
        let object = input.as_object_mut().expect("an object");
        object.remove("variants");
        input["analytics"]["meta"]["product"]["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        input
    }

    #[test]
    fn the_fixtures_deserialize() {
        for input in [document(), document_without_picker()] {
            serde_json::from_value::<UgreenNasDeRawDocument>(input)
                .expect("the raw fixture deserializes");
        }
    }

    #[test]
    fn every_island_but_the_picker_is_required() {
        for key in [
            "paypal",
            "ld-json",
            "analytics",
            "title",
            "selected_variant",
            "price",
            "images",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(key);

            assert!(
                serde_json::from_value::<UgreenNasDeRawDocument>(input).is_err(),
                "{key}"
            );
        }
    }

    #[test]
    fn a_sold_out_variant_renders_a_disabled_input() {
        let mut input = document();
        input["selected_variant"]["disabled"] = json!("");

        let raw = serde_json::from_value::<UgreenNasDeRawDocument>(input).expect("deserializes");
        assert_eq!(raw.selected_variant.disabled.as_deref(), Some(""));
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

        let error = serde_json::from_value::<UgreenNasDeRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }

    #[test]
    fn an_unknown_key_fails() {
        for (path, key) in [
            ("", "canonical"),
            ("/price", "badge"),
            ("/images/0", "width"),
        ] {
            let mut input = document();
            input
                .pointer_mut(path)
                .and_then(Value::as_object_mut)
                .expect("an object")
                .insert(key.to_string(), json!("x"));

            assert!(
                serde_json::from_value::<UgreenNasDeRawDocument>(input).is_err(),
                "{path}/{key}"
            );
        }
    }
}
