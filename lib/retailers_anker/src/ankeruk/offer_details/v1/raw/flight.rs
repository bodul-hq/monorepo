use html_extractor::serde_json::Value;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlightPayload {
    pub product: Product,
    pub shop: Shop,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Product {
    pub id: String,
    pub handle: String,
    pub title: String,
    pub vendor: String,
    pub product_type: String,
    pub description: String,
    pub description_html: String,
    pub available_for_sale: bool,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: String,

    pub tags: Vec<String>,
    pub seo: Seo,

    pub price: Price,
    pub compare_at_price: Price,
    pub featured_image: Image,

    pub images: Vec<Image>,

    pub media: Vec<Media>,

    pub options: Vec<ProductOption>,

    pub variants: Vec<Variant>,

    pub selling_plan_groups: Vec<SellingPlanGroup>,

    pub metafields: ProductMetafields,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Price {
    /// Decimal currency units, e.g. `"99.99"`, rather than an integer minor-unit amount.
    pub amount: String,

    pub currency_code: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Image {
    pub url: String,
    pub alt_text: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seo {
    pub title: Option<String>,
    pub description: Option<String>,
}

/// One entry of `product.media`, keyed by `mediaContentType`. Shopify also emits `VIDEO` and
/// `MODEL_3D`, unseen in this corpus; an unrecognized tag fails the document rather than silently
/// dropping the entry.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(
    tag = "mediaContentType",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum Media {
    #[serde(rename = "IMAGE")]
    Image { id: String, image: MediaImage },
    /// The storefront's query selects nothing of an external video but its id.
    #[serde(rename = "EXTERNAL_VIDEO")]
    ExternalVideo { id: String },
}

/// The still inside an `IMAGE` media entry. Narrower than [`Image`] - the media list carries
/// no dimensions.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaImage {
    pub url: String,
    pub alt_text: Option<String>,
}

/// One product option and the values it offers, e.g. `Colour` over `["Black", "White"]`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductOption {
    pub id: String,
    pub name: String,
    pub values: Vec<String>,
}

/// `product.metafields`. The namespace set is `deny_unknown_fields`, so a new or renamed
/// namespace fails the document loudly; the content inside each namespace stays `Value` because
/// it is CMS-authored and open-ended.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductMetafields {
    pub global: Option<Value>,
    pub component: Option<Value>,
    #[serde(rename = "originalPayload")]
    pub original_payload: Option<Value>,
    pub seo: Option<Value>,
    pub product: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variant {
    pub id: String,
    pub title: String,
    pub sku: String,
    pub available_for_sale: bool,
    pub requires_shipping: bool,
    pub quantity_available: i64,
    pub price: Price,
    /// Absent from every variant in the current corpus; present where a variant is actually
    /// marked down.
    #[serde(default)]
    pub compare_at_price: Option<Price>,
    pub weight: f64,
    /// Absent on a few single-variant products, which show the product's featured image.
    #[serde(default)]
    pub image: Option<Image>,
    pub selected_options: Vec<SelectedOption>,
    pub selling_plan_allocations: Vec<SellingPlanAllocation>,
    pub metafields: VariantMetafields,
}

/// Which option value this variant is, e.g. `Colour` / `Black`.
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
    pub global: Option<Value>,
    #[serde(rename = "__discountCodeApp")]
    pub discount_code_app: Option<DiscountCodeApp>,
    pub component: Option<Value>,
    #[serde(rename = "originalPayload")]
    pub original_payload: Option<Value>,
    /// Snake_case on the wire, unlike its siblings.
    pub discounts_function: Option<Value>,
    pub filter: Option<Value>,
    /// Ad-tracking tags, e.g. `{"tags": {"checkout-completed": [...]}}`.
    pub track: Option<Value>,
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
    /// Signed and quoted, e.g. `"-32.0"`.
    pub value: String,
    pub currency: String,
    /// The unsigned magnitude of `value`, quoted.
    pub fixed_value: String,
    pub value_style: String,
    pub variant_price4wscode: f64,
    pub discount_type: String,
    pub amazon_deal: bool,
    pub sales_channel: Option<Vec<String>>,
    pub cdi_id: i64,
    pub cdi_updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SellingPlanAllocation {
    pub selling_plan: AllocatedSellingPlan,
    pub price_adjustments: Vec<EmptyAdjustment>,
}

/// The plan an allocation points at. Narrower than [`SellingPlan`]: an allocation's copy
/// carries no `priceAdjustments` of its own.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocatedSellingPlan {
    pub id: String,
    pub name: String,
    pub options: Vec<SellingPlanOption>,
}

/// A named group of selling plans, e.g. a subscribe-and-save offer.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SellingPlanGroup {
    pub name: String,
    pub selling_plans: Vec<SellingPlan>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SellingPlan {
    pub id: String,
    pub name: String,
    pub options: Vec<SellingPlanOption>,
    pub price_adjustments: Vec<EmptyAdjustment>,
}

/// A plan's name/value pair, e.g. `Delivery every` / `30 days`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SellingPlanOption {
    pub name: String,
    pub value: String,
}

/// Adjustment lists are empty in the inspected corpus. This placeholder accepts `{}` entries
/// and rejects objects with fields; it does not enforce an empty list.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyAdjustment {}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Shop {
    pub name: String,
    pub description: String,

    pub money_format: String,
    pub primary_domain: PrimaryDomain,
    pub payment_settings: PaymentSettings,
    pub brand: Brand,

    pub metafields: Value,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimaryDomain {
    pub host: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaymentSettings {
    pub currency_code: String,
    pub enabled_presentment_currencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    pub colors: Value,
    pub logo: Value,
}

#[cfg(test)]
mod tests {
    use super::{DiscountCode, FlightPayload, Media, Variant};
    use html_extractor::serde_json::{self, json};

    #[test]
    fn money_is_a_decimal_string() {
        let payload: FlightPayload = serde_json::from_str(PAYLOAD).expect("the payload reads");
        assert_eq!(payload.product.price.amount, "99.99");
        assert_eq!(payload.product.price.currency_code, "GBP");
    }

    #[test]
    fn the_metafield_namespaces_are_named_and_their_contents_stay_open() {
        let payload: FlightPayload = serde_json::from_str(PAYLOAD).expect("the payload reads");
        assert!(
            payload
                .product
                .metafields
                .global
                .is_some_and(|g| g.is_object())
        );
        assert!(payload.product.metafields.component.is_none());
        assert!(payload.shop.metafields.is_object());

        let renamed = PAYLOAD.replace(r#""global":{}"#, r#""surprise":{}"#);
        assert!(serde_json::from_str::<FlightPayload>(&renamed).is_err());
    }

    #[test]
    fn the_seo_product_and_filter_namespaces_read() {
        let payload = PAYLOAD.replace(
            r#""metafields":{"global":{}}"#,
            r#""metafields":{"global":{},"seo":{"hidden":"1"},
                "product":{"data":{"scene_app_img":"u"}}}"#,
        );
        let payload: FlightPayload = serde_json::from_str(&payload).expect("the payload reads");
        assert!(payload.product.metafields.seo.is_some());
        assert!(payload.product.metafields.product.is_some());

        let mut variant = variant();
        variant["metafields"]["filter"] = json!({"capacity": "10.000mAh - 21.000mAh"});
        let variant: Variant = serde_json::from_value(variant).expect("the variant reads");
        assert!(variant.metafields.filter.is_some());

        let mut variant = self::variant();
        variant["metafields"]["track"] = json!({"tags": {"checkout-completed": ["gtag:x"]}});
        let variant: Variant = serde_json::from_value(variant).expect("the variant reads");
        assert!(variant.metafields.track.is_some());

        let mut variant = self::variant();
        variant.as_object_mut().unwrap().remove("image");
        let variant: Variant = serde_json::from_value(variant).expect("the variant reads");
        assert_eq!(variant.image, None);
    }

    #[test]
    fn an_unmodelled_product_key_is_rejected() {
        let extra = PAYLOAD.replace(r#""handle":"h""#, r#""handle":"h","surprise":1"#);
        let error = serde_json::from_str::<FlightPayload>(&extra)
            .expect_err("a new key from the retailer's query must fail the document");
        assert!(error.to_string().contains("unknown field `surprise`"));
    }

    #[test]
    fn an_external_video_carries_only_its_id_and_an_unknown_media_type_fails() {
        let video: Media = serde_json::from_value(json!({
            "id": "gid://shopify/ExternalVideo/1", "mediaContentType": "EXTERNAL_VIDEO"
        }))
        .expect("an external video reads");
        assert_eq!(
            video,
            Media::ExternalVideo {
                id: "gid://shopify/ExternalVideo/1".to_string()
            }
        );

        let model = serde_json::from_value::<Media>(json!({
            "id": "gid://shopify/Model3d/1", "mediaContentType": "MODEL_3D"
        }));
        assert!(
            model.is_err(),
            "an unseen media type must fail the document"
        );
    }

    #[test]
    fn a_variant_without_a_compare_at_price_reads_and_its_discount_code_is_typed() {
        let variant: Variant = serde_json::from_value(variant()).expect("the variant reads");

        assert_eq!(variant.compare_at_price, None);
        let codes = variant.metafields.discount_code_app.unwrap().data;
        assert_eq!(codes[0].title, "WS24A2345GU");
        assert_eq!(codes[0].sales_channel, None);
        assert_eq!(codes[0].ends_at, None);
    }

    #[test]
    fn an_unmodelled_discount_code_key_is_rejected() {
        let mut code = discount_code();
        code["surprise"] = json!(1);

        assert!(serde_json::from_value::<DiscountCode>(code).is_err());
    }

    fn discount_code() -> html_extractor::serde_json::Value {
        json!({
            "title": "WS24A2345GU", "starts_at": "2026-08-11T06:39:04.000Z", "ends_at": null,
            "variant_shopify_id": 59852742099025i64, "value_type": "fixed_amount",
            "value": "-32.0", "currency": "GBP", "fixed_value": "32.0", "value_style": "£32",
            "variant_price4wscode": 127.99, "discount_type": "WS24", "amazon_deal": false,
            "sales_channel": null, "cdi_id": 168394, "cdi_updated_at": "2026-09-07T09:49:21.942Z"
        })
    }

    fn variant() -> html_extractor::serde_json::Value {
        json!({
            "id": "gid://shopify/ProductVariant/59852742099025", "title": "Black",
            "sku": "A2345341", "availableForSale": true, "requiresShipping": true,
            "quantityAvailable": 3, "price": {"amount": "159.99", "currencyCode": "GBP"},
            "weight": 1, "image": {"url": "u", "altText": null, "width": 1, "height": 1},
            "selectedOptions": [{"name": "Colour", "value": "Black"}],
            "sellingPlanAllocations": [],
            "metafields": {"__discountCodeApp": {"data": [discount_code()]}}
        })
    }

    const PAYLOAD: &str = r#"{
      "product":{"id":"gid://shopify/Product/1","handle":"h","title":"t","vendor":"Anker",
        "productType":"","description":"d","descriptionHtml":"<p>d</p>","availableForSale":true,
        "createdAt":"2024-01-01","updatedAt":"2024-01-02","publishedAt":"2024-01-03","tags":[],
        "seo":{"title":null,"description":null},
        "price":{"amount":"99.99","currencyCode":"GBP"},
        "compareAtPrice":{"amount":"0.0","currencyCode":"GBP"},
        "featuredImage":{"url":"u","altText":null,"width":800,"height":800},
        "images":[{"url":"u","altText":"a","width":800,"height":800}],
        "media":[],"options":[],"variants":[],"sellingPlanGroups":[],"metafields":{"global":{}}},
      "shop":{"name":"Anker","description":"d","moneyFormat":"${{amount}}",
        "primaryDomain":{"host":"h","url":"u"},
        "paymentSettings":{"currencyCode":"GBP","enabledPresentmentCurrencies":["GBP"]},
        "brand":{"colors":{},"logo":{}},"metafields":{}}}"#;
}
