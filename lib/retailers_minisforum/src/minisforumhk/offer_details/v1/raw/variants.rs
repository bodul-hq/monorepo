use html_extractor::serde_json::Value;

/// `script#bm_product_variants`: every variant of the product, as a JSON array.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantsScript {
    pub id: String,
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: Vec<Variant>,
}

/// `script#bm_product_selected_or_first_available_variant`: the variant the page opens on, the
/// one in the URL or else the first available. It equals its entry in the list on all 29 saved
/// pages.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariantScript {
    pub id: String,
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: Variant,
}

/// Liquid's `variant | json`. Every amount in it is in hundredths of the shop currency, which it
/// does not name. 78 variants on the 29 saved pages.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    /// The selected values joined with ` / `, or `Default Title` where the product has no
    /// options to choose (3 pages).
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    /// Empty on 1 of the 78 saved variants (`products-運費補價.html`).
    pub sku: Option<String>,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Present on 2 of the 78 saved variants, both of `products-um760-slim.html`.
    pub featured_image: Option<FeaturedImage>,
    pub available: bool,
    /// The product title and `title`, joined with ` - `; the product title alone where `title`
    /// is `Default Title`.
    pub name: String,
    /// `null` exactly where `title` is `Default Title`.
    pub public_title: Option<String>,
    pub options: Vec<String>,
    pub price: i64,
    pub weight: i64,
    pub compare_at_price: Option<i64>,
    /// `null` on 1 of the 78 saved variants (`products-運費補價.html`).
    pub inventory_management: Option<String>,
    /// `null` on all 78 saved variants.
    pub barcode: Option<String>,
    /// Only rendered alongside a `featured_image`.
    #[serde(default)]
    pub featured_media: Option<FeaturedMedia>,
    pub requires_selling_plan: bool,
    /// Empty on every saved variant.
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
    use super::{SelectedVariantScript, Variant, VariantsScript};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn variant(
        id: u64,
        price: i64,
        compare_at: i64,
        sku: &str,
        options: [&str; 2],
    ) -> Value {
        let title = options.join(" / ");
        json!({
            "id": id,
            "title": title,
            "option1": options[0],
            "option2": options[1],
            "option3": null,
            "sku": sku,
            "requires_shipping": true,
            "taxable": true,
            "featured_image": null,
            "available": true,
            "name": format!("MS-S1 Max - {title}"),
            "public_title": title,
            "options": options,
            "price": price,
            "weight": 0,
            "compare_at_price": compare_at,
            "inventory_management": "shopify",
            "barcode": null,
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    /// Trimmed from `products-minisforum-ms-s1-max.html`.
    pub(crate) fn variants() -> Value {
        json!([
            variant(
                46559370805426,
                3299900,
                3499900,
                "CS-MMSS1MD",
                ["AMD Ryzen™ AI Max+ 395", "128GB + 2TB SSD"]
            ),
            variant(
                67586234319026,
                5499900,
                5699900,
                "CS-MMSS14D",
                ["AMD Ryzen™ AI Max+ PRO 495", "192GB + 2TB SSD"]
            )
        ])
    }

    #[test]
    fn the_variants_deserialize() {
        let variants = serde_json::from_value::<Vec<Variant>>(variants()).expect("deserializes");
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].options.len(), 2);
    }

    #[test]
    fn a_variant_with_an_image_deserializes() {
        let mut input = variant(1, 100, 100, "A", ["a", "b"]);
        input["featured_image"] = json!({
            "id": 39293195354290_u64, "product_id": 8296452718770_u64, "position": 2,
            "created_at": "2024-09-03T14:44:41+08:00", "updated_at": "2024-09-03T14:44:42+08:00",
            "alt": null, "width": 4000, "height": 4000,
            "src": "//minisforum.hk/cdn/shop/files/009.jpg?v=1725345882",
            "variant_ids": [1]
        });
        input["featured_media"] = json!({
            "alt": null, "id": 31645141565618_u64, "position": 2,
            "preview_image": {"aspect_ratio": 1.0, "height": 4000, "width": 4000,
                "src": "//minisforum.hk/cdn/shop/files/009.jpg?v=1725345882"}
        });

        let variant = serde_json::from_value::<Variant>(input).expect("deserializes");
        assert!(variant.featured_image.is_some() && variant.featured_media.is_some());
    }

    #[test]
    fn a_new_key_in_a_variant_fails() {
        let mut input = variant(1, 100, 100, "A", ["a", "b"]);
        input["unit_price"] = json!(null);

        assert!(serde_json::from_value::<Variant>(input).is_err());
    }

    #[test]
    fn a_new_script_attribute_fails() {
        let list = json!({
            "id": "bm_product_variants", "type": "application/json", "data-x": "1", "data": variants()
        });
        assert!(serde_json::from_value::<VariantsScript>(list).is_err());

        let selected = json!({
            "id": "bm_product_selected_or_first_available_variant",
            "type": "application/json", "data-x": "1", "data": variant(1, 100, 100, "A", ["a", "b"])
        });
        assert!(serde_json::from_value::<SelectedVariantScript>(selected).is_err());
    }
}
