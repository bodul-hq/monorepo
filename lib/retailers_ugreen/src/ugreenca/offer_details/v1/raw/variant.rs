use html_extractor::serde_json::Value;

/// `script.pr_variants_json`: the variant picker's records of every variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantsScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub class: String,
    pub data: Vec<Variant>,
}

/// `script.pr_options_json`: the product's options, named in the page's language.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionsScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub class: String,
    pub data: Vec<ProductOption>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductOption {
    pub name: String,
    /// One-based.
    pub position: usize,
    pub values: Vec<String>,
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
    /// Only rendered where `featured_image` is.
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

    /// A one-option (`Color`) variant of the Nexode 100W charger, without an image of its own.
    pub(crate) fn variant(id: u64, price: i64, color: &str, barcode: &str) -> Value {
        json!({
            "id": id,
            "title": color,
            "option1": color,
            "option2": null,
            "option3": null,
            "sku": "40737",
            "requires_shipping": true,
            "taxable": true,
            "featured_image": null,
            "available": true,
            "name": format!("UGREEN Nexode 100W USB C Charger - {color}"),
            "public_title": color,
            "options": [color],
            "price": price,
            "weight": 265,
            "compare_at_price": null,
            "inventory_management": null,
            "barcode": barcode,
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    #[test]
    fn a_variant_with_its_own_image_carries_it_twice() {
        let mut input = variant(1, 5999, "Black", "");
        input["featured_image"] = json!({"id": 2, "product_id": 3, "position": 1,
            "created_at": "2023-11-03T18:10:02+08:00", "updated_at": "2023-11-03T18:10:02+08:00",
            "alt": null, "width": 800, "height": 800,
            "src": "//ca.ugreen.com/cdn/shop/files/a.png?v=1", "variant_ids": [1]});
        input["featured_media"] = json!({"alt": null, "id": 4, "position": 1,
            "preview_image": {"aspect_ratio": 1.0, "height": 800, "width": 800,
                "src": "//ca.ugreen.com/cdn/shop/files/a.png?v=1"}});

        let variant = serde_json::from_value::<Variant>(input).expect("deserializes");
        assert!(variant.featured_image.is_some() && variant.featured_media.is_some());
    }

    #[test]
    fn a_variant_without_an_image_omits_its_media() {
        let variant =
            serde_json::from_value::<Variant>(variant(1, 5999, "Black", "")).expect("deserializes");
        assert!(variant.featured_media.is_none());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = variant(1, 1, "Black", "");
        input["unit_price"] = Value::Null;

        let error = serde_json::from_value::<Variant>(input).expect_err("not in the corpus");
        assert!(error.to_string().contains("unknown field `unit_price`"));
    }
}
