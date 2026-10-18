use html_extractor::serde_json::Value;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductJsonItem {
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
    use html_extractor::serde_json;

    #[test]
    fn an_unseen_media_type_fails() {
        let model = r#"{"media_type":"model","alt":null,"id":1,"position":1,
            "preview_image":{"aspect_ratio":1.0,"height":1,"width":1,"src":"s"},"aspect_ratio":1.0}"#;

        let error = serde_json::from_str::<Media>(model).expect_err("no model media is typed");
        assert!(error.to_string().contains("unknown variant `model`"));
    }

    #[test]
    fn the_unused_option_slots_read_as_null() {
        let product =
            serde_json::from_str::<ProductJson>(PRODUCT).expect("the record deserializes");
        let variant = &product.variants[0];

        assert_eq!(variant.option1.as_deref(), Some("ブラック"));
        assert!(variant.option2.is_none() && variant.option3.is_none());
    }

    #[test]
    fn a_variant_sku_may_be_null() {
        let product = serde_json::from_str::<ProductJson>(
            &PRODUCT.replace(r#""sku":"68ANHUB-B14JA""#, r#""sku":null"#),
        )
        .expect("some variants render a null sku");
        assert!(product.variants[0].sku.is_none());
    }

    pub(crate) const PRODUCT: &str = r#"{"id":1,"title":"Anker Hub","handle":"hub",
        "description":"<p>d</p>","published_at":"2021-06-07T09:28:57+09:00",
        "created_at":"2021-06-07T09:28:57+09:00","vendor":"Anker","type":"Hub","tags":["eol"],
        "price":619900,"price_min":619900,"price_max":619900,"available":false,
        "price_varies":false,"compare_at_price":null,"compare_at_price_min":0,
        "compare_at_price_max":0,"compare_at_price_varies":false,
        "images":["//www.ankerjapan.com/cdn/shop/products/a.jpg?v=1"],
        "featured_image":"//www.ankerjapan.com/cdn/shop/products/a.jpg?v=1","options":["カラー"],
        "media":[{"alt":null,"id":2,"position":1,"media_type":"image",
            "preview_image":{"aspect_ratio":1.0,"height":800,"width":800,"src":"//a.jpg"},
            "aspect_ratio":1.0,"height":800,"width":800,"src":"//a.jpg"}],
        "requires_selling_plan":false,"selling_plan_groups":[],"content":"<p>d</p>",
        "variants":[{"id":40082645057697,"title":"ブラック","option1":"ブラック","option2":null,
            "option3":null,"sku":"68ANHUB-B14JA","requires_shipping":true,"taxable":true,
            "featured_image":{"id":3,"product_id":1,"position":1,
                "created_at":"2021-06-07T13:13:17+09:00","updated_at":"2021-06-07T13:21:47+09:00",
                "alt":null,"width":800,"height":800,
                "src":"//www.ankerjapan.com/cdn/shop/products/a.jpg?v=1","variant_ids":[40082645057697]},
            "available":false,"name":"Anker Hub - ブラック","public_title":"ブラック",
            "options":["ブラック"],"price":619900,"weight":0,"compare_at_price":null,
            "inventory_management":"shopify","barcode":"4571411181795",
            "featured_media":{"alt":null,"id":2,"position":1,
                "preview_image":{"aspect_ratio":1.0,"height":800,"width":800,"src":"//a.jpg"}},
            "requires_selling_plan":false,"selling_plan_allocations":[],
            "quantity_rule":{"min":1,"max":null,"increment":1}}]}"#;
}
