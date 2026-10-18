use html_extractor::serde_json::Value;

/// The variant picker's record of the selected variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: Variant,
}

/// Liquid's `variant | json`. Its amounts are in hundredths of the shop currency, which it does
/// not name.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    pub sku: Option<String>,
    pub requires_shipping: bool,
    pub taxable: bool,
    pub featured_image: Option<FeaturedImage>,
    pub available: bool,
    pub name: String,
    pub public_title: Option<String>,
    pub options: Vec<String>,
    /// The Shopify price, before any automatic discount.
    pub price: i64,
    pub weight: i64,
    /// Null on 113 of the 1,848 records; equal to `price`, so no markdown, on 612 others.
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
    /// The ld+json variant's `gtin` on every record, or null where it has none.
    pub barcode: Option<String>,
    /// Missing, rather than null, on the 168 records without a `featured_image`.
    #[serde(default)]
    pub featured_media: Option<FeaturedMedia>,
    pub requires_selling_plan: bool,
    pub selling_plan_allocations: Vec<Value>,
    pub quantity_rule: QuantityRule,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturedImage {
    pub id: u64,
    pub product_id: u64,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
    pub alt: Option<String>,
    pub width: i64,
    pub height: i64,
    pub src: String,
    pub variant_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturedMedia {
    pub alt: Option<String>,
    pub id: u64,
    pub position: i64,
    pub preview_image: PreviewImage,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewImage {
    pub aspect_ratio: f64,
    pub height: i64,
    pub width: i64,
    pub src: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityRule {
    pub min: i64,
    pub max: Option<i64>,
    pub increment: i64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Variant;
    use html_extractor::serde_json::{self, Value, json};

    /// The `Schwarz` variant of `de-at-products-pb561.html`.
    pub(crate) fn variant() -> Value {
        json!({
            "id": 48981450031452_u64,
            "title": "Schwarz",
            "option1": "Schwarz",
            "option2": null,
            "option3": null,
            "sku": "25919",
            "requires_shipping": true,
            "taxable": true,
            "featured_image": {
                "id": 54716146254172_u64,
                "product_id": 8625471816028_u64,
                "position": 1,
                "created_at": "2024-07-01T21:06:46+08:00",
                "updated_at": "2024-07-03T16:14:40+08:00",
                "alt": "UGREEN Wireless Powerbank 10000mAh Magnetische Power Bank - UGREEN EU",
                "width": 1200,
                "height": 1200,
                "src": "//eu.ugreen.com/cdn/shop/files/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-451511.png?v=1719994480",
                "variant_ids": [48981450031452_u64]
            },
            "available": true,
            "name": "UGREEN Kabelloses Magnetisches Powerbank 10000mAh - Schwarz",
            "public_title": "Schwarz",
            "options": ["Schwarz"],
            "price": 5599,
            "weight": 0,
            "compare_at_price": 6999,
            "inventory_management": "shopify",
            "barcode": "6941876229192",
            "featured_media": {
                "alt": "UGREEN Wireless Powerbank 10000mAh Magnetische Power Bank - UGREEN EU",
                "id": 47380209434972_u64,
                "position": 1,
                "preview_image": {
                    "aspect_ratio": 1.0,
                    "height": 1200,
                    "width": 1200,
                    "src": "//eu.ugreen.com/cdn/shop/files/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-451511.png?v=1719994480"
                }
            },
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    #[test]
    fn the_record_deserializes() {
        serde_json::from_value::<Variant>(variant()).expect("deserializes");
    }

    #[test]
    fn a_variant_without_its_own_image_has_no_media_key() {
        let mut input = variant();
        input["featured_image"] = Value::Null;
        input
            .as_object_mut()
            .expect("an object")
            .remove("featured_media");

        let variant = serde_json::from_value::<Variant>(input).expect("deserializes");
        assert!(variant.featured_image.is_none() && variant.featured_media.is_none());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = variant();
        input["unit_price"] = json!(1);

        let error = serde_json::from_value::<Variant>(input).expect_err("not in the corpus");
        assert!(error.to_string().contains("unknown field `unit_price`"));
    }
}
