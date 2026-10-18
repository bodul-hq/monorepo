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
    pub price: i64,
    pub weight: i64,
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
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

    /// The `2M` variant of the 100W USB-C cable.
    pub(crate) fn variant() -> Value {
        json!({
            "id": 51557243388247_u64,
            "title": "2M",
            "option1": "2M",
            "option2": null,
            "option3": null,
            "sku": "65833",
            "requires_shipping": true,
            "taxable": true,
            "featured_image": {
                "id": 68816499278167_u64,
                "product_id": 10080883081559_u64,
                "position": 1,
                "created_at": "2025-10-17T18:37:14+08:00",
                "updated_at": "2025-10-17T20:05:32+08:00",
                "alt": "UGREEN USB - C Kabel ( 100W ) mit E - Marker - Kabel",
                "width": 1200,
                "height": 1600,
                "src": "//de.ugreen.com/cdn/shop/files/71QIYZFjMgL.png?v=1760702732",
                "variant_ids": [51557243388247_u64]
            },
            "available": true,
            "name": "UGREEN USB-C Kabel ( 100W ) mit E-Marker - 2M",
            "public_title": "2M",
            "options": ["2M"],
            "price": 1019,
            "weight": 0,
            "compare_at_price": 1099,
            "inventory_management": "shopify",
            "barcode": null,
            "featured_media": {
                "alt": "UGREEN USB - C Kabel ( 100W ) mit E - Marker - Kabel",
                "id": 57459611828567_u64,
                "position": 1,
                "preview_image": {
                    "aspect_ratio": 0.75,
                    "height": 1600,
                    "width": 1200,
                    "src": "//de.ugreen.com/cdn/shop/files/71QIYZFjMgL.png?v=1760702732"
                }
            },
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    #[test]
    fn a_variant_without_its_own_image_reads_null() {
        let mut input = variant();
        input["featured_image"] = Value::Null;
        input["featured_media"] = Value::Null;

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
