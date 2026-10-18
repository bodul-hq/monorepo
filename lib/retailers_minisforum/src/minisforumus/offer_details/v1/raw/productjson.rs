use html_extractor::serde_json::Value;

/// `script#xcotton_pp_variants`: an app's rendering of Liquid's `product | json`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductJsonItem {
    pub data: ProductJson,
}

/// Every amount in it is in hundredths of the market's currency, which it does not name.
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
    pub sku: Option<String>,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Null on every variant so far, so its shape is not typed.
    pub featured_image: Option<Value>,
    pub available: bool,
    pub name: String,
    pub public_title: Option<String>,
    pub options: Vec<String>,
    pub price: i64,
    pub weight: i64,
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
    pub barcode: Option<String>,
    pub requires_selling_plan: bool,
    pub selling_plan_allocations: Vec<SellingPlanAllocation>,
    pub quantity_rule: QuantityRule,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityRule {
    pub min: i64,
    pub max: Option<i64>,
    pub increment: i64,
}

/// A variant's price under one of the product's selling plans.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanAllocation {
    pub price_adjustments: Vec<Value>,
    pub price: i64,
    pub compare_at_price: Option<i64>,
    pub per_delivery_price: i64,
    pub selling_plan_id: u64,
    pub selling_plan_group_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanGroup {
    pub id: String,
    pub name: String,
    pub options: Vec<SellingPlanGroupOption>,
    pub selling_plans: Vec<SellingPlan>,
    pub app_id: Option<String>,
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
    pub price_adjustments: Vec<Value>,
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
pub struct CheckoutCharge {
    pub value_type: String,
    pub value: i64,
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
    Video {
        alt: Option<String>,
        id: u64,
        position: i64,
        preview_image: PreviewImage,
        aspect_ratio: f64,
        duration: i64,
        sources: Vec<VideoSource>,
    },
    ExternalVideo {
        alt: Option<String>,
        id: u64,
        position: i64,
        preview_image: PreviewImage,
        aspect_ratio: f64,
        external_id: String,
        host: String,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoSource {
    pub format: String,
    pub height: i64,
    pub mime_type: String,
    pub url: String,
    pub width: i64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Media, ProductJson};
    use html_extractor::serde_json::{self, Value, json};

    /// An `Intel® Core™ i9-12900H / <memory> / US` variant of the MS-01 SE.
    pub(crate) fn variant(id: u64, price: i64, memory: &str, sku: &str) -> Value {
        let title = format!("Intel® Core™ i9-12900H / {memory} / US");
        json!({
            "id": id,
            "title": title,
            "option1": "Intel® Core™ i9-12900H",
            "option2": memory,
            "option3": "US",
            "sku": sku,
            "requires_shipping": true,
            "taxable": true,
            "featured_image": null,
            "available": true,
            "name": format!("MINISFORUM MS-01 SE - {title}"),
            "public_title": title,
            "options": ["Intel® Core™ i9-12900H", memory, "US"],
            "price": price,
            "weight": 0,
            "compare_at_price": 125930,
            "inventory_management": "shopify",
            "barcode": null,
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    pub(crate) fn product() -> Value {
        json!({
            "id": 9093286985973_u64,
            "title": "MINISFORUM MS-01 SE",
            "handle": "minisforum-ms-01-se-workstation",
            "description": "<p>MINISFORUM MS-01 SE Workstation</p>",
            "published_at": "2025-12-04T09:12:22+08:00",
            "created_at": "2023-12-13T23:24:19+08:00",
            "vendor": "MINISFORUM US",
            "type": "Work Station",
            "tags": ["active", "All", "Intel"],
            "price": 54343,
            "price_min": 54343,
            "price_max": 112308,
            "available": true,
            "price_varies": true,
            "compare_at_price": 125930,
            "compare_at_price_min": 125930,
            "compare_at_price_max": 125930,
            "compare_at_price_varies": false,
            "variants": [
                variant(46589796385013, 54343, "Barebone", "MS-01SE00US"),
                variant(47590282297589, 112308, "32GB RAM+1TB SSD", "MS-01SE31US")
            ],
            "images": [
                "//store.minisforum.com/cdn/shop/files/MS-01.png?v=1",
                "//store.minisforum.com/cdn/shop/files/MS-01-2.png?v=1"
            ],
            "featured_image": "//store.minisforum.com/cdn/shop/files/MS-01.png?v=1",
            "options": ["CPU", "Type", "Adapter"],
            "media": [{
                "alt": "MINISFORUM MS-01 SE MINISFORUM US", "id": 32689116446965_u64,
                "position": 1, "media_type": "image",
                "preview_image": {"aspect_ratio": 1.0, "height": 1600, "width": 1600,
                    "src": "//store.minisforum.com/cdn/shop/files/MS-01.png?v=1"},
                "aspect_ratio": 1.0, "height": 1600, "width": 1600,
                "src": "//store.minisforum.com/cdn/shop/files/MS-01.png?v=1"
            }],
            "requires_selling_plan": false,
            "selling_plan_groups": [],
            "content": "<p>MINISFORUM MS-01 SE Workstation</p>"
        })
    }

    /// The store's only selling plan group, as `minisforum-um890pro-mini-pc` renders it.
    pub(crate) fn preorder_group(plan_id: u64) -> Value {
        json!({
            "id": "6c17fabdef8f8d1808aafff6115bf384c07f7045",
            "name": "Preorder",
            "options": [{"name": "Preorder", "position": 1, "values": ["Full payment"]}],
            "selling_plans": [{
                "id": plan_id, "name": "Preorder", "description": null,
                "options": [{"name": "Preorder", "position": 1, "value": "Full payment"}],
                "recurring_deliveries": false, "price_adjustments": [],
                "checkout_charge": {"value_type": "percentage", "value": 100}
            }],
            "app_id": null
        })
    }

    #[test]
    fn the_record_deserializes() {
        let product = serde_json::from_value::<ProductJson>(product()).expect("deserializes");
        assert_eq!(product.variants.len(), 2);
    }

    #[test]
    fn a_preorder_group_deserializes() {
        let mut input = product();
        input["selling_plan_groups"] = json!([preorder_group(10215063797)]);

        serde_json::from_value::<ProductJson>(input).expect("selling plans are typed");
    }

    #[test]
    fn an_unseen_media_type_fails() {
        let model = json!({"media_type": "model", "alt": null, "id": 1, "position": 1,
            "preview_image": {"aspect_ratio": 1.0, "height": 1, "width": 1, "src": "s"},
            "aspect_ratio": 1.0});

        let error = serde_json::from_value::<Media>(model).expect_err("no model media is typed");
        assert!(error.to_string().contains("unknown variant `model`"));
    }
}
