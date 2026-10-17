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
    /// Absent, like the variant `image`, on the few accessory pages that carry no image at all,
    /// e.g. the Sport X20 eartips kits (`a3968z21-94`); `images` and `media` are empty there.
    pub featured_image: Option<Image>,

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
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seo {
    pub title: Option<String>,
    pub description: Option<String>,
}

/// One entry of `product.media`, keyed by `mediaContentType`. Shopify also emits `MODEL_3D`,
/// unseen in this corpus; an unrecognized tag fails the document rather than silently dropping
/// the entry.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(
    tag = "mediaContentType",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum Media {
    #[serde(rename = "IMAGE")]
    Image { id: String, image: MediaImage },
    #[serde(rename = "VIDEO")]
    Video {
        id: String,
        sources: Vec<MediaSource>,
    },
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

/// One encoded rendition of a `VIDEO` media entry.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaSource {
    pub url: String,
    pub format: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
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
/// it is CMS-authored and genuinely open-ended - the `component` namespace, for instance, keys
/// its entries by CMS block instance rather than a fixed key set.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductMetafields {
    pub global: Option<Value>,
    pub component: Option<Value>,
    #[serde(rename = "originalPayload")]
    pub original_payload: Option<Value>,
    pub seo: Option<Value>,
    pub product: Option<Value>,
    pub template: Option<Value>,
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
    pub compare_at_price: Option<Price>,
    pub weight: f64,
    /// Absent where the product has no `featuredImage`.
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
/// why the contents stay `Value`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantMetafields {
    pub global: Option<Value>,
    #[serde(rename = "__discountCodeApp")]
    pub discount_code_app: Option<Value>,
    pub component: Option<Value>,
    #[serde(rename = "originalPayload")]
    pub original_payload: Option<Value>,
    pub filter: Option<Value>,
    /// Snake_case on the wire, unlike its siblings.
    pub discounts_function: Option<Value>,
    pub variant: Option<Value>,
    pub track: Option<Value>,
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
    use super::FlightPayload;
    use html_extractor::serde_json;

    #[test]
    fn money_is_a_decimal_string() {
        let payload: FlightPayload = serde_json::from_str(PAYLOAD).expect("the payload reads");
        assert_eq!(payload.product.price.amount, "99.99");
        assert_eq!(payload.product.price.currency_code, "CAD");
    }

    #[test]
    fn the_shop_metafields_stay_named_rather_than_typed() {
        let payload: FlightPayload = serde_json::from_str(PAYLOAD).expect("the payload reads");
        assert!(payload.shop.metafields.is_object());
    }

    #[test]
    fn the_product_metafield_namespaces_stay_present_but_their_contents_stay_opaque() {
        let payload: FlightPayload = serde_json::from_str(PAYLOAD).expect("the payload reads");
        assert!(payload.product.metafields.global.unwrap().is_object());
        assert!(payload.product.metafields.component.is_none());
    }

    #[test]
    fn an_unmodelled_product_key_is_rejected() {
        let extra = PAYLOAD.replace(r#""handle":"h""#, r#""handle":"h","surprise":1"#);
        let error = serde_json::from_str::<FlightPayload>(&extra)
            .expect_err("a new key from the retailer's query must fail the document");
        assert!(error.to_string().contains("unknown field `surprise`"));
    }

    #[test]
    fn an_unmodelled_metafield_namespace_is_rejected() {
        let extra = PAYLOAD.replace(
            r#""metafields":{"global":{}}"#,
            r#""metafields":{"global":{},"surprise":{}}"#,
        );
        let error = serde_json::from_str::<FlightPayload>(&extra).expect_err(
            "a new metafield namespace from the retailer's query must fail the document",
        );
        assert!(error.to_string().contains("unknown field `surprise`"));
    }

    #[test]
    fn media_is_tagged_by_its_content_type() {
        let payload: FlightPayload = serde_json::from_str(PAYLOAD).expect("the payload reads");
        match &payload.product.media[0] {
            super::Media::Image { id, image } => {
                assert_eq!(id, "gid://shopify/MediaImage/1");
                assert_eq!(image.url, "u");
            }
            other => panic!("the fixture's only entry is an image, got {other:?}"),
        }
    }

    #[test]
    fn an_external_video_carries_only_its_id() {
        let media: super::Media = serde_json::from_value(serde_json::json!({
            "id": "gid://shopify/ExternalVideo/1", "mediaContentType": "EXTERNAL_VIDEO"
        }))
        .expect("an external video entry reads");
        assert_eq!(
            media,
            super::Media::ExternalVideo {
                id: "gid://shopify/ExternalVideo/1".to_owned()
            }
        );
    }

    #[test]
    fn a_variant_carries_its_own_price_and_metafields() {
        let payload: FlightPayload = serde_json::from_str(PAYLOAD).expect("the payload reads");
        let variant = &payload.product.variants[0];
        assert_eq!(variant.price.amount, "99.99");
        assert!(variant.compare_at_price.is_none());
        assert_eq!(variant.selected_options[0].value, "Default Title");
    }

    #[test]
    fn an_imageless_product_and_its_variant_read() {
        let imageless = PAYLOAD
            .replace(
                r#""featuredImage":{"url":"u","altText":null,"width":800,"height":800},"#,
                "",
            )
            .replace(
                r#""weight":1.5,"image":{"url":"u","altText":null,"width":800,"height":800},"#,
                r#""weight":1.5,"#,
            );
        let payload: FlightPayload =
            serde_json::from_str(&imageless).expect("the image-less payload reads");
        assert!(payload.product.featured_image.is_none());
        assert!(payload.product.variants[0].image.is_none());
    }

    #[test]
    fn the_bundle_metafield_namespaces_read() {
        let bundle = PAYLOAD
            .replace(
                r#""sellingPlanGroups":[],"metafields":{"global":{}}"#,
                r#""sellingPlanGroups":[],"metafields":{"global":{},"template":{}}"#,
            )
            .replace(
                r#""sellingPlanAllocations":[],"metafields":{"global":{}}"#,
                r#""sellingPlanAllocations":[],"metafields":{"global":{},"track":{}}"#,
            );
        let payload: FlightPayload = serde_json::from_str(&bundle).expect("the bundle reads");
        assert!(payload.product.metafields.template.is_some());
        assert!(payload.product.variants[0].metafields.track.is_some());
    }

    const PAYLOAD: &str = r#"{
      "product":{"id":"gid://shopify/Product/1","handle":"h","title":"t","vendor":"Anker",
        "productType":"","description":"d","descriptionHtml":"<p>d</p>","availableForSale":true,
        "createdAt":"2024-01-01","updatedAt":"2024-01-02","publishedAt":"2024-01-03","tags":[],
        "seo":{"title":null,"description":null},
        "price":{"amount":"99.99","currencyCode":"CAD"},
        "compareAtPrice":{"amount":"0.0","currencyCode":"CAD"},
        "featuredImage":{"url":"u","altText":null,"width":800,"height":800},
        "images":[{"url":"u","altText":"a","width":800,"height":800}],
        "media":[{"id":"gid://shopify/MediaImage/1","mediaContentType":"IMAGE",
          "image":{"url":"u","altText":null}}],
        "options":[{"id":"gid://shopify/ProductOption/1","name":"Title","values":["Default Title"]}],
        "variants":[{"id":"gid://shopify/ProductVariant/1","title":"Default Title","sku":"s",
          "availableForSale":true,"requiresShipping":true,"quantityAvailable":1,
          "price":{"amount":"99.99","currencyCode":"CAD"},
          "weight":1.5,"image":{"url":"u","altText":null,"width":800,"height":800},
          "selectedOptions":[{"name":"Title","value":"Default Title"}],
          "sellingPlanAllocations":[],"metafields":{"global":{}}}],
        "sellingPlanGroups":[],"metafields":{"global":{}}},
      "shop":{"name":"Anker","description":"d","moneyFormat":"${{amount}}",
        "primaryDomain":{"host":"h","url":"u"},
        "paymentSettings":{"currencyCode":"CAD","enabledPresentmentCurrencies":["CAD"]},
        "brand":{"colors":{},"logo":{}},"metafields":{}}}"#;
}
