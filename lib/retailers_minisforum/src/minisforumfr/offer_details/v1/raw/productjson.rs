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
/// it does not name. Nothing in it names the page's language either: the French and the English
/// page of a product carry the same record, down to the (French) option names.
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
    /// Present on all 118 saved pages, each with at least one entry.
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
    /// `shopify` on 362 of the 376 saved variants, `null` on the 14 of the gift card.
    pub inventory_management: Option<String>,
    pub barcode: Option<String>,
    /// Only rendered alongside a `featured_image`, which 14 of the 376 saved variants have.
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

/// Every one of the 664 media entries on the saved pages is an image; `minisforumuk`'s product
/// video appears on none, so a `video`, `external_video` or `model` fails here.
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
            "name": format!("Minisforum MS-03 Workstation Locale IA Edge - {title}"),
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
            "id": 15885734478193_u64,
            "title": "Minisforum MS-03 Workstation Locale IA Edge",
            "handle": "minisforum-ms-03-workstation",
            "description": "<p>d</p>",
            "published_at": "2026-07-08T11:00:55+08:00",
            "created_at": "2026-07-06T17:15:16+08:00",
            "vendor": "Minisforum FR",
            "type": "Mini Workstation",
            "tags": ["INTEL"],
            "price": 79900,
            "price_min": 79900,
            "price_max": 139900,
            "available": true,
            "price_varies": true,
            "compare_at_price": 99900,
            "compare_at_price_min": 99900,
            "compare_at_price_max": 174900,
            "compare_at_price_varies": true,
            "variants": [
                variant(
                    64248383471985, 79900, 99900, "MS-03-33600EU",
                    "Barebone (Licence OS non incluse)"
                ),
                variant(64248383504753, 139900, 174900, "MS-03-33631EU", "32GB RAM + 1TB SSD")
            ],
            "images": ["//minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698"],
            "featured_image": "//minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698",
            "options": ["CPU", "Type"],
            "media": [{
                "alt": null, "id": 70151040631153_u64, "position": 1, "media_type": "image",
                "preview_image": {"aspect_ratio": 1.0, "height": 1600, "width": 1600,
                    "src": "//minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698"},
                "aspect_ratio": 1.0, "height": 1600, "width": 1600,
                "src": "//minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698"
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
    fn a_record_without_media_fails() {
        let mut input = product();
        input.as_object_mut().expect("an object").remove("media");

        let error = serde_json::from_value::<ProductJson>(input).expect_err("media is required");
        assert!(error.to_string().contains("missing field `media`"));
    }

    #[test]
    fn a_variant_with_an_image_of_its_own_deserializes() {
        let mut input = product();
        input["variants"][0]["featured_image"] = json!({
            "id": 81421996491121_u64, "product_id": 15885734478193_u64, "position": 1,
            "created_at": "2026-09-02T19:44:32+08:00", "updated_at": "2026-09-02T19:44:34+08:00",
            "alt": null, "width": 1600, "height": 1600,
            "src": "//minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698",
            "variant_ids": [64248383471985_u64]
        });
        input["variants"][0]["featured_media"] = json!({
            "alt": null, "id": 70151040631153_u64, "position": 1,
            "preview_image": {"aspect_ratio": 1.0, "height": 1600, "width": 1600,
                "src": "//minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698"}
        });

        let product = serde_json::from_value::<ProductJson>(input).expect("deserializes");
        assert!(product.variants[0].featured_media.is_some());
        assert!(product.variants[1].featured_media.is_none());
    }

    #[test]
    fn an_unseen_media_type_fails() {
        for media_type in ["video", "external_video", "model"] {
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
