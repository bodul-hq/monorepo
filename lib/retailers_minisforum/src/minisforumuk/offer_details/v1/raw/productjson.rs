use html_extractor::serde_json::Value;

/// `script#xcotton_pp_variants`, which carries the app's consent-manager opt-outs as attributes.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductJsonScript {
    pub id: String,
    #[serde(rename = "type")]
    pub script_type: String,
    #[serde(rename = "data-cmp-ab")]
    pub cmp_ab: String,
    #[serde(rename = "data-cookieconsent")]
    pub cookieconsent: String,
    #[serde(rename = "data-ot-ignore")]
    pub ot_ignore: String,
    #[serde(rename = "data-uc-allowed")]
    pub uc_allowed: String,
    #[serde(rename = "data-ta-type")]
    pub ta_type: String,
    pub data: ProductJson,
}

/// Liquid's `product | json`. Every amount in it is in hundredths of the shop currency, which
/// it does not name.
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
    pub selling_plan_groups: Vec<Value>,
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

    pub(crate) fn variant(id: u64, price: i64, sku: &str, memory: &str) -> Value {
        let title = format!("AMD Ryzen™ AI 9 HX 370 / {memory} / UK Warehouse");
        json!({
            "id": id,
            "title": title,
            "option1": "AMD Ryzen™ AI 9 HX 370",
            "option2": memory,
            "option3": "UK Warehouse",
            "sku": sku,
            "requires_shipping": true,
            "taxable": true,
            "featured_image": null,
            "available": true,
            "name": format!("Minisforum AI X1 Pro-370 - {title}"),
            "public_title": title,
            "options": ["AMD Ryzen™ AI 9 HX 370", memory, "UK Warehouse"],
            "price": price,
            "weight": 0,
            "compare_at_price": null,
            "inventory_management": "shopify",
            "barcode": null,
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    /// Trimmed from `products-ai-x1-pro.html`.
    pub(crate) fn product() -> Value {
        json!({
            "id": 9875989692694_u64,
            "title": "Minisforum AI X1 Pro-370",
            "handle": "ai-x1-pro",
            "description": "<p>d</p>",
            "published_at": "2025-02-27T17:09:05+08:00",
            "created_at": "2025-02-27T09:55:09+08:00",
            "vendor": "Minisforum UK",
            "type": "Mini PC",
            "tags": ["AI MINIPC", "UK Warehouse"],
            "price": 63900,
            "price_min": 63900,
            "price_max": 107900,
            "available": true,
            "price_varies": true,
            "compare_at_price": null,
            "compare_at_price_min": 0,
            "compare_at_price_max": 0,
            "compare_at_price_varies": false,
            "variants": [
                variant(50279469515030, 63900, "X1PRO00UK", "Barebone (Not include os license)"),
                variant(49777802674454, 107900, "X1PRO31UK", "32GB RAM + 1TB SSD")
            ],
            "images": ["//www.minisforum.uk/cdn/shop/files/ai-x1-pro-mini-pc-1.png?v=1787902570"],
            "featured_image": "//www.minisforum.uk/cdn/shop/files/ai-x1-pro-mini-pc-1.png?v=1787902570",
            "options": ["CPU", "Type", "Dispatch warehouse"],
            "media": [{
                "alt": null, "id": 45974314942742_u64, "position": 1, "media_type": "image",
                "preview_image": {"aspect_ratio": 1.0, "height": 1600, "width": 1600,
                    "src": "//www.minisforum.uk/cdn/shop/files/ai-x1-pro-mini-pc-1.png?v=1787902570"},
                "aspect_ratio": 1.0, "height": 1600, "width": 1600,
                "src": "//www.minisforum.uk/cdn/shop/files/ai-x1-pro-mini-pc-1.png?v=1787902570"
            }],
            "requires_selling_plan": false,
            "selling_plan_groups": [],
            "content": "<p>d</p>"
        })
    }

    #[test]
    fn the_record_deserializes() {
        let product = serde_json::from_value::<ProductJson>(product()).expect("deserializes");
        assert_eq!(product.variants.len(), 2);
    }

    #[test]
    fn a_video_media_deserializes_and_an_unseen_type_fails() {
        let video = json!({"media_type": "video", "alt": null, "id": 1, "position": 2,
            "preview_image": {"aspect_ratio": 1.0, "height": 1, "width": 1, "src": "s"},
            "aspect_ratio": 1.0, "duration": 36000,
            "sources": [{"format": "mp4", "height": 480, "mime_type": "video/mp4",
                "url": "//u", "width": 268}]});
        serde_json::from_value::<Media>(video).expect("videos are typed");

        let model = json!({"media_type": "model", "alt": null, "id": 1, "position": 1});
        let error = serde_json::from_value::<Media>(model).expect_err("no model media is typed");
        assert!(error.to_string().contains("unknown variant `model`"));
    }
}
