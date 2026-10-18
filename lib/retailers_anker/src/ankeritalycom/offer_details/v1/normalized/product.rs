use super::super::raw;
use super::{NormalizeError, decimal, decimal_money, money};
use chrono::{DateTime, Utc};
use money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductRecord {
    pub id: String,
    pub handle: String,
    // Commented out: equal to `handle` on every page.
    // pub slug: String,
    // Commented out: `/products/` followed by `handle` on every page.
    // pub path: String,
    pub title: String,
    pub description: String,
    // Commented out: the same copy with markup; adds no value for now.
    // pub description_html: String,
    /// The sub-brand - `Anker`, `Soundcore`, `Nebula` - in the storefront's own spellings.
    pub vendor: String,
    pub product_type: String,
    pub updated_at: String,
    pub available_for_sale: bool,
    // Commented out: not needed for now.
    // pub tags: Vec<String>,
    pub price: Money,
    // Commented out: its minimum and maximum are the variants' own prices on every page.
    // pub price_range: PriceRange,
    /// `None` where the product is not marked down.
    pub compare_at_price: Option<Money>,
    pub featured_image: ProductImage,
    pub images: Vec<ProductImage>,
    pub seo: ProductSeo,
    // Commented out: every entry is an image, mirroring `images` one for one.
    // pub media: Vec<Media>,
    pub options: Vec<ProductOption>,
    pub variants: Vec<ProductVariant>,
    // Commented out: an untyped, per-namespace CMS/storefront-configuration blob - see
    // `raw::nextdata::product::ProductMetafields`. The specification blocks and the judge.me
    // rating inside it are the facts worth lifting out of it by name.
    // pub metafields: ProductMetafields,
}

impl ProductRecord {
    pub(crate) fn from_raw(
        raw: raw::nextdata::product::Product,
    ) -> Result<ProductRecord, NormalizeError> {
        let compare_at = raw.compare_at_price_range.min_variant_price;

        Ok(ProductRecord {
            id: raw.id,
            handle: raw.handle,
            // slug: raw.slug,
            // path: raw.path,
            title: raw.title,
            description: raw.description,
            // description_html: raw.description_html,
            vendor: raw.vendor,
            product_type: raw.product_type,
            updated_at: raw.updated_at,
            available_for_sale: raw.available_for_sale,
            // tags: raw.tags,
            price: money(raw.price.amount, &raw.price.currency_code, "price")?,
            // price_range: PriceRange::from_raw(raw.price_range),
            compare_at_price: compare_at_price(
                &compare_at.amount,
                &compare_at.currency_code,
                "compareAtPriceRange.minVariantPrice",
            )?,
            featured_image: ProductImage::from_raw(raw.featured_image),
            images: raw.images.into_iter().map(ProductImage::from_raw).collect(),
            seo: ProductSeo::from_raw(raw.seo),
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
            // metafields: ProductMetafields::from_raw(raw.metafields),
        })
    }
}

// `"0.0"` is the storefront's sentinel for "not marked down", rather than a real zero price.
fn compare_at_price(
    amount: &str,
    currency: &str,
    field: &str,
) -> Result<Option<Money>, NormalizeError> {
    let amount = decimal(amount, field)?;

    (amount != 0.0)
        .then(|| money(amount, currency, field))
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductImage {
    pub url: String,
    // Commented out: not needed for now.
    // pub alt_text: Option<String>,
    // pub width: i64,
    // pub height: i64,
}

impl ProductImage {
    fn from_raw(raw: raw::nextdata::product::Image) -> ProductImage {
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
    fn from_raw(raw: raw::nextdata::product::ProductSeo) -> ProductSeo {
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
    pub values: Vec<OptionValue>,
}

impl ProductOption {
    fn from_raw(raw: raw::nextdata::product::ProductOption) -> ProductOption {
        ProductOption {
            id: raw.id,
            name: raw.name,
            values: raw.values.into_iter().map(OptionValue::from_raw).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionValue {
    pub label: String,
    pub hex_colors: Option<Vec<String>>,
}

impl OptionValue {
    fn from_raw(raw: raw::nextdata::product::OptionValue) -> OptionValue {
        OptionValue {
            label: raw.label,
            hex_colors: raw.hex_colors,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductVariant {
    pub id: String,
    pub title: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub quantity_available: i64,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    pub image: ProductImage,
    // Commented out: the Pages Router rendering of `selected_options`, whose `id` is the variant's
    // own gid rather than an option's.
    // pub options: Vec<ProductOption>,
    pub selected_options: Vec<SelectedOption>,
    // Commented out: the same entries as the discount app's namespace, lifted into `discounts`.
    // pub coupons: Vec<Discount>,
    // Commented out: a copy of the product record this variant belongs to.
    // pub product: ProductRecord,
    // Commented out: untyped, per-namespace CMS/storefront configuration - see
    // `raw::nextdata::product::VariantMetafields`. The discount app's namespace is lifted into
    // `discounts`.
    // pub metafields: VariantMetafields,
    pub discounts: Vec<Discount>,
}

impl ProductVariant {
    fn from_raw(raw: raw::nextdata::product::Variant) -> Result<ProductVariant, NormalizeError> {
        Ok(ProductVariant {
            id: raw.id,
            title: raw.title,
            sku: raw.sku,
            barcode: raw.barcode,
            available_for_sale: raw.available_for_sale,
            currently_not_in_stock: raw.currently_not_in_stock,
            quantity_available: raw.quantity_available,
            price: money(
                raw.price.amount,
                &raw.price.currency_code,
                "variants[].price",
            )?,
            compare_at_price: raw
                .compare_at_price
                .map(|price| {
                    compare_at_price(
                        &price.amount,
                        &price.currency_code,
                        "variants[].compareAtPrice",
                    )
                })
                .transpose()?
                .flatten(),
            image: ProductImage::from_raw(raw.image),
            // options: raw.options.into_iter().map(ProductOption::from_raw).collect(),
            selected_options: raw
                .selected_options
                .into_iter()
                .map(SelectedOption::from_raw)
                .collect(),
            // coupons: raw.coupons.into_iter().map(Discount::from_raw).collect(),
            // product: raw.product,
            // metafields: VariantMetafields::from_raw(raw.metafields),
            discounts: discounts(raw.metafields.discount_code_app)?,
        })
    }
}

fn discounts(
    raw: Option<raw::nextdata::product::DiscountCodeApp>,
) -> Result<Vec<Discount>, NormalizeError> {
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
    fn from_raw(raw: raw::nextdata::product::SelectedOption) -> SelectedOption {
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
    fn from_raw(raw: raw::nextdata::product::DiscountCode) -> Result<Discount, NormalizeError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::{self, Value, json};

    fn eur(amount: i64) -> Money {
        Money::from_raw(amount as f64 / 100.0, "EUR").unwrap()
    }

    fn discount_code() -> Value {
        json!({
            "title": "WS24FG3SG6TD", "starts_at": "2025-10-22T05:36:56.000Z", "ends_at": null,
            "variant_shopify_id": 45124210524297i64, "value_type": "fixed_amount",
            "value": "-16.0", "currency": "EUR", "fixed_value": "16.0", "value_style": "16 €",
            "variant_price4wscode": 33.99, "discount_type": "WS24", "amazon_deal": false,
            "sales_channel": null, "cdi_id": 127766, "cdi_updated_at": "2026-09-07T10:25:51.914Z"
        })
    }

    fn raw_variant(compare_at: Value, codes: Vec<Value>) -> Value {
        json!({
            "title": "Nero", "quantityAvailable": 3893, "currentlyNotInStock": false,
            "availableForSale": true, "barcode": "194644248444",
            "image": {"url": "u", "altText": null, "width": 1, "height": 1},
            "compareAtPrice": compare_at, "id": "gid://shopify/ProductVariant/45124210524297",
            "sku": "A121C311", "price": {"amount": 49.99, "currencyCode": "EUR"},
            "options": [{"__typename": "MultipleChoiceOption",
                         "id": "gid://shopify/ProductVariant/45124210524297",
                         "name": "color", "values": [{"label": "Nero"}]}],
            "selectedOptions": [{"name": "Color", "value": "Nero"}],
            "coupons": codes.clone(), "product": {},
            "metafields": {"__discountCodeApp": {"data": codes}}
        })
    }

    fn raw_product(compare_at: &str, variants: Vec<Value>) -> Value {
        json!({
            "id": "gid://shopify/Product/1", "handle": "a121c", "slug": "a121c",
            "path": "/products/a121c", "title": "Anker Zolo Caricatore", "description": "d",
            "descriptionHtml": "<p>d</p>", "vendor": "Anker IT", "productType": "",
            "updatedAt": "2026-01-01T00:00:00Z", "availableForSale": true, "tags": [],
            "price": {"amount": 49.99, "currencyCode": "EUR"},
            "priceRange": {"minVariantPrice": {"amount": "49.99", "currencyCode": "EUR"},
                           "maxVariantPrice": {"amount": "49.99", "currencyCode": "EUR"}},
            "compareAtPriceRange": {"minVariantPrice": {"amount": compare_at,
                                                         "currencyCode": "EUR"}},
            "featuredImage": {"url": "u", "altText": null, "width": 1, "height": 1},
            "images": [{"url": "u", "altText": null, "width": 1, "height": 1}],
            "seo": {"title": null, "description": null}, "media": [],
            "options": [{"__typename": "MultipleChoiceOption",
                         "id": "gid://shopify/ProductOption/10843221262473", "name": "color",
                         "values": [{"label": "Nero", "hexColors": ["#000000"]}]}],
            "variants": variants, "metafields": {}
        })
    }

    fn record(raw: Value) -> Result<ProductRecord, NormalizeError> {
        ProductRecord::from_raw(serde_json::from_value(raw).unwrap())
    }

    #[test]
    fn a_zero_compare_at_range_is_no_markdown_and_a_real_one_crosses() {
        assert_eq!(
            record(raw_product("0.0", vec![])).unwrap().compare_at_price,
            None
        );
        assert_eq!(
            record(raw_product("69.99", vec![]))
                .unwrap()
                .compare_at_price,
            Some(eur(6999))
        );
    }

    #[test]
    fn a_quoted_variant_compare_at_price_crosses_and_a_malformed_one_fails() {
        let marked = record(raw_product(
            "0.0",
            vec![raw_variant(
                json!({"amount": "69.99", "currencyCode": "EUR"}),
                vec![],
            )],
        ))
        .unwrap();
        assert_eq!(marked.variants[0].compare_at_price, Some(eur(6999)));

        let error = record(raw_product(
            "0.0",
            vec![raw_variant(
                json!({"amount": "69,99", "currencyCode": "EUR"}),
                vec![],
            )],
        ))
        .unwrap_err();
        assert!(matches!(
            error,
            NormalizeError::PriceNotANumber { value, .. } if value == "69,99"
        ));
    }

    #[test]
    fn the_discount_apps_codes_cross_and_the_coupon_copy_does_not() {
        let record = record(raw_product(
            "0.0",
            vec![raw_variant(json!(null), vec![discount_code()])],
        ))
        .unwrap();
        let variant = &record.variants[0];

        assert_eq!(variant.discounts.len(), 1);
        assert_eq!(variant.discounts[0].code, "WS24FG3SG6TD");
        assert_eq!(
            variant.discounts[0].value,
            DiscountValue::FixedAmount(eur(-1600))
        );
        assert_eq!(variant.discounts[0].price, eur(3399));
        assert_eq!(variant.discounts[0].ends_at, None);
        assert_eq!(variant.discounts[0].sales_channels, None);

        let output = serde_json::to_value(variant).unwrap();
        for dropped in ["coupons", "product", "options", "metafields"] {
            assert!(output.get(dropped).is_none(), "{dropped} crossed");
        }
    }

    #[test]
    fn a_percentage_code_keeps_its_signed_points_and_an_unknown_type_fails() {
        let mut code = discount_code();
        code["value_type"] = json!("percentage");
        code["value"] = json!("-20.0");
        let discount = Discount::from_raw(serde_json::from_value(code.clone()).unwrap()).unwrap();
        assert_eq!(discount.value, DiscountValue::Percentage(-20.0));

        code["value_type"] = json!("buy_x_get_y");
        let error = Discount::from_raw(serde_json::from_value(code).unwrap()).unwrap_err();
        assert_eq!(
            error,
            NormalizeError::UnknownDiscountValueType {
                code: "WS24FG3SG6TD".to_string(),
                value_type: "buy_x_get_y".to_string(),
            }
        );
    }

    #[test]
    fn the_record_survives_the_round_trip_the_next_stage_makes() {
        let record = record(raw_product(
            "69.99",
            vec![raw_variant(
                json!({"amount": "69.99", "currencyCode": "EUR"}),
                vec![discount_code()],
            )],
        ))
        .unwrap();

        let json = serde_json::to_string(&record).unwrap();

        assert_eq!(
            serde_json::from_str::<ProductRecord>(&json).unwrap(),
            record
        );
    }
}
