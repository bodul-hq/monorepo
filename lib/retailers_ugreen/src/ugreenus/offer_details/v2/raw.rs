//! Raw GraphQL source shapes owned by ugreenus.
//! These are source representations, not extracted/normalized offer models.
//! Unknown source fields fail deserialization. Decimal money stays as strings;
//! other numeric scalars retain their JSON representation. See fixtures/README.md.

use retailer_sauce::shopify::initial_fill::GraphqlRequest;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

/// The persisted artifact: schema 1 without context, or schema 2 with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "ArtifactWire")]
pub struct RawOfferDetails {
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
    pub product: Product,
    /// Original request/response evidence, retained separately from the merged product.
    pub pages: Vec<RecordedPage>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactWire {
    schema_version: u32,
    #[serde(default)]
    context: Option<retailer_sauce::shopify::markets::MarketContext>,
    product: Product,
    pages: Vec<RecordedPage>,
}
impl TryFrom<ArtifactWire> for RawOfferDetails {
    type Error = String;
    fn try_from(wire: ArtifactWire) -> Result<Self, String> {
        if !matches!(
            (wire.schema_version, wire.context.is_some()),
            (1, false) | (2, true)
        ) {
            return Err("unsupported artifact version or missing/inconsistent context".into());
        }
        Ok(Self {
            schema_version: wire.schema_version,
            context: wire.context,
            product: wire.product,
            pages: wire.pages,
        })
    }
}

/// Pages use different GraphQL selections. Keep their diagnostic evidence verbatim;
/// `ProductResponse` models a full initial response and `Product` the merged result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedPage {
    pub request: GraphqlRequest,
    pub response: Value,
}

/// A full OfferDetails GraphQL response (including partial-data/error responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductResponse {
    pub data: Option<ProductData>,
    pub errors: Option<Vec<GraphqlError>>,
    /// Shopify diagnostic/cost metadata, outside the product source contract.
    pub extensions: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductData {
    pub product: Option<Product>,
    /// The saved data3 crawl queried full products through this connection.
    pub products: Option<Connection<Product>>,
    pub localization: Option<Localization>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Localization {
    pub country: Country,
    pub language: IsoCode,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Country {
    pub iso_code: String,
    pub currency: IsoCode,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct IsoCode {
    pub iso_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphqlError {
    pub message: String,
    pub locations: Option<Vec<ErrorLocation>>,
    pub path: Option<Vec<ErrorPath>>,
    pub extensions: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorPath {
    Field(String),
    Index(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Connection<T> {
    pub page_info: PageInfo,
    pub edges: Vec<Edge<T>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge<T> {
    pub cursor: String,
    pub node: T,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Product {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub available_for_sale: bool,
    pub category: Option<Category>,
    pub collections: Connection<Collection>,
    pub compare_at_price_range: PriceRange,
    pub created_at: String,
    pub description: String,
    pub description_html: String,
    pub encoded_variant_availability: Option<String>,
    pub encoded_variant_existence: Option<String>,
    pub featured_image: Option<Image>,
    pub handle: String,
    pub id: String,
    pub images: Connection<Image>,
    pub is_gift_card: bool,
    pub media: Connection<Media>,
    pub online_store_url: Option<String>,
    pub options: Vec<ProductOption>,
    pub price_range: PriceRange,
    pub product_type: String,
    pub published_at: String,
    pub requires_selling_plan: bool,
    pub selling_plan_groups: Connection<SellingPlanGroup>,
    pub seo: Seo,
    pub tags: Vec<String>,
    pub title: String,
    pub updated_at: String,
    pub variants: Connection<Variant>,
    pub variants_count: Option<Count>,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Category {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub ancestors: Vec<CategoryAncestor>,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CategoryAncestor {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Collection {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub description: String,
    pub description_html: String,
    pub handle: String,
    pub id: String,
    pub image: Option<Image>,
    pub online_store_url: Option<String>,
    pub seo: Seo,
    pub title: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Connection<ProductSummary>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProductSummary {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: String,
    pub handle: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Seo {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Count {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub count: i64,
    pub precision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Image {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub alt_text: Option<String>,
    pub height: Option<i64>,
    pub id: Option<String>,
    pub thumbhash: Option<String>,
    pub url: String,
    pub width: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Money {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub amount: String,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PriceRange {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub max_variant_price: Money,
    pub min_variant_price: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProductOption {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: String,
    pub name: String,
    pub option_values: Vec<OptionValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OptionValue {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: String,
    pub name: String,
    pub swatch: Option<Swatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Swatch {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub color: Option<String>,
    pub image: Option<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MediaPresentation {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VideoSource {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub format: String,
    pub height: i64,
    pub mime_type: String,
    pub url: String,
    pub width: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelSource {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub filesize: i64,
    pub format: String,
    pub mime_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Variant {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub available_for_sale: bool,
    pub barcode: Option<String>,
    pub compare_at_price: Option<Money>,
    pub components: Connection<Component>,
    pub currently_not_in_stock: bool,
    pub grouped_by: Connection<VariantSummary>,
    pub id: String,
    pub image: Option<Image>,
    pub price: Money,
    pub quantity_price_breaks: Connection<QuantityPriceBreak>,
    pub quantity_rule: QuantityRule,
    pub requires_components: bool,
    pub requires_shipping: bool,
    pub selected_options: Vec<SelectedOption>,
    pub selling_plan_allocations: Connection<SellingPlanAllocation>,
    pub sku: Option<String>,
    pub store_availability: Connection<StoreAvailability>,
    pub taxable: bool,
    pub title: String,
    pub unit_price: Option<Money>,
    pub unit_price_measurement: Option<UnitPriceMeasurement>,
    pub weight: Option<Number>,
    pub weight_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VariantSummary {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Component {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub product_variant: VariantSummary,
    pub quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QuantityPriceBreak {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub minimum_quantity: i64,
    pub price: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QuantityRule {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub increment: i64,
    pub maximum: Option<i64>,
    pub minimum: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SelectedOption {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UnitPriceMeasurement {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub measured_type: Option<String>,
    pub quantity_unit: Option<String>,
    pub quantity_value: Number,
    pub reference_unit: Option<String>,
    pub reference_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StoreAvailability {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub available: bool,
    pub location: Location,
    pub pick_up_time: String,
    pub quantity_available: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Location {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub address: Address,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Address {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub formatted: Vec<String>,
    pub latitude: Option<Number>,
    pub longitude: Option<Number>,
    pub phone: Option<String>,
    pub province: Option<String>,
    pub province_code: Option<String>,
    pub zip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SellingPlanAllocation {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub checkout_charge_amount: Money,
    pub price_adjustments: Vec<AllocationPriceAdjustment>,
    pub remaining_balance_charge_amount: Money,
    pub selling_plan: SellingPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AllocationPriceAdjustment {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub compare_at_price: Money,
    pub per_delivery_price: Money,
    pub price: Money,
    pub unit_price: Option<Money>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SellingPlanGroup {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub app_name: Option<String>,
    pub name: String,
    pub options: Vec<SellingPlanGroupOption>,
    pub selling_plans: Connection<SellingPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SellingPlanGroupOption {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SellingPlan {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub billing_policy: Option<BillingPolicy>,
    pub checkout_charge: CheckoutCharge,
    pub delivery_policy: Option<DeliveryPolicy>,
    pub description: Option<String>,
    pub id: String,
    pub name: String,
    pub options: Vec<SellingPlanOption>,
    pub price_adjustments: Vec<PlanPriceAdjustment>,
    pub recurring_deliveries: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SellingPlanOption {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CheckoutCharge {
    #[serde(rename = "__typename")]
    pub type_name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: CheckoutChargeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlanPriceAdjustment {
    #[serde(rename = "__typename")]
    pub type_name: String,
    pub adjustment_value: PriceAdjustmentValue,
    pub order_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename", deny_unknown_fields)]
pub enum Media {
    MediaImage {
        alt: Option<String>,
        id: Option<String>,
        #[serde(rename = "mediaContentType")]
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        #[serde(rename = "previewImage")]
        preview_image: Option<Image>,
        image: Option<Image>,
    },
    ExternalVideo {
        alt: Option<String>,
        id: Option<String>,
        #[serde(rename = "mediaContentType")]
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        #[serde(rename = "previewImage")]
        preview_image: Option<Image>,
        #[serde(rename = "embedUrl")]
        embed_url: String,
        host: String,
        #[serde(rename = "originUrl")]
        origin_url: String,
    },
    Video {
        alt: Option<String>,
        id: Option<String>,
        #[serde(rename = "mediaContentType")]
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        #[serde(rename = "previewImage")]
        preview_image: Option<Image>,
        sources: Vec<VideoSource>,
    },
    Model3d {
        alt: Option<String>,
        id: Option<String>,
        #[serde(rename = "mediaContentType")]
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        #[serde(rename = "previewImage")]
        preview_image: Option<Image>,
        sources: Vec<ModelSource>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename", deny_unknown_fields)]
pub enum BillingPolicy {
    SellingPlanRecurringBillingPolicy {
        interval: String,
        #[serde(rename = "intervalCount")]
        interval_count: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename", deny_unknown_fields)]
pub enum DeliveryPolicy {
    SellingPlanRecurringDeliveryPolicy {
        interval: String,
        #[serde(rename = "intervalCount")]
        interval_count: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename", deny_unknown_fields)]
pub enum CheckoutChargeValue {
    MoneyV2 {
        amount: String,
        #[serde(rename = "currencyCode")]
        currency_code: String,
    },
    SellingPlanCheckoutChargePercentageValue {
        percentage: Number,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename", deny_unknown_fields)]
pub enum PriceAdjustmentValue {
    SellingPlanFixedAmountPriceAdjustment {
        #[serde(rename = "adjustmentAmount")]
        adjustment_amount: Money,
    },
    SellingPlanFixedPriceAdjustment {
        price: Money,
    },
    SellingPlanPercentagePriceAdjustment {
        #[serde(rename = "adjustmentPercentage")]
        adjustment_percentage: Number,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_product_preserves_all_source_fields() {
        let source: Value = serde_json::from_str(include_str!("fixtures/product.json")).unwrap();
        let product: Product = serde_json::from_value(source.clone()).unwrap();
        let typed = serde_json::to_value(product).unwrap();
        fn preserved(source: &Value, typed: &Value) {
            match source {
                Value::Object(fields) => {
                    for (key, value) in fields {
                        preserved(value, &typed[key]);
                    }
                }
                Value::Array(values) => {
                    assert_eq!(values.len(), typed.as_array().unwrap().len());
                    for (index, value) in values.iter().enumerate() {
                        preserved(value, &typed[index]);
                    }
                }
                _ => assert_eq!(source, typed),
            }
        }
        preserved(&source, &typed);
    }
    #[test]
    #[ignore = "reads external data3 corpus"]
    fn saved_graphql_corpus() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data3/ugreenus/runs");
        fn walk(path: &std::path::Path, count: &mut usize) {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, count);
                } else if path.parent().unwrap().file_name().unwrap() == "products"
                    && path.extension().is_some_and(|x| x == "json")
                {
                    let _: Product = serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
                        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                    *count += 1;
                }
            }
        }
        let mut count = 0;
        walk(&root, &mut count);
        assert!(count > 0);
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn nested_fields_and_money_are_strict() {
        let product: Value = serde_json::from_str(include_str!("fixtures/product.json")).unwrap();
        for pointer in ["", "/priceRange/minVariantPrice"] {
            let mut changed = product.clone();
            changed
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unmodeledField".into(), Value::Bool(true));
            assert!(serde_json::from_value::<Product>(changed).is_err());
        }
        let mut changed = product.clone();
        changed.as_object_mut().unwrap().remove("variants");
        assert!(serde_json::from_value::<Product>(changed).is_err());
        let mut changed = product;
        changed["priceRange"]["minVariantPrice"]["amount"] = json!(12.34);
        assert!(serde_json::from_value::<Product>(changed).is_err());
        let adjustment = json!({"__typename":"SellingPlanPercentagePriceAdjustment","adjustmentPercentage":12.5});
        let typed: PriceAdjustmentValue = serde_json::from_value(adjustment.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), adjustment);
    }
    #[test]
    fn historical_and_contextual_artifacts_remain_readable() {
        let product: Value = serde_json::from_str(include_str!("fixtures/product.json")).unwrap();
        let mut artifact = json!({"schema_version":1,"product":product,"pages":[]});
        assert!(serde_json::from_value::<RawOfferDetails>(artifact.clone()).is_ok());
        artifact["schema_version"] = json!(2);
        assert!(serde_json::from_value::<RawOfferDetails>(artifact.clone()).is_err());
        artifact["context"] = json!({"country":"CA","language":"FR","currency":"CAD"});
        assert!(serde_json::from_value::<RawOfferDetails>(artifact.clone()).is_ok());
        artifact["schema_version"] = json!(1);
        assert!(serde_json::from_value::<RawOfferDetails>(artifact).is_err());
        let error: ProductResponse = serde_json::from_value(
            json!({"data":{"product":null},"errors":[{"message":"missing"}]}),
        )
        .unwrap();
        assert!(error.data.unwrap().product.is_none());
    }
}
