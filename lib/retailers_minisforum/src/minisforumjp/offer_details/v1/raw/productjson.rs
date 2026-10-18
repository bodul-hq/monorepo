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
/// it does not name, and that holds for yen too: the record renders a ¥130,399 price as
/// `13039900`. Checked on the saved pages against the schema.org offers, which state whole yen
/// (`"130399"`): all 177 offers of the 70 `ProductGroup` islands are exactly one hundredth of
/// their variant's `price`, as is the 1 single-`Product` island, and every one of the 277
/// variant `price`s and `compare_at_price`s divides by 100. The record's integers are therefore
/// not yen; dividing them by 100 gives the yen amount.
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
    /// Present on all 73 saved pages.
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
    /// Grams, per Liquid's documentation; 0 on 266 of the 277 variants.
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

/// The 445 media entries of the saved pages: 422 images, 21 external (YouTube) videos on 14
/// pages and 2 hosted videos on 2 pages. Every image has a `src`; a video has none, only a
/// preview image.
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
    /// A video hosted by Shopify, with one source per rendition.
    Video {
        alt: Option<String>,
        id: u64,
        position: i64,
        preview_image: PreviewImage,
        aspect_ratio: f64,
        /// Milliseconds, as Shopify documents it.
        duration: i64,
        sources: Vec<VideoSource>,
    },
    /// A video hosted elsewhere, `youtube` on all 21.
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
    pub mime_type: String,
    pub url: String,
    pub height: i64,
    pub width: i64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{Media, ProductJson};
    use html_extractor::serde_json::{self, Value, json};

    const PRODUCT_IMAGE: &str =
        "//www.minisforum.jp/cdn/shop/files/MS-01MS-03_NO_BP.webp?v=1787303758";

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
            "name": format!("MINISFORUM MS-03 ローカルAIエッジワークステーション - {title}"),
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

    /// Trimmed from `products-minisforum-ms-03-core-ultra-5-336h-ultra-9-386h.html`; the prices
    /// are hundredths of a yen, as Liquid renders them.
    pub(crate) fn product() -> Value {
        json!({
            "id": 15388676128934_u64,
            "title": "MINISFORUM MS-03 ローカルAIエッジワークステーション",
            "handle": "minisforum-ms-03-core-ultra-5-336h-ultra-9-386h",
            "description": "<p>d</p>",
            "published_at": "2026-07-08T10:43:38+08:00",
            "created_at": "2026-07-06T16:39:55+08:00",
            "vendor": "Minisforum JP",
            "type": "Mini WorkStation",
            "tags": ["Intel", "popular"],
            "price": 13039900,
            "price_min": 13039900,
            "price_max": 24999900,
            "available": true,
            "price_varies": true,
            "compare_at_price": 16299900,
            "compare_at_price_min": 16299900,
            "compare_at_price_max": 30499900,
            "compare_at_price_varies": true,
            "variants": [
                variant(56749131137190, 13039900, 16299900, "MS-03-33600JP", "ベアボーンキット（OSライセンスなし）"),
                variant(56749137494182, 24999900, 30499900, "MS-03-33631JP", "32GB RAM + 1TB SSD")
            ],
            "images": [PRODUCT_IMAGE],
            "featured_image": PRODUCT_IMAGE,
            "options": ["CPU", "TYPE"],
            "media": [image_media(), external_video_media()],
            "requires_selling_plan": false,
            "selling_plan_groups": [],
            "content": "<p>d</p>"
        })
    }

    pub(crate) fn image_media() -> Value {
        json!({
            "alt": null, "id": 69722158334118_u64, "position": 1, "media_type": "image",
            "preview_image": {"aspect_ratio": 1.0, "height": 5000, "width": 5000,
                "src": PRODUCT_IMAGE},
            "aspect_ratio": 1.0, "height": 5000, "width": 5000,
            "src": PRODUCT_IMAGE
        })
    }

    /// As on `products-g1-pro.html`.
    pub(crate) fn external_video_media() -> Value {
        json!({
            "alt": null, "id": 70243443671206_u64, "position": 2, "media_type": "external_video",
            "preview_image": {"aspect_ratio": 1.333, "height": 360, "width": 480,
                "src": "//www.minisforum.jp/cdn/shop/files/preview_images/hqdefault.jpg?v=1789372035"},
            "aspect_ratio": 1.77, "external_id": "Vr2-nYTRW7g", "host": "youtube"
        })
    }

    /// As on `products-minisforum-n5-pro-nas.html`.
    fn video_media() -> Value {
        json!({
            "alt": null, "id": 35488576110758_u64, "position": 3, "media_type": "video",
            "preview_image": {"aspect_ratio": 0.563, "height": 2560, "width": 1440,
                "src": "//www.minisforum.jp/cdn/shop/files/preview_images/287a.thumbnail.0000000000.jpg?v=1755861663"},
            "aspect_ratio": 0.563, "duration": 36000,
            "sources": [{
                "format": "mp4", "height": 480, "mime_type": "video/mp4", "width": 270,
                "url": "//www.minisforum.jp/cdn/shop/videos/c/vp/287a/287a.SD-480p-1.0Mbps-55634411.mp4?v=0"
            }]
        })
    }

    #[test]
    fn the_record_deserializes() {
        let product = serde_json::from_value::<ProductJson>(product()).expect("deserializes");
        assert_eq!(product.variants.len(), 2);
        assert_eq!(product.price, 13039900);
    }

    #[test]
    fn every_media_type_deserializes() {
        let media = [image_media(), external_video_media(), video_media()]
            .map(|media| serde_json::from_value::<Media>(media).expect("deserializes"));

        assert!(matches!(media[0], Media::Image { .. }));
        assert!(matches!(media[1], Media::ExternalVideo { .. }));
        assert!(matches!(media[2], Media::Video { .. }));
    }

    #[test]
    fn a_record_without_media_fails() {
        let mut input = product();
        input.as_object_mut().expect("an object").remove("media");

        assert!(serde_json::from_value::<ProductJson>(input).is_err());
    }

    #[test]
    fn an_unseen_media_type_fails() {
        for media_type in ["model", "3d_model"] {
            let media = json!({"media_type": media_type, "alt": null, "id": 1, "position": 1});
            let error = serde_json::from_value::<Media>(media).expect_err("the type is not typed");
            assert!(
                error
                    .to_string()
                    .contains(&format!("unknown variant `{media_type}`"))
            );
        }
    }

    #[test]
    fn an_unknown_key_on_a_media_entry_fails() {
        let mut media = video_media();
        media["transcript"] = json!("x");

        assert!(serde_json::from_value::<Media>(media).is_err());
    }
}
