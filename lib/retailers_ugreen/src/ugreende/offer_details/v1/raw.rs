//! Typed raw document for `ugreende`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.
//!
//! The storefront is a customised Shopify Horizon theme. Its product ld+json is not read: on some
//! products the theme replaces Shopify's island with a hand-written one that renders German
//! decimals (`"price": 43,99`) and is not JSON, which would fail the whole extraction. Everything
//! else comes from Shopify's analytics `meta` and the main product section.

pub mod analytics;
pub mod variant;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenDeRawDocument {
    pub analytics: analytics::AnalyticsScript,
    pub title: Title,
    pub price: PriceBlock,
    pub product_form: ProductForm,
    pub selected_variant_id: SelectedVariantId,
    pub add_to_cart: AddToCart,
    /// Only rendered when the product has options to choose between.
    pub variant_picker: Option<VariantPicker>,
    pub option_names: Vec<OptionName>,
    /// The variant picker's record of the selected variant; absent without a picker.
    pub selected_variant: Option<variant::VariantScript>,
    pub images: Vec<Image>,
}

/// The main product section's `h1`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Title {
    pub text: String,
}

/// The main `product-price` element's data attributes, all strings. They describe the selected
/// variant as the theme displays it, after any automatic discount.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceBlock {
    pub product_id: String,
    pub base_price_cents: String,
    pub base_compare_cents: String,
    pub currency: String,
}

/// The main product form, not the sticky bar's copy of it.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductForm {
    pub product_id: String,
    pub product_url: String,
}

/// The main product form's hidden `id` input.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariantId {
    pub value: String,
}

/// The main product form's add-to-cart button.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddToCart {
    /// Present, and empty, where the selected variant is sold out.
    pub disabled: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantPicker {
    pub product_id: String,
    pub product_url: String,
}

/// A picker fieldset's legend title, such as `Länge:`, in the order of the variants' option
/// slots.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionName {
    pub text: String,
}

/// An image slide of the main media gallery; video slides are left out.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenDeRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `products-100w-usb-c-kabel-mit-e-marker.html`.
    pub(crate) fn document() -> Value {
        json!({
            "analytics": {"meta": super::analytics::tests::meta()},
            "title": {"text": "UGREEN USB-C Kabel ( 100W ) mit E-Marker"},
            "price": {
                "product_id": "10080883081559",
                "base_price_cents": "1019",
                "base_compare_cents": "1099",
                "currency": "EUR"
            },
            "product_form": {
                "product_id": "10080883081559",
                "product_url": "/products/100w-usb-c-kabel-mit-e-marker"
            },
            "selected_variant_id": {"value": "51557243388247"},
            "add_to_cart": {},
            "variant_picker": {
                "product_id": "10080883081559",
                "product_url": "/products/100w-usb-c-kabel-mit-e-marker"
            },
            "option_names": [{"text": "Länge:"}],
            "selected_variant": {
                "type": "application/json",
                "data": super::variant::tests::variant()
            },
            "images": [{
                "src": "//de.ugreen.com/cdn/shop/files/71QIYZFjMgL.png?v=1760702732&width=3840",
                "alt": "UGREEN USB-C Kabel ( 100W ) mit E-Marker"
            }]
        })
    }

    /// A single-variant page: no picker, no legends, no variant record.
    pub(crate) fn single_variant_document() -> Value {
        let mut document = document();
        let object = document.as_object_mut().expect("an object");
        object.remove("variant_picker");
        object.remove("selected_variant");
        object.insert("option_names".to_string(), json!([]));
        document["analytics"]["meta"]["product"]["variants"] = json!([{
            "id": 51557243388247_u64, "price": 1019,
            "name": "UGREEN USB-C Kabel ( 100W ) mit E-Marker",
            "public_title": null, "sku": "65833"
        }]);
        document
    }

    #[test]
    fn the_fixtures_deserialize() {
        for document in [document(), single_variant_document()] {
            serde_json::from_value::<UgreenDeRawDocument>(document)
                .expect("the raw fixture deserializes");
        }
    }

    #[test]
    fn the_section_islands_are_required() {
        for island in [
            "analytics",
            "title",
            "price",
            "product_form",
            "selected_variant_id",
            "add_to_cart",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(island);

            assert!(
                serde_json::from_value::<UgreenDeRawDocument>(input).is_err(),
                "{island}"
            );
        }
    }

    #[test]
    fn an_unknown_island_fails() {
        let mut input = document();
        input["ld-json"] = json!([]);

        let error = serde_json::from_value::<UgreenDeRawDocument>(input)
            .expect_err("the ld+json island is not read");
        assert!(error.to_string().contains("unknown field `ld-json`"));
    }
}
