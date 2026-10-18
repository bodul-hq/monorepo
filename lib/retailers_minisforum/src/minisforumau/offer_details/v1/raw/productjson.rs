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
    /// Missing on the one saved page whose product is announced without a price
    /// (`products-minisforum-ai-nas-s5-304.html`), which renders no media at all.
    #[serde(default)]
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

/// Every one of the 290 media entries on the saved pages is an image; `minisforumuk`'s product
/// video appears on none, so a video fails here.
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

#[cfg(test)]
pub(crate) mod tests {
    use super::{Media, ProductJson};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn variant(id: u64, price: i64, compare_at: i64, sku: &str, kind: &str) -> Value {
        let title = format!("Intel® Core™ Ultra 5 336H / {kind}");
        json!({
            "id": id,
            "title": title,
            "option1": "Intel® Core™ Ultra 5 336H",
            "option2": kind,
            "option3": null,
            "sku": sku,
            "requires_shipping": true,
            "taxable": true,
            "featured_image": null,
            "available": true,
            "name": format!("MINISFORUM MS-03 - {title}"),
            "public_title": title,
            "options": ["Intel® Core™ Ultra 5 336H", kind],
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

    /// Trimmed from `products-minisforum-ms-03-workstation.html`.
    pub(crate) fn product() -> Value {
        json!({
            "id": 8313879462003_u64,
            "title": "MINISFORUM MS-03",
            "handle": "minisforum-ms-03-workstation",
            "description": "<p>d</p>",
            "published_at": "2026-07-08T11:12:15+08:00",
            "created_at": "2026-07-08T11:12:15+08:00",
            "vendor": "Au Minisforum",
            "type": "Work Station",
            "tags": ["Intel", "Work Station"],
            "price": 114290,
            "price_min": 114290,
            "price_max": 202290,
            "available": true,
            "price_varies": true,
            "compare_at_price": 142890,
            "compare_at_price_min": 142890,
            "compare_at_price_max": 252890,
            "compare_at_price_varies": true,
            "variants": [
                variant(45188354441331, 114290, 142890, "MS-03-33600AU", "Barebone"),
                variant(45270950707315, 202290, 252890, "MS-03-33631AU", "32GB RAM+1TB SSD")
            ],
            "images": ["//au.minisforum.com/cdn/shop/files/MinisforumMS-03.png?v=1784715204"],
            "featured_image": "//au.minisforum.com/cdn/shop/files/MinisforumMS-03.png?v=1784715204",
            "options": ["CPU", "Type"],
            "media": [{
                "alt": null, "id": 31742027694195_u64, "position": 1, "media_type": "image",
                "preview_image": {"aspect_ratio": 1.0, "height": 5000, "width": 5000,
                    "src": "//au.minisforum.com/cdn/shop/files/MinisforumMS-03.png?v=1784715204"},
                "aspect_ratio": 1.0, "height": 5000, "width": 5000,
                "src": "//au.minisforum.com/cdn/shop/files/MinisforumMS-03.png?v=1784715204"
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
    fn a_record_without_media_deserializes() {
        let mut input = product();
        input.as_object_mut().expect("an object").remove("media");

        let product = serde_json::from_value::<ProductJson>(input).expect("deserializes");
        assert!(product.media.is_empty());
    }

    #[test]
    fn an_unseen_media_type_fails() {
        for media_type in ["video", "model"] {
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
