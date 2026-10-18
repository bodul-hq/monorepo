//! Typed raw document for `ugreenjp`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.
//!
//! The schema.org `Product` island carries every variant's offer, Shopify's analytics `meta` the
//! ids and handle, and the PayPal metadata tag the shop currency. The main product section adds
//! what only it states: the picker's `available` and `compare_at_price`, the selected variant,
//! the rendered price block and the gallery.

pub mod analytics;
pub mod ldjson;
pub mod paypal;
pub mod section;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenJpRawDocument {
    pub paypal: paypal::PayPalMetadata,

    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjson::LdJsonScript>,

    pub analytics: analytics::AnalyticsScript,

    /// Missing on the 63 `Default Title` products, which render no variant picker.
    #[serde(default)]
    pub variants: Option<section::VariantsScript>,

    pub selected_variant_id: section::SelectedVariantId,

    pub price: section::PriceBlock,

    pub media: Vec<section::MediaSlide>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenJpRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `products-lightning-usb-cable.html`.
    pub(crate) fn document() -> Value {
        json!({
            "paypal": super::paypal::tests::metadata(),
            "ld-json": [
                {"type": "application/ld+json", "data": super::ldjson::tests::organization()},
                {"type": "application/ld+json", "data": super::ldjson::tests::product()}
            ],
            "analytics": {"meta": super::analytics::tests::meta()},
            "variants": super::section::tests::variants(),
            "selected_variant_id": {"value": "49142586999064"},
            "price": super::section::tests::price("price price--product", "¥1,399", ""),
            "media": super::section::tests::media()
        })
    }

    /// A `Default Title` product on sale only in its rendered price block, as on
    /// `products-green-nasync-dh4300-plus-nas-storage.html`: no picker, no SKU.
    pub(crate) fn default_title_document() -> Value {
        let id = 52659056148760_u64;
        let url = "https://ugreen.jp/products/green-nasync-dh4300-plus-nas-storage";
        let mut document = document();
        document["ld-json"][1]["data"] = json!({
            "@context": "http://schema.org/",
            "@type": "Product",
            "name": "UGREEN NASync DH4300 Plus",
            "url": url,
            "image": ["https://ugreen.jp/cdn/shop/files/DH4300.png?v=1789197630&width=1920"],
            "description": "\n\n\n大容量128TBストレージ\n\n\nパワフルで効率的なハードウェア",
            "brand": {"@type": "Brand", "name": "株式会社ユーグリーン・ジャパン"},
            "offers": [{
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": 52700.0,
                "priceCurrency": "JPY",
                "url": format!("{url}?variant={id}")
            }]
        });
        document["analytics"]["meta"]["product"] = json!({
            "id": 10414739226904_u64,
            "gid": "gid://shopify/Product/10414739226904",
            "vendor": "株式会社ユーグリーン・ジャパン",
            "type": "",
            "handle": "green-nasync-dh4300-plus-nas-storage",
            "variants": [{"id": id, "price": 5270000, "name": "UGREEN NASync DH4300 Plus",
                "public_title": null, "sku": null}],
            "remote": false
        });
        document
            .as_object_mut()
            .expect("an object")
            .remove("variants");
        document["selected_variant_id"] = json!({"value": id.to_string()});
        document["price"] = super::section::tests::price(
            "price price--product price--on-sale",
            "¥52,700",
            "¥65,880",
        );
        document["media"] = json!([{
            "id": "template--22963901956376__main-44874680205592",
            "src": "//ugreen.jp/cdn/shop/files/DH4300.png?v=1789197630&width=1946",
            "alt": ""
        }]);
        document
    }

    #[test]
    fn the_fixtures_deserialize() {
        serde_json::from_value::<UgreenJpRawDocument>(document())
            .expect("the raw fixture deserializes");
        let default_title = serde_json::from_value::<UgreenJpRawDocument>(default_title_document())
            .expect("the Default Title fixture deserializes");
        assert!(default_title.variants.is_none());
    }

    #[test]
    fn every_key_but_the_picker_is_required() {
        for key in [
            "paypal",
            "ld-json",
            "analytics",
            "selected_variant_id",
            "price",
            "media",
        ] {
            let mut input = document();
            input.as_object_mut().expect("an object").remove(key);

            assert!(
                serde_json::from_value::<UgreenJpRawDocument>(input).is_err(),
                "{key}"
            );
        }
    }

    #[test]
    fn the_hyphenated_key_is_what_the_architecture_emits() {
        let mut input = document();
        let items = input
            .as_object_mut()
            .expect("an object")
            .remove("ld-json")
            .expect("ld-json");
        input["ld_json"] = items;

        let error = serde_json::from_value::<UgreenJpRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }

    #[test]
    fn an_unknown_top_level_key_fails() {
        let mut input = document();
        input["breadcrumbs"] = Value::Null;

        assert!(serde_json::from_value::<UgreenJpRawDocument>(input).is_err());
    }
}
