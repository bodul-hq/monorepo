use super::super::raw;
use super::{NormalizeError, decimal, decimal_money, money};
use chrono::{DateTime, Utc};
use money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductRecord {
    pub id: String,
    pub handle: String,
    pub title: String,
    /// The sub-brand - `Eufy`, `Soundcore` - that the island's `Anker` flattens away.
    pub vendor: String,
    pub product_type: String,
    pub description: String,
    // Commented out: the long copy with markup; adds no value for now.
    // pub description_html: String,
    pub available_for_sale: bool,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: DateTime<Utc>,
    // Commented out: not needed for now.
    // pub tags: Vec<String>,
    pub seo: ProductSeo,
    pub price: Money,
    /// `None` where the product is not marked down.
    pub compare_at_price: Option<Money>,
    pub featured_image: ProductImage,
    pub images: Vec<ProductImage>,
    // Commented out: its images mirror `images` one for one, and its external videos carry no
    // more than an id.
    // pub media: Vec<Value>,
    pub options: Vec<ProductOption>,
    pub variants: Vec<ProductVariant>,
    pub selling_plan_groups: Vec<SellingPlanGroup>,
    // Commented out: an untyped, per-namespace CMS/storefront-configuration blob - see
    // `raw::flight::ProductMetafields`. The specification blocks inside `originalPayload` are the
    // facts worth lifting out of it by name.
    // pub metafields: ProductMetafields,
}

impl ProductRecord {
    pub(crate) fn from_raw(raw: raw::flight::Product) -> Result<ProductRecord, NormalizeError> {
        Ok(ProductRecord {
            id: raw.id,
            handle: raw.handle,
            title: raw.title,
            vendor: raw.vendor,
            product_type: raw.product_type,
            description: raw.description,
            // description_html: raw.description_html,
            available_for_sale: raw.available_for_sale,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            published_at: published_at(&raw.published_at)?,
            // tags: raw.tags,
            seo: ProductSeo::from_raw(raw.seo),
            price: price(&raw.price, "price")?,
            compare_at_price: compare_at_price(&raw.compare_at_price, "compareAtPrice")?,
            featured_image: ProductImage::from_raw(raw.featured_image),
            images: raw.images.into_iter().map(ProductImage::from_raw).collect(),
            // media: raw.media,
            options: raw
                .options
                .into_iter()
                .map(ProductOption::from_raw)
                .collect(),
            variants: raw
                .variants
                .into_iter()
                .map(ProductVariant::from_raw)
                .collect::<Result<_, _>>()?,
            selling_plan_groups: raw
                .selling_plan_groups
                .into_iter()
                .map(SellingPlanGroup::from_raw)
                .collect(),
            // metafields: ProductMetafields::from_raw(raw.metafields),
        })
    }
}

fn published_at(raw: &str) -> Result<DateTime<Utc>, NormalizeError> {
    let parsed = DateTime::parse_from_rfc3339(raw).map_err(|source| {
        NormalizeError::PublishedAtNotAnInstant {
            value: raw.to_string(),
            source,
        }
    })?;

    Ok(parsed.with_timezone(&Utc))
}

fn price(raw: &raw::flight::Price, field: &str) -> Result<Money, NormalizeError> {
    decimal_money(&raw.amount, &raw.currency_code, field)
}

// `"0.0"` is the storefront's sentinel for "not marked down", rather than a real zero price.
fn compare_at_price(
    raw: &raw::flight::Price,
    field: &str,
) -> Result<Option<Money>, NormalizeError> {
    let amount = decimal(&raw.amount, field)?;

    (amount != 0.0)
        .then(|| money(amount, &raw.currency_code, field))
        .transpose()
}

// `0.0` is what an unweighed variant renders rather than a weightless one.
fn weight(raw: f64) -> Option<f64> {
    (raw != 0.0).then_some(raw)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductImage {
    pub url: String,
    // Commented out: not needed for now.
    // pub alt_text: Option<String>,
    // pub width: Option<i64>,
    // pub height: Option<i64>,
}

impl ProductImage {
    fn from_raw(raw: raw::flight::Image) -> ProductImage {
        ProductImage {
            url: raw.url,
            // alt_text: raw.alt_text,
            // width: raw.width,
            // height: raw.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductSeo {
    pub title: Option<String>,
    pub description: Option<String>,
}

impl ProductSeo {
    fn from_raw(raw: raw::flight::Seo) -> ProductSeo {
        ProductSeo {
            title: raw.title,
            description: raw.description,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductOption {
    pub id: String,
    pub name: String,
    pub values: Vec<String>,
}

impl ProductOption {
    fn from_raw(raw: raw::flight::ProductOption) -> ProductOption {
        ProductOption {
            id: raw.id,
            name: raw.name,
            values: raw.values,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductVariant {
    pub id: String,
    pub title: String,
    pub sku: String,
    pub available_for_sale: bool,
    pub requires_shipping: bool,
    pub quantity_available: i64,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    /// Pounds: every SKU this storefront shares with `ankerde` reads 2.205 times its kilograms.
    /// `None` where the variant is unweighed.
    pub weight: Option<f64>,
    pub image: Option<ProductImage>,
    pub selected_options: Vec<SelectedOption>,
    pub selling_plan_allocations: Vec<SellingPlanAllocation>,
    // Commented out: untyped, per-namespace CMS/storefront configuration - see
    // `raw::flight::VariantMetafields`. The discount app's namespace is lifted into `discounts`.
    // pub metafields: VariantMetafields,
    pub discounts: Vec<Discount>,
}

impl ProductVariant {
    fn from_raw(raw: raw::flight::Variant) -> Result<ProductVariant, NormalizeError> {
        Ok(ProductVariant {
            id: raw.id,
            title: raw.title,
            sku: raw.sku,
            available_for_sale: raw.available_for_sale,
            requires_shipping: raw.requires_shipping,
            quantity_available: raw.quantity_available,
            price: price(&raw.price, "variants[].price")?,
            compare_at_price: raw
                .compare_at_price
                .map(|price| compare_at_price(&price, "variants[].compareAtPrice"))
                .transpose()?
                .flatten(),
            weight: weight(raw.weight),
            image: raw.image.map(ProductImage::from_raw),
            selected_options: raw
                .selected_options
                .into_iter()
                .map(SelectedOption::from_raw)
                .collect(),
            selling_plan_allocations: raw
                .selling_plan_allocations
                .into_iter()
                .map(SellingPlanAllocation::from_raw)
                .collect(),
            // metafields: VariantMetafields::from_raw(raw.metafields),
            discounts: discounts(raw.metafields.discount_code_app)?,
        })
    }
}

fn discounts(raw: Option<raw::flight::DiscountCodeApp>) -> Result<Vec<Discount>, NormalizeError> {
    raw.map_or_else(Vec::new, |app| app.data)
        .into_iter()
        .map(Discount::from_raw)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedOption {
    pub name: String,
    pub value: String,
}

impl SelectedOption {
    fn from_raw(raw: raw::flight::SelectedOption) -> SelectedOption {
        SelectedOption {
            name: raw.name,
            value: raw.value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Discount {
    pub code: String,
    pub kind: String,
    pub value: DiscountValue,
    pub price: Money,
    pub saving: Money,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub variant_shopify_id: i64,
    pub amazon_deal: bool,
    /// The channels the code is limited to, e.g. `["app"]`. What `None` means is not established.
    pub sales_channels: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscountValue {
    /// Signed percentage points as supplied by the page: `-25.0` means 25% off.
    Percentage(f64),
    FixedAmount(Money),
}

impl Discount {
    fn from_raw(raw: raw::flight::DiscountCode) -> Result<Discount, NormalizeError> {
        let instant = |field: &str, value: &str| {
            DateTime::parse_from_rfc3339(value)
                .map(|instant| instant.with_timezone(&Utc))
                .map_err(|source| NormalizeError::DiscountInstantNotAnInstant {
                    code: raw.title.clone(),
                    field: field.to_string(),
                    value: value.to_string(),
                    source,
                })
        };

        let value = match raw.value_type.as_str() {
            "percentage" => DiscountValue::Percentage(decimal(&raw.value, "value")?),
            "fixed_amount" => {
                DiscountValue::FixedAmount(decimal_money(&raw.value, &raw.currency, "value")?)
            }
            other => {
                return Err(NormalizeError::UnknownDiscountValueType {
                    code: raw.title,
                    value_type: other.to_string(),
                });
            }
        };

        Ok(Discount {
            value,
            price: money(
                raw.variant_price4wscode,
                &raw.currency,
                "variant_price4wscode",
            )?,
            saving: decimal_money(&raw.fixed_value, &raw.currency, "fixed_value")?,
            starts_at: instant("starts_at", &raw.starts_at)?,
            ends_at: raw
                .ends_at
                .as_deref()
                .map(|ends_at| instant("ends_at", ends_at))
                .transpose()?,
            code: raw.title,
            kind: raw.discount_type,
            variant_shopify_id: raw.variant_shopify_id,
            amazon_deal: raw.amazon_deal,
            sales_channels: raw.sales_channel,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SellingPlanAllocation {
    pub selling_plan: AllocatedSellingPlan,
}

impl SellingPlanAllocation {
    // The `priceAdjustments` each allocation carries are empty everywhere in the corpus - see
    // [`raw::flight::EmptyAdjustment`] - so they carry no information forward here.
    fn from_raw(raw: raw::flight::SellingPlanAllocation) -> SellingPlanAllocation {
        SellingPlanAllocation {
            selling_plan: AllocatedSellingPlan::from_raw(raw.selling_plan),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocatedSellingPlan {
    pub id: String,
    pub name: String,
    pub options: Vec<SellingPlanOption>,
}

impl AllocatedSellingPlan {
    fn from_raw(raw: raw::flight::AllocatedSellingPlan) -> AllocatedSellingPlan {
        AllocatedSellingPlan {
            id: raw.id,
            name: raw.name,
            options: raw
                .options
                .into_iter()
                .map(SellingPlanOption::from_raw)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SellingPlanGroup {
    pub name: String,
    pub selling_plans: Vec<SellingPlan>,
}

impl SellingPlanGroup {
    fn from_raw(raw: raw::flight::SellingPlanGroup) -> SellingPlanGroup {
        SellingPlanGroup {
            name: raw.name,
            selling_plans: raw
                .selling_plans
                .into_iter()
                .map(SellingPlan::from_raw)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SellingPlan {
    pub id: String,
    pub name: String,
    pub options: Vec<SellingPlanOption>,
}

impl SellingPlan {
    fn from_raw(raw: raw::flight::SellingPlan) -> SellingPlan {
        SellingPlan {
            id: raw.id,
            name: raw.name,
            options: raw
                .options
                .into_iter()
                .map(SellingPlanOption::from_raw)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SellingPlanOption {
    pub name: String,
    pub value: String,
}

impl SellingPlanOption {
    fn from_raw(raw: raw::flight::SellingPlanOption) -> SellingPlanOption {
        SellingPlanOption {
            name: raw.name,
            value: raw.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::{self, Value, json};

    fn gbp(amount: i64) -> Money {
        Money::from_raw(amount as f64 / 100.0, "GBP").unwrap()
    }

    fn discount_code() -> Value {
        json!({
            "title": "WS24A2345GU", "starts_at": "2026-08-11T06:39:04.000Z",
            "ends_at": "2026-09-13T21:59:59.000Z", "variant_shopify_id": 59852742099025i64,
            "value_type": "fixed_amount", "value": "-32.0", "currency": "GBP",
            "fixed_value": "32.0", "value_style": "£32", "variant_price4wscode": 127.99,
            "discount_type": "WS24", "amazon_deal": false, "sales_channel": null,
            "cdi_id": 168394, "cdi_updated_at": "2026-09-07T09:49:21.942Z"
        })
    }

    fn raw_variant(weight: f64, codes: Vec<Value>) -> Value {
        json!({
            "id": "gid://shopify/ProductVariant/59852742099025", "title": "Black",
            "sku": "A2345341", "availableForSale": true, "requiresShipping": true,
            "quantityAvailable": 3, "price": {"amount": "159.99", "currencyCode": "GBP"},
            "weight": weight, "image": {"url": "u", "altText": null, "width": 1, "height": 1},
            "selectedOptions": [{"name": "Colour", "value": "Black"}],
            "sellingPlanAllocations": [],
            "metafields": {"__discountCodeApp": {"data": codes}}
        })
    }

    fn raw_product(published_at: &str, compare_at: &str, variants: Vec<Value>) -> Value {
        json!({
            "id": "gid://shopify/Product/1", "handle": "a2345", "title": "Anker Prime Ladegerät",
            "vendor": "Anker", "productType": "", "description": "d",
            "descriptionHtml": "<p>d</p>", "availableForSale": true,
            "createdAt": "2024-01-01T00:00:00Z", "updatedAt": "2024-01-02T00:00:00Z",
            "publishedAt": published_at, "tags": [], "seo": {"title": null, "description": null},
            "price": {"amount": "159.99", "currencyCode": "GBP"},
            "compareAtPrice": {"amount": compare_at, "currencyCode": "GBP"},
            "featuredImage": {"url": "u", "altText": null, "width": 1, "height": 1},
            "images": [{"url": "u", "altText": null, "width": 1, "height": 1}],
            "media": [], "options": [{"id": "gid://shopify/ProductOption/7", "name": "Colour",
                                      "values": ["Black"]}],
            "variants": variants, "sellingPlanGroups": [], "metafields": {}
        })
    }

    fn record(raw: Value) -> Result<ProductRecord, NormalizeError> {
        ProductRecord::from_raw(serde_json::from_value(raw).unwrap())
    }

    #[test]
    fn a_zero_compare_at_price_is_no_markdown_and_a_real_one_crosses() {
        let unmarked = record(raw_product("2024-01-03T00:00:00Z", "0.0", vec![])).unwrap();
        let marked = record(raw_product("2024-01-03T00:00:00Z", "199.99", vec![])).unwrap();

        assert_eq!(unmarked.compare_at_price, None);
        assert_eq!(marked.compare_at_price, Some(gbp(19999)));
    }

    #[test]
    fn a_published_at_that_is_not_an_instant_fails_with_its_value() {
        let error = record(raw_product("2024-01-03", "0.0", vec![])).unwrap_err();

        assert!(matches!(
            error,
            NormalizeError::PublishedAtNotAnInstant { value, .. } if value == "2024-01-03"
        ));
    }

    #[test]
    fn an_unweighed_variant_has_no_weight_and_a_weighed_one_keeps_its_kilograms() {
        let record = record(raw_product(
            "2024-01-03T00:00:00Z",
            "0.0",
            vec![raw_variant(0.0, vec![]), raw_variant(1.052, vec![])],
        ))
        .unwrap();

        assert_eq!(record.variants[0].weight, None);
        assert_eq!(record.variants[1].weight, Some(1.052));
    }

    #[test]
    fn a_fixed_amount_code_crosses_as_money_in_its_own_currency() {
        let discount =
            Discount::from_raw(serde_json::from_value(discount_code()).unwrap()).unwrap();

        assert_eq!(discount.code, "WS24A2345GU");
        assert_eq!(discount.kind, "WS24");
        assert_eq!(discount.value, DiscountValue::FixedAmount(gbp(-3200)));
        assert_eq!(discount.price, gbp(12799));
        assert_eq!(discount.saving, gbp(3200));
        assert_eq!(discount.variant_shopify_id, 59852742099025);
        assert_eq!(discount.sales_channels, None);
    }

    #[test]
    fn an_app_only_code_keeps_its_channel_and_an_open_ended_one_has_no_end() {
        let mut code = discount_code();
        code["sales_channel"] = json!(["app"]);
        code["ends_at"] = json!(null);

        let discount = Discount::from_raw(serde_json::from_value(code).unwrap()).unwrap();

        assert_eq!(discount.sales_channels, Some(vec!["app".to_string()]));
        assert_eq!(discount.ends_at, None);
    }

    #[test]
    fn an_unknown_value_type_fails_rather_than_guessing_the_unit() {
        let mut code = discount_code();
        code["value_type"] = json!("buy_x_get_y");

        let error = Discount::from_raw(serde_json::from_value(code).unwrap()).unwrap_err();

        assert_eq!(
            error,
            NormalizeError::UnknownDiscountValueType {
                code: "WS24A2345GU".to_string(),
                value_type: "buy_x_get_y".to_string(),
            }
        );
    }

    #[test]
    fn a_variants_codes_reach_its_discounts_and_a_variant_without_any_has_none() {
        let record = record(raw_product(
            "2024-01-03T00:00:00Z",
            "0.0",
            vec![
                raw_variant(1.0, vec![discount_code()]),
                raw_variant(1.0, vec![]),
            ],
        ))
        .unwrap();

        assert_eq!(record.variants[0].discounts.len(), 1);
        assert_eq!(record.variants[1].discounts, vec![]);
    }

    #[test]
    fn the_record_survives_the_round_trip_the_next_stage_makes() {
        let record = record(raw_product(
            "2024-01-03T00:00:00Z",
            "199.99",
            vec![raw_variant(1.052, vec![discount_code()])],
        ))
        .unwrap();

        let json = serde_json::to_string(&record).unwrap();

        assert_eq!(
            serde_json::from_str::<ProductRecord>(&json).unwrap(),
            record
        );
    }
}
