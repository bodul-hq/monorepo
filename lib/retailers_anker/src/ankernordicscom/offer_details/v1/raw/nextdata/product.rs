use html_extractor::serde_json::Value;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Product {
    pub id: String,
    pub handle: String,
    pub slug: String,
    pub path: String,
    pub title: String,
    pub description: String,
    pub description_html: String,
    pub vendor: String,
    pub product_type: String,
    pub updated_at: String,
    pub available_for_sale: bool,
    pub tags: Vec<String>,

    pub price: Price,
    pub price_range: PriceRange,

    pub compare_at_price_range: CompareAtPriceRange,

    pub featured_image: Image,

    pub images: Vec<Image>,
    pub seo: ProductSeo,

    pub media: Vec<Media>,

    pub options: Vec<ProductOption>,

    pub variants: Vec<Variant>,

    pub metafields: ProductMetafields,
}

/// A JSON number on the wire: an integer on most pages, a float on some.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Price {
    pub amount: f64,
    pub currency_code: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PriceStr {
    pub amount: String,
    pub currency_code: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PriceRange {
    pub min_variant_price: PriceStr,
    pub max_variant_price: PriceStr,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompareAtPriceRange {
    pub min_variant_price: PriceStr,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Image {
    pub url: String,

    pub alt_text: Option<String>,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductSeo {
    pub title: Option<String>,

    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "mediaContentType")]
pub enum Media {
    #[serde(rename = "IMAGE")]
    Image(MediaImage),
    #[serde(rename = "VIDEO")]
    Video(MediaVideo),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaImage {
    pub id: String,
    pub presentation: Option<MediaPresentation>,
    pub image: MediaImageSource,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaImageSource {
    pub url: String,
    pub alt_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaVideo {
    pub id: String,
    pub presentation: Option<MediaPresentation>,
    pub sources: Vec<VideoSource>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaPresentation {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoSource {
    pub url: String,
    pub format: String,
    pub mime_type: String,
    pub width: i64,
    pub height: i64,
}

/// The product-level list. Its names are lowercase (`color`), matching the variant's own
/// `options` rendering rather than its `selectedOptions`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductOption {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub id: String,
    pub name: String,
    pub values: Vec<OptionValue>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OptionValue {
    pub label: String,
    #[serde(default)]
    pub hex_colors: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductMetafields {
    pub product: Option<Value>,
    pub global: Option<Value>,
    pub component: Option<Value>,
    pub judgeme: Option<Value>,
    pub collection: Option<Value>,
    pub template: Option<Value>,
    pub landing: Option<Value>,
    pub seo: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variant {
    pub id: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub title: String,
    pub price: Price,
    pub compare_at_price: Option<Price>,
    pub quantity_available: i64,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub image: Image,
    pub options: Vec<VariantOptionRendering>,
    pub selected_options: Vec<SelectedOption>,
    /// The storefront's filtered copy of `metafields.__discountCodeApp.data`.
    pub coupons: Vec<DiscountCode>,
    /// The parent record again, without its variants.
    pub product: Value,
    pub metafields: VariantMetafields,
}

/// Its `id` is the variant's own gid, not an option id.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantOptionRendering {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub id: String,
    pub name: String,
    pub values: Vec<OptionValue>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedOption {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantMetafields {
    #[serde(rename = "__discountCodeApp")]
    pub discount_code_app: Option<DiscountCodeApp>,
    pub component: Option<Value>,
    pub global: Option<Value>,
    pub bundle: Option<Value>,
    pub free_gift: Option<Value>,
    pub discounts_function: Option<Value>,
    pub qty_level: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscountCodeApp {
    pub data: Vec<DiscountCode>,
}

/// One discount code standing against a variant. Snake_case on the wire.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscountCode {
    pub title: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
    pub variant_shopify_id: i64,
    pub value_type: String,
    /// Signed and quoted, e.g. `"-20.0"`.
    pub value: String,
    pub currency: String,
    /// The saving in the code's currency, quoted.
    pub fixed_value: String,
    /// A display label such as `kr20`, not a currency amount.
    pub value_style: String,
    pub variant_price4wscode: f64,
    pub discount_type: String,
    pub amazon_deal: bool,
    pub sales_channel: Option<Vec<String>>,
    pub cdi_id: i64,
    pub cdi_updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::{DiscountCode, Media, Price, ProductMetafields};
    use html_extractor::serde_json;

    #[test]
    fn the_landing_and_seo_namespaces_read() {
        let metafields: ProductMetafields = serde_json::from_str(
            r#"{"landing":{"url":"/pages/boom2-plus-outdoor-bass-speaker"},"seo":{}}"#,
        )
        .expect("the namespaces read");
        assert!(metafields.landing.is_some());
        assert!(metafields.seo.is_some());
    }

    #[test]
    fn an_unseen_media_type_fails() {
        assert!(
            serde_json::from_str::<Media>(
                r#"{"mediaContentType":"MODEL_3D","id":"m","presentation":null}"#
            )
            .is_err()
        );
    }

    #[test]
    fn a_video_carries_its_sources() {
        let media: Media = serde_json::from_str(
            r#"{"mediaContentType":"VIDEO","id":"m","presentation":null,"sources":[
                {"url":"u","format":"mp4","mimeType":"video/mp4","width":1,"height":1}]}"#,
        )
        .unwrap();

        assert!(matches!(media, Media::Video(video) if video.sources.len() == 1));
    }

    #[test]
    fn an_integer_amount_parses() {
        let price: Price = serde_json::from_str(r#"{"amount":2490,"currencyCode":"SEK"}"#).unwrap();

        assert_eq!(price.amount, 2490.0);
    }

    #[test]
    fn an_unknown_discount_key_fails() {
        let code = r#"{"title":"WS248DCU5Y4G","starts_at":"2026-09-06T21:59:00.000Z",
            "ends_at":"2026-09-13T21:59:00.000Z","variant_shopify_id":1,
            "value_type":"percentage","value":"-20.0","currency":"SEK","fixed_value":"498.0",
            "value_style":"kr20","variant_price4wscode":1992,"discount_type":"WS24",
            "amazon_deal":false,"sales_channel":null,"cdi_id":1,
            "cdi_updated_at":"2026-09-07T02:58:48.801Z"}"#;

        assert!(serde_json::from_str::<DiscountCode>(code).is_ok());
        assert!(
            serde_json::from_str::<DiscountCode>(&code.replace("\"cdi_id\"", "\"other_id\""))
                .is_err()
        );
    }
}
