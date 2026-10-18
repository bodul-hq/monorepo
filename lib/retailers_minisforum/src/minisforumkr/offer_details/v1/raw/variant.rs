use html_extractor::serde_json::Value;

/// The variant picker's record of the selected variant: the variant the page opens on.
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
    /// Rendered only alongside a `featured_image`.
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

    /// The UM890 Pro's first variant, as the picker records it.
    pub(crate) fn variant() -> Value {
        json!({
            "id": 41332824080430_u64,
            "title": "AMD Ryzen™ 9 8945HS / FreeDOS / EU",
            "option1": "AMD Ryzen™ 9 8945HS",
            "option2": "FreeDOS",
            "option3": "EU",
            "sku": "UM890Pro00EU",
            "requires_shipping": true,
            "taxable": true,
            "featured_image": {
                "id": 31880606285870_u64,
                "product_id": 7128100143150_u64,
                "position": 1,
                "created_at": "2025-01-21T15:16:55+09:00",
                "updated_at": "2025-04-28T17:46:05+09:00",
                "alt": null,
                "width": 800,
                "height": 800,
                "src": "//www.minisforum.kr/cdn/shop/files/UM890Pro1.jpg?v=1776075578",
                "variant_ids": [41332824080430_u64, 41332824113198_u64]
            },
            "available": true,
            "name": "MINISFORUM UM890 Pro 고성능 미니PC - AMD Ryzen™ 9 8945HS / FreeDOS / EU",
            "public_title": "AMD Ryzen™ 9 8945HS / FreeDOS / EU",
            "options": ["AMD Ryzen™ 9 8945HS", "FreeDOS", "EU"],
            "price": 75690000,
            "weight": 1700,
            "compare_at_price": 89999000,
            "inventory_management": "shopify",
            "barcode": null,
            "featured_media": {
                "alt": null,
                "id": 24041626337326_u64,
                "position": 1,
                "preview_image": {
                    "aspect_ratio": 1.0,
                    "height": 800,
                    "width": 800,
                    "src": "//www.minisforum.kr/cdn/shop/files/UM890Pro1.jpg?v=1776075578"
                }
            },
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    #[test]
    fn a_variant_without_an_image_renders_no_media() {
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
