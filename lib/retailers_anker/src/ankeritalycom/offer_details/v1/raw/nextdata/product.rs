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

/// One entry of `product.media`, keyed by `mediaContentType`. Shopify also emits `VIDEO`,
/// `EXTERNAL_VIDEO` and `MODEL_3D`, unseen in this corpus; an unrecognized tag fails the document
/// rather than silently dropping the entry.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(
    tag = "mediaContentType",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum Media {
    #[serde(rename = "IMAGE")]
    Image {
        id: String,
        image: MediaImage,
        presentation: Option<MediaPresentation>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaImage {
    pub url: String,
    pub alt_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaPresentation {
    pub id: String,
}

/// A product option and the values it offers, e.g. `color` over `Nero` and `Bianco`. Also the
/// shape of each variant's own `options` entry, where `id` holds the variant's gid instead.
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
    pub hex_colors: Option<Vec<String>>,
}

/// `product.metafields`. The namespace set is `deny_unknown_fields`, so a new or renamed
/// namespace fails the document loudly; the content inside each namespace stays `Value` because
/// it is CMS-authored and open-ended.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductMetafields {
    pub judgeme: Option<Value>,
    pub global: Option<Value>,
    pub component: Option<Value>,
    pub product: Option<Value>,
    pub seo: Option<Value>,
    pub layout: Option<Value>,
    pub landing: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variant {
    pub id: String,
    pub title: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub quantity_available: i64,
    pub price: Price,
    /// Its amount is quoted, unlike `price`'s.
    pub compare_at_price: Option<PriceStr>,
    pub image: Image,
    pub options: Vec<ProductOption>,
    pub selected_options: Vec<SelectedOption>,
    /// The same entries as `metafields.__discountCodeApp.data`.
    pub coupons: Vec<DiscountCode>,
    /// A copy of the product record this variant belongs to.
    pub product: Value,
    pub metafields: VariantMetafields,
}

/// Which option value this variant is, e.g. `Color` / `Nero`. The name's case is not the
/// product option's on every page.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedOption {
    pub name: String,
    pub value: String,
}

/// `product.variants[].metafields`. Every namespace is optional; see [`ProductMetafields`] for
/// why most contents stay `Value`. The discount app's is typed, because it is an app's record
/// rather than CMS content.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantMetafields {
    pub component: Option<Value>,
    #[serde(rename = "__discountCodeApp")]
    pub discount_code_app: Option<DiscountCodeApp>,
    pub bundle: Option<Value>,
    pub global: Option<Value>,
    pub free_gift: Option<Value>,
    pub discounts_function: Option<Value>,
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
    /// Signed and quoted, e.g. `"-16.0"`.
    pub value: String,
    pub currency: String,
    /// The unsigned saving, quoted.
    pub fixed_value: String,
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
    use super::{DiscountCode, Media, ProductMetafields, Variant};
    use html_extractor::serde_json::{self, Value, json};

    fn discount_code() -> Value {
        json!({
            "title": "WS24FG3SG6TD", "starts_at": "2025-10-22T05:36:56.000Z", "ends_at": null,
            "variant_shopify_id": 45124210524297i64, "value_type": "fixed_amount",
            "value": "-16.0", "currency": "EUR", "fixed_value": "16.0", "value_style": "16 €",
            "variant_price4wscode": 33.99, "discount_type": "WS24", "amazon_deal": false,
            "sales_channel": null, "cdi_id": 127766, "cdi_updated_at": "2026-09-07T10:25:51.914Z"
        })
    }

    fn variant() -> Value {
        json!({
            "title": "Nero", "quantityAvailable": 3893, "currentlyNotInStock": false,
            "availableForSale": true, "barcode": "194644248444",
            "image": {"url": "u", "altText": null, "width": 1, "height": 1},
            "compareAtPrice": null, "id": "gid://shopify/ProductVariant/45124210524297",
            "sku": "A121C311", "price": {"amount": 49.99, "currencyCode": "EUR"},
            "options": [{"__typename": "MultipleChoiceOption",
                         "id": "gid://shopify/ProductVariant/45124210524297",
                         "name": "color", "values": [{"label": "Nero"}]}],
            "selectedOptions": [{"name": "Color", "value": "Nero"}],
            "coupons": [discount_code()], "product": {},
            "metafields": {"__discountCodeApp": {"data": [discount_code()]}}
        })
    }

    #[test]
    fn a_variant_reads_with_its_codes_typed_and_no_compare_at_price() {
        let variant: Variant = serde_json::from_value(variant()).expect("the variant reads");

        assert_eq!(variant.compare_at_price, None);
        assert_eq!(variant.coupons[0].title, "WS24FG3SG6TD");
        assert_eq!(
            variant.metafields.discount_code_app.unwrap().data,
            variant.coupons
        );
    }

    #[test]
    fn a_quoted_compare_at_amount_reads() {
        let mut raw = variant();
        raw["compareAtPrice"] = json!({"amount": "69.99", "currencyCode": "EUR"});

        let variant: Variant = serde_json::from_value(raw).expect("the variant reads");

        assert_eq!(variant.compare_at_price.unwrap().amount, "69.99");
    }

    #[test]
    fn an_unmodelled_discount_code_key_is_rejected() {
        let mut code = discount_code();
        code["surprise"] = json!(1);

        assert!(serde_json::from_value::<DiscountCode>(code).is_err());
    }

    #[test]
    fn an_unseen_media_type_fails_the_document() {
        let image = json!({
            "id": "gid://shopify/MediaImage/1", "mediaContentType": "IMAGE",
            "presentation": null, "image": {"altText": null, "url": "u"}
        });
        let video = json!({"id": "gid://shopify/Video/1", "mediaContentType": "VIDEO"});

        assert!(serde_json::from_value::<Media>(image).is_ok());
        assert!(serde_json::from_value::<Media>(video).is_err());
    }

    #[test]
    fn the_metafield_namespaces_are_named() {
        let named: ProductMetafields =
            serde_json::from_value(json!({"judgeme": {}, "global": {}})).unwrap();
        assert!(named.judgeme.is_some());

        let landing: ProductMetafields = serde_json::from_value(
            json!({"landing": {"url": "/pages/boom2-plus-outdoor-bass-speaker"}}),
        )
        .unwrap();
        assert!(landing.landing.is_some());

        assert!(serde_json::from_value::<ProductMetafields>(json!({"surprise": {}})).is_err());
    }
}
