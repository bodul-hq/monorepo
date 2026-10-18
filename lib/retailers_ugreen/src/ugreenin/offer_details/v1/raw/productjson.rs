/// `script[name="BSPD-SHOPIFY-VARS"]`, the Bitespeed app embed's script, which assigns Liquid's
/// `product | json` to `BspdCurrentProduct`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductJsonScript {
    pub name: String,
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: ProductJson,
}

/// Liquid's `product | json`. Every amount in it is in hundredths of the shop currency (paise),
/// which it does not name. The Hindi pages render the same record with the title, description,
/// product type, tags, option names and variant names in Hindi where the shop has translated
/// them.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductJson {
    pub id: u64,
    pub title: String,
    pub handle: String,
    pub description: String,
    pub published_at: String,
    pub created_at: String,
    pub vendor: String,
    #[serde(rename = "type")]
    pub product_type: String,
    pub tags: Vec<String>,
    pub price: i64,
    pub price_min: i64,
    pub price_max: i64,
    pub available: bool,
    pub price_varies: bool,
    /// Null on the 4 products per locale that are not marked down.
    pub compare_at_price: Option<i64>,
    pub compare_at_price_min: i64,
    pub compare_at_price_max: i64,
    pub compare_at_price_varies: bool,
    pub variants: Vec<Variant>,
    pub images: Vec<String>,
    pub featured_image: Option<String>,
    pub options: Vec<String>,
    pub media: Vec<Media>,
    pub requires_selling_plan: bool,
    /// A pre-order plan on 5 products per locale; empty on the rest.
    pub selling_plan_groups: Vec<SellingPlanGroup>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    /// Set on every variant of the saved pages.
    pub sku: Option<String>,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Null on every variant of the saved pages.
    pub featured_image: Option<FeaturedImage>,
    pub available: bool,
    pub name: String,
    pub public_title: Option<String>,
    pub options: Vec<String>,
    pub price: i64,
    /// Unit unknown: Liquid documents grams. 0 on 286 of the 1,422 saved variants.
    pub weight: i64,
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
    /// 13 digits, 12 digits, null, or a spreadsheet-mangled `6.96E+12` / `6.94E+12`.
    pub barcode: Option<String>,
    pub requires_selling_plan: bool,
    pub selling_plan_allocations: Vec<SellingPlanAllocation>,
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

/// Every one of the 10,142 media entries on the saved pages is an image, so a video fails here.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "media_type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Media {
    Image {
        alt: Option<String>,
        id: u64,
        position: i64,
        preview_image: PreviewImage,
        aspect_ratio: f64,
        height: i64,
        width: i64,
        src: String,
    },
}

/// The pre-order app's plan (`amp-preorder`): "Full upfront payment, ships 15 days after
/// checkout", with a 0% price adjustment.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanGroup {
    pub id: String,
    pub name: String,
    pub app_id: String,
    pub options: Vec<SellingPlanGroupOption>,
    pub selling_plans: Vec<SellingPlan>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanGroupOption {
    pub name: String,
    pub position: i64,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlan {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub options: Vec<SellingPlanOption>,
    pub recurring_deliveries: bool,
    pub price_adjustments: Vec<SellingPlanPriceAdjustment>,
    pub checkout_charge: CheckoutCharge,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanOption {
    pub name: String,
    pub position: i64,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanPriceAdjustment {
    pub order_count: Option<i64>,
    pub position: i64,
    pub value_type: String,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckoutCharge {
    pub value_type: String,
    pub value: i64,
}

/// A variant's price under a selling plan, in paise.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanAllocation {
    pub price_adjustments: Vec<AllocationPriceAdjustment>,
    pub price: i64,
    pub compare_at_price: i64,
    pub per_delivery_price: i64,
    pub selling_plan_id: u64,
    pub selling_plan_group_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationPriceAdjustment {
    pub position: i64,
    pub price: i64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Media, ProductJson, ProductJsonScript};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn variant(id: u64, sku: &str, colour: &str, barcode: Option<&str>) -> Value {
        json!({
            "available": true,
            "barcode": barcode,
            "compare_at_price": 222000,
            "featured_image": null,
            "id": id,
            "inventory_management": "shopify",
            "name": format!("UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest - {colour}"),
            "option1": colour,
            "option2": null,
            "option3": null,
            "options": [colour],
            "price": 99900,
            "public_title": colour,
            "quantity_rule": {"increment": 1, "max": null, "min": 1},
            "requires_selling_plan": false,
            "requires_shipping": true,
            "selling_plan_allocations": [],
            "sku": sku,
            "taxable": true,
            "title": colour,
            "weight": 0
        })
    }

    /// Trimmed from `products-ugreen-ergonomic-mouse-pad-with-wrist-rest-…-black-25245.html`,
    /// with the handle shortened.
    pub(crate) fn product() -> Value {
        let image = "//www.ugreenindia.com/cdn/shop/files/LP668-Black.png?v=1763832672";
        json!({
            "available": true,
            "compare_at_price": 222000,
            "compare_at_price_max": 222000,
            "compare_at_price_min": 222000,
            "compare_at_price_varies": false,
            "content": "<p>d</p>",
            "created_at": "2024-12-12T16:19:10+05:30",
            "description": "<p>d</p>",
            "featured_image": image,
            "handle": "ugreen-ergonomic-mouse-pad-black-25245",
            "id": 7702424813667_u64,
            "images": [image],
            "media": [{
                "alt": "UGREEN LP668 25245 ergonomic mouse pad with wrist rest black",
                "aspect_ratio": 1.0, "height": 800, "id": 26084735352931_u64,
                "media_type": "image", "position": 1,
                "preview_image": {"aspect_ratio": 1.0, "height": 800, "src": image, "width": 800},
                "src": image, "width": 800
            }],
            "options": ["Color"],
            "price": 99900,
            "price_max": 99900,
            "price_min": 99900,
            "price_varies": false,
            "published_at": "2024-12-12T16:26:41+05:30",
            "requires_selling_plan": false,
            "selling_plan_groups": [],
            "tags": ["Ergonomic Mouse Pad", "Gaming Accessories"],
            "title": "UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest",
            "type": "Mouse Pad",
            "variants": [
                variant(43454938677347, "25245", "Black", Some("6941876222452")),
                variant(43742608687203, "25243", "Green", None),
                variant(43742608719971, "25244", "Gray", None)
            ],
            "vendor": "UGREEN"
        })
    }

    pub(crate) fn script() -> Value {
        json!({"name": "BSPD-SHOPIFY-VARS", "type": "text/javascript", "data": product()})
    }

    /// As on `products-ugreen-laptop-carry-bag-for-14-14-9-inch-…-50337.html`.
    fn preorder() -> (Value, Value) {
        let group = json!({
            "app_id": "amp-preorder",
            "id": "874ae572ca3d46ead8a08a0325460eb7c3af70c0",
            "name": "Full upfront payment, ships 15 days after checkout",
            "options": [{"name": "Pay full now", "position": 1, "values": ["Pay full now"]}],
            "selling_plans": [{
                "checkout_charge": {"value": 100, "value_type": "percentage"},
                "description": null,
                "id": 16906125411_u64,
                "name": "Full upfront payment, ships 15 days after checkout",
                "options": [{"name": "Pay full now", "position": 1, "value": "Pay full now"}],
                "price_adjustments": [
                    {"order_count": null, "position": 1, "value": 0, "value_type": "percentage"}
                ],
                "recurring_deliveries": false
            }]
        });
        let allocation = json!({
            "compare_at_price": 99900,
            "per_delivery_price": 99900,
            "price": 99900,
            "price_adjustments": [{"position": 1, "price": 99900}],
            "selling_plan_group_id": "874ae572ca3d46ead8a08a0325460eb7c3af70c0",
            "selling_plan_id": 16906125411_u64
        });
        (group, allocation)
    }

    #[test]
    fn the_script_deserializes() {
        let script = serde_json::from_value::<ProductJsonScript>(script()).expect("deserializes");
        assert_eq!(script.data.variants.len(), 3);
    }

    #[test]
    fn a_new_script_attribute_fails() {
        let mut input = script();
        input["id"] = json!("bspd");

        assert!(serde_json::from_value::<ProductJsonScript>(input).is_err());
    }

    #[test]
    fn a_pre_order_plan_deserializes() {
        let (group, allocation) = preorder();
        let mut input = product();
        input["selling_plan_groups"] = json!([group]);
        input["variants"][0]["selling_plan_allocations"] = json!([allocation]);

        let product = serde_json::from_value::<ProductJson>(input).expect("deserializes");
        assert_eq!(product.selling_plan_groups[0].app_id, "amp-preorder");
        assert_eq!(product.variants[0].selling_plan_allocations[0].price, 99900);
    }

    #[test]
    fn an_unknown_key_in_the_record_fails() {
        let mut input = product();
        input["variants"][0]["featured_media"] = json!(null);

        assert!(serde_json::from_value::<ProductJson>(input).is_err());
    }

    #[test]
    fn an_unseen_media_type_fails() {
        for media_type in ["video", "model", "external_video"] {
            let media = json!({"media_type": media_type, "alt": null, "id": 1, "position": 1});
            let error = serde_json::from_value::<Media>(media).expect_err("only images are typed");
            assert!(
                error
                    .to_string()
                    .contains(&format!("unknown variant `{media_type}`"))
            );
        }
    }
}
