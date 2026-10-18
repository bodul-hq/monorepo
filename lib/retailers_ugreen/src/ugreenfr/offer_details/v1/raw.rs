//! Typed raw document for `ugreenfr`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.
//!
//! The storefront is `ugreende`'s Horizon theme, but here the product ld+json is valid JSON on
//! every saved page and is read: it is the only island with every variant's availability and
//! barcode. Shopify's analytics `meta` lists the variants with their ids; the main section's
//! price block, product form, picker and gallery describe the selected variant and the images.

pub mod analytics;
pub mod ldjson;
pub mod variant;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenFrRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjson::LdJsonScript>,
    pub analytics: analytics::AnalyticsScript,
    pub price: PriceBlock,
    pub product_form: ProductForm,
    pub selected_variant_id: SelectedVariantId,
    /// Empty without a picker, that is on the 262 single-`Product` pages.
    pub option_names: Vec<OptionName>,
    /// The variant picker's record of the selected variant; missing without a picker.
    pub selected_variant: Option<variant::VariantScript>,
    pub images: Vec<Image>,
}

/// The main `product-price` element's data attributes, all strings. They describe the selected
/// variant as the theme displays it, after any automatic discount.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceBlock {
    pub product_id: String,
    /// Hundredths of `currency`: the price shown.
    pub base_price_cents: String,
    /// Hundredths of `currency`: the struck-through price, equal to `base_price_cents` where
    /// none is shown.
    pub base_compare_cents: String,
    pub currency: String,
}

/// The main product form, not the sticky bar's copy of it.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductForm {
    pub product_id: String,
    /// A path, `/products/<handle>`.
    pub product_url: String,
    /// `"true"` on the 40 pages whose displayed price is the automatic discount's; missing on
    /// the other 266.
    pub auto_discount: Option<String>,
    /// `DL<sku>`, exactly where `auto_discount` is present.
    pub discount_code: Option<String>,
}

/// The main product form's hidden `id` input.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariantId {
    pub value: String,
}

/// A picker fieldset's legend title, such as `Pack:`, in the order of the variants' option
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
    /// Protocol-relative, with the display `width` the theme asks the CDN for.
    pub src: String,
    pub alt: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenFrRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    fn script(data: Value) -> Value {
        json!({"type": "application/ld+json", "data": data})
    }

    /// A two-variant page with a picker, trimmed from `products-45699.html`, with a Judge.me
    /// review island added.
    pub(crate) fn document() -> Value {
        json!({
            "ld-json": [
                script(super::ldjson::tests::organization()),
                script(super::ldjson::tests::product_group()),
                {
                    "class": "jdgm-server-jld",
                    "type": "application/ld+json",
                    "data": super::ldjson::tests::reviews("45699")
                }
            ],
            "analytics": {"meta": super::analytics::tests::meta()},
            "price": {
                "product_id": "9526794977600",
                "base_price_cents": "13499",
                "base_compare_cents": "17999",
                "currency": "EUR"
            },
            "product_form": {
                "product_id": "9526794977600",
                "product_url": "/products/45699"
            },
            "selected_variant_id": {"value": "50354801180992"},
            "option_names": [{"text": "Pack:"}],
            "selected_variant": {
                "type": "application/json",
                "data": super::variant::tests::variant()
            },
            "images": [
                {
                    "src": "//fr.ugreen.com/cdn/shop/files/charger.png?v=1738861172&width=3840",
                    "alt": "UGREEN 300W Nexode Chargeur"
                },
                {
                    "src": "//fr.ugreen.com/cdn/shop/files/61kMB5x0AfL.jpg?v=1738834470&width=3840",
                    "alt": "alt"
                }
            ]
        })
    }

    /// A single-variant page under an automatic discount, trimmed from `products-15532.html`:
    /// no picker, no option names, no variant record.
    pub(crate) fn discounted_document() -> Value {
        json!({
            "ld-json": [
                script(super::ldjson::tests::organization()),
                script(super::ldjson::tests::product())
            ],
            "analytics": {"meta": {
                "product": {
                    "id": 8986551157056_u64,
                    "gid": "gid://shopify/Product/8986551157056",
                    "vendor": "UGREEN",
                    "type": "USB-C HUB",
                    "handle": "15532",
                    "variants": [{
                        "id": 47543757046080_u64, "price": 6199,
                        "name": "UGREEN Revodok Pro 109 Hub USB C",
                        "public_title": null, "sku": "15532"
                    }],
                    "remote": false
                },
                "page": {
                    "pageType": "product",
                    "resourceType": "product",
                    "resourceId": 8986551157056_u64,
                    "requestId": "755ba429-8081-4c67-8931-b62cd0ae535c-1792012972"
                }
            }},
            "price": {
                "product_id": "8986551157056",
                "base_price_cents": "5299",
                "base_compare_cents": "6199",
                "currency": "EUR"
            },
            "product_form": {
                "product_id": "8986551157056",
                "product_url": "/products/15532",
                "auto_discount": "true",
                "discount_code": "DL15532"
            },
            "selected_variant_id": {"value": "47543757046080"},
            "option_names": [],
            "images": [{
                "src": "//fr.ugreen.com/cdn/shop/files/hub.png?v=1746741214&width=3840",
                "alt": "UGREEN Revodok Pro 109 Hub USB C"
            }]
        })
    }

    #[test]
    fn the_fixtures_deserialize() {
        for document in [document(), discounted_document()] {
            serde_json::from_value::<UgreenFrRawDocument>(document)
                .expect("the raw fixture deserializes");
        }
    }

    #[test]
    fn the_islands_are_required() {
        for island in [
            "ld-json",
            "analytics",
            "price",
            "product_form",
            "selected_variant_id",
            "option_names",
            "images",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(island);

            assert!(
                serde_json::from_value::<UgreenFrRawDocument>(input).is_err(),
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

        let error = serde_json::from_value::<UgreenFrRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }

    #[test]
    fn the_discount_attributes_are_read() {
        let raw: UgreenFrRawDocument =
            serde_json::from_value(discounted_document()).expect("deserializes");
        assert_eq!(raw.product_form.auto_discount.as_deref(), Some("true"));
        assert_eq!(raw.product_form.discount_code.as_deref(), Some("DL15532"));
    }

    #[test]
    fn an_unknown_island_or_attribute_fails() {
        let mut island = document();
        island["add_to_cart"] = json!({});
        let error = serde_json::from_value::<UgreenFrRawDocument>(island)
            .expect_err("the button is not read");
        assert!(error.to_string().contains("unknown field `add_to_cart`"));

        let mut attribute = document();
        attribute["price"]["block_id"] = json!("x");
        assert!(serde_json::from_value::<UgreenFrRawDocument>(attribute).is_err());
    }
}
