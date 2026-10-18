use html_extractor::serde_json::Value;

/// The variant picker's record of the selected variant, on the 44 pages with a picker.
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
    /// Empty on the gift cards.
    pub sku: String,
    pub requires_shipping: bool,
    pub taxable: bool,
    pub featured_image: Option<FeaturedImage>,
    pub available: bool,
    pub name: String,
    pub public_title: Option<String>,
    pub options: Vec<String>,
    pub price: i64,
    pub weight: i64,
    /// Null where the variant has no compare-at price (1 of the 44 records).
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
    /// Null on 17 of the 44 records; may end in a no-break space, as the ld+json `gtin` does.
    pub barcode: Option<String>,
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

    /// The `Chargeur` variant of `products-45699.html`.
    pub(crate) fn variant() -> Value {
        json!({
            "id": 50354801180992_u64,
            "title": "Chargeur",
            "option1": "Chargeur",
            "option2": null,
            "option3": null,
            "sku": "45699",
            "requires_shipping": true,
            "taxable": true,
            "featured_image": null,
            "available": true,
            "name": "UGREEN 300W Nexode Chargeur - Chargeur",
            "public_title": "Chargeur",
            "options": ["Chargeur"],
            "price": 13499,
            "weight": 0,
            "compare_at_price": 17999,
            "inventory_management": "shopify",
            "barcode": "6941876246991",
            "featured_media": null,
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    #[test]
    fn the_record_deserializes() {
        let variant = serde_json::from_value::<Variant>(variant()).expect("deserializes");
        assert_eq!(variant.compare_at_price, Some(17999));
    }

    #[test]
    fn a_variant_with_its_own_image_deserializes() {
        let mut input = variant();
        input["featured_image"] = json!({
            "id": 1, "product_id": 9526794977600_u64, "position": 1,
            "created_at": "2025-02-06T17:39:31+08:00", "updated_at": "2025-02-06T17:39:32+08:00",
            "alt": null, "width": 1600, "height": 1600,
            "src": "//fr.ugreen.com/cdn/shop/files/a.png?v=1", "variant_ids": [50354801180992_u64]
        });
        input["featured_media"] = json!({
            "alt": null, "id": 2, "position": 1,
            "preview_image": {"aspect_ratio": 1.0, "height": 1600, "width": 1600,
                "src": "//fr.ugreen.com/cdn/shop/files/a.png?v=1"}
        });

        let variant = serde_json::from_value::<Variant>(input).expect("deserializes");
        assert!(variant.featured_image.is_some() && variant.featured_media.is_some());
    }

    #[test]
    fn a_null_barcode_and_compare_at_price_read_none() {
        let mut input = variant();
        input["barcode"] = Value::Null;
        input["compare_at_price"] = Value::Null;

        let variant = serde_json::from_value::<Variant>(input).expect("deserializes");
        assert!(variant.barcode.is_none() && variant.compare_at_price.is_none());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = variant();
        input["unit_price"] = json!(1);

        let error = serde_json::from_value::<Variant>(input).expect_err("not in the corpus");
        assert!(error.to_string().contains("unknown field `unit_price`"));
    }
}
