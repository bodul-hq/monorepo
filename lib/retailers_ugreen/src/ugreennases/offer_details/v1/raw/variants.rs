use html_extractor::serde_json::Value;

/// The picker's `script[type="application/json"]` inside `variant-selects`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantsScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: Vec<Variant>,
}

/// One entry of Liquid's `product.variants | json`. Every amount is in hundredths of the shop
/// currency, which it does not name.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    pub sku: String,
    pub requires_shipping: bool,
    pub taxable: bool,
    pub featured_image: Option<FeaturedImage>,
    pub available: bool,
    pub name: String,
    pub public_title: String,
    pub options: Vec<String>,
    pub price: i64,
    pub weight: i64,
    /// Null where the variant is not marked down.
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
    /// Null on 19 of the 34 saved variants; a 13-digit EAN on the rest.
    pub barcode: Option<String>,
    /// Only rendered alongside a `featured_image`.
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
    /// Protocol-relative, without a display width.
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
    use super::{Variant, VariantsScript};
    use html_extractor::serde_json::{self, Value, json};

    /// A picker record, trimmed from `products-ugreen-nasync-dxp2800-gt-2-bay-nas-storage.html`.
    pub(crate) fn variant(
        id: u64,
        title: &str,
        sku: &str,
        price: i64,
        compare_at: i64,
        barcode: Option<&str>,
    ) -> Value {
        json!({
            "id": id,
            "title": title,
            "option1": title,
            "option2": null,
            "option3": null,
            "sku": sku,
            "requires_shipping": true,
            "taxable": true,
            "featured_image": {
                "id": 56466475647258_u64,
                "product_id": 10254048985370_u64,
                "position": 1,
                "created_at": "2026-06-05T14:55:09+08:00",
                "updated_at": "2026-06-05T14:55:11+08:00",
                "alt": null,
                "width": 2400,
                "height": 2400,
                "src": "//ai-es.ugreen.com/cdn/shop/files/DXP2800_GT-16051-nuevo.webp?v=1780642511",
                "variant_ids": [id]
            },
            "available": true,
            "name": format!("UGREEN NASync DXP2800 GT - {title}"),
            "public_title": title,
            "options": [title],
            "price": price,
            "weight": 0,
            "compare_at_price": compare_at,
            "inventory_management": "shopify",
            "barcode": barcode,
            "featured_media": {
                "alt": null,
                "id": 45443724345626_u64,
                "position": 1,
                "preview_image": {
                    "aspect_ratio": 1.0,
                    "height": 2400,
                    "width": 2400,
                    "src": "//ai-es.ugreen.com/cdn/shop/files/DXP2800_GT-16051-nuevo.webp?v=1780642511"
                }
            },
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    /// The picker of `super::super::analytics::tests::meta()`'s product.
    pub(crate) fn script() -> Value {
        json!({
            "type": "application/json",
            "data": [
                variant(52296690008346, "DXP2800 GT (Envío desde el 27 de octubre)", "16051", 45999, 50999,
                    Some("6942752010514")),
                variant(52640999375130, "DXP2800 GT + 120W DC UPS (Envío desde el 27 de octubre)", "16051+45615", 48999,
                    61999, None)
            ]
        })
    }

    #[test]
    fn the_script_deserializes() {
        let script = serde_json::from_value::<VariantsScript>(script()).expect("deserializes");
        assert_eq!(script.data.len(), 2);
    }

    #[test]
    fn a_record_without_an_image_or_markdown_deserializes() {
        let mut input = script()["data"][0].clone();
        input["featured_image"] = Value::Null;
        input
            .as_object_mut()
            .expect("an object")
            .remove("featured_media");
        input["compare_at_price"] = Value::Null;

        let variant = serde_json::from_value::<Variant>(input).expect("deserializes");
        assert!(variant.featured_image.is_none());
        assert!(variant.compare_at_price.is_none());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = script();
        input["data"][0]["unit_price"] = json!(1);
        assert!(serde_json::from_value::<VariantsScript>(input).is_err());

        let mut input = script();
        input["id"] = json!("variants");
        assert!(serde_json::from_value::<VariantsScript>(input).is_err());
    }
}
