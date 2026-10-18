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
/// it does not name. The `/products/` and the `/de/products/` page of a product render the same
/// record, translated: ids, handle, prices, SKUs and availability are equal across all 137
/// pairs of saved pages.
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
    /// The product's images and, on 8 pages, a video; see [`Media`].
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
    pub option1: String,
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

/// The product's media, in three shapes: 1938 images, 6 hosted videos and 2 videos hosted on
/// YouTube, the 8 videos on 8 pages. `images` on the product is exactly the images' `src`s, in
/// order, on every page.
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
    /// Shopify-hosted, with its renditions.
    Video {
        alt: Option<String>,
        id: u64,
        position: i64,
        preview_image: PreviewImage,
        aspect_ratio: f64,
        /// In milliseconds.
        duration: u64,
        sources: Vec<VideoSource>,
    },
    /// Hosted elsewhere, as `host` (`youtube`) and its `external_id`; it has no dimensions.
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

    pub(crate) fn variant(id: u64, price: i64, compare_at: i64, sku: &str, kind: &str) -> Value {
        let title = format!("Intel® Core™ i5-5257U Processor / {kind}");
        json!({
            "id": id,
            "title": title,
            "option1": "Intel® Core™ i5-5257U Processor",
            "option2": kind,
            "option3": null,
            "sku": sku,
            "requires_shipping": true,
            "taxable": true,
            "featured_image": null,
            "available": true,
            "name": format!("MINISFORUM U700 Refurbished - {title}"),
            "public_title": title,
            "options": ["Intel® Core™ i5-5257U Processor", kind],
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

    /// Trimmed from `products-minisforum-u700-refurbished.html`.
    pub(crate) fn product() -> Value {
        let mut barebone = variant(53753234751854, 9900, 19900, "U70000REU", "Barebone");
        barebone["available"] = json!(false);

        json!({
            "id": 14959267250542_u64,
            "title": "MINISFORUM U700 Refurbished",
            "handle": "minisforum-u700-refurbished",
            "description": "<p>d</p>",
            "published_at": "2026-02-11T15:45:22+08:00",
            "created_at": "2026-02-11T15:45:21+08:00",
            "vendor": "Refurbished Mini PC",
            "type": "Refurbished",
            "tags": ["Refurbished Products"],
            "price": 9900,
            "price_min": 9900,
            "price_max": 12900,
            "available": true,
            "price_varies": true,
            "compare_at_price": 19900,
            "compare_at_price_min": 19900,
            "compare_at_price_max": 19900,
            "compare_at_price_varies": false,
            "variants": [
                variant(53296398172526, 12900, 19900, "U70082REU", "8GB RAM + 256GB SSD"),
                barebone
            ],
            "images": ["//minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122"],
            "featured_image": "//minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122",
            "options": ["CPU", "Type"],
            "media": [{
                "alt": "Minisforum U700 Refurbished - Minisforum EU", "id": 53697244987758_u64,
                "position": 1, "media_type": "image",
                "preview_image": {"aspect_ratio": 1.554, "height": 623, "width": 968,
                    "src": "//minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122"},
                "aspect_ratio": 1.554, "height": 623, "width": 968,
                "src": "//minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122"
            }],
            "requires_selling_plan": false,
            "selling_plan_groups": [],
            "content": "<p>d</p>"
        })
    }

    fn preview() -> Value {
        json!({"aspect_ratio": 1.333, "height": 360, "width": 480,
            "src": "//minisforumpc.eu/cdn/shop/files/preview_images/hqdefault.jpg?v=1747994417"})
    }

    /// The two kinds of video, as on `products-kabeltasche.html` and
    /// `products-minisforum-n5-n5-pro.html`.
    fn videos() -> [Value; 2] {
        [
            json!({
                "alt": null, "id": 52214798352750_u64, "position": 7,
                "media_type": "external_video", "preview_image": preview(),
                "aspect_ratio": 1.77, "external_id": "tr9tXOdcEVE", "host": "youtube"
            }),
            json!({
                "alt": null, "id": 52744607891822_u64, "position": 14,
                "media_type": "video", "preview_image": preview(),
                "aspect_ratio": 0.563, "duration": 23570,
                "sources": [{
                    "format": "mp4", "height": 480, "mime_type": "video/mp4", "width": 270,
                    "url": "//minisforumpc.eu/cdn/shop/videos/c/vp/71dfaf56/71dfaf56.SD-480p.mp4?v=0"
                }]
            }),
        ]
    }

    #[test]
    fn the_record_deserializes() {
        let product = serde_json::from_value::<ProductJson>(product()).expect("deserializes");
        assert_eq!(product.variants.len(), 2);
    }

    #[test]
    fn a_record_without_media_fails() {
        let mut input = product();
        input.as_object_mut().expect("an object").remove("media");

        assert!(serde_json::from_value::<ProductJson>(input).is_err());
    }

    #[test]
    fn a_record_with_both_kinds_of_video_deserializes() {
        let mut input = product();
        for video in videos() {
            input["media"].as_array_mut().expect("media").push(video);
        }

        let product = serde_json::from_value::<ProductJson>(input).expect("deserializes");
        assert!(matches!(product.media[0], Media::Image { .. }));
        assert!(matches!(product.media[1], Media::ExternalVideo { .. }));
        assert!(matches!(product.media[2], Media::Video { .. }));
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
    fn an_unknown_key_on_a_video_fails() {
        for mut video in videos() {
            video["captions"] = json!([]);
            assert!(serde_json::from_value::<Media>(video).is_err());
        }
    }
}
