//! The storefront keeps key features, specifications, discount codes, member pricing and
//! per-variant key parameters in untyped metafield blobs holding dozens of unrelated namespaces.
//! Each fact is lifted by its path instead of typing the whole blob, so a namespace nobody reads
//! cannot fail a document.

use super::{NormalizeError, money};
use chrono::{DateTime, Utc};
use html_extractor::serde_json::Value;
use money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyFeature {
    pub title: String,
    pub body: String,
}

impl KeyFeature {
    fn from_entry(entry: &Value) -> Result<KeyFeature, NormalizeError> {
        Ok(KeyFeature {
            title: string_field(entry, "key", "key_features")?,
            body: string_field(entry, "value", "key_features")?,
        })
    }
}

pub(super) fn key_features(product_metafields: &Value) -> Result<Vec<KeyFeature>, NormalizeError> {
    let Some(list) = product_metafields
        .get("productInfos")
        .and_then(|infos| infos.get("key_features"))
        .and_then(|features| features.get("list"))
    else {
        return Ok(vec![]);
    };

    list_at(list, "productInfos.key_features.list")?
        .iter()
        .map(KeyFeature::from_entry)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Specification {
    pub name: String,
    pub value: String,
}

impl Specification {
    fn from_entry(entry: &Value) -> Result<Specification, NormalizeError> {
        Ok(Specification {
            name: string_field(entry, "name", "productSpecs")?,
            value: string_field(entry, "presentation", "productSpecs")?,
        })
    }
}

// The displayed specifications belong to the page's metafields, not the product's.
pub(super) fn specifications(
    page_metafields: &Value,
) -> Result<Vec<Specification>, NormalizeError> {
    let Some(items) = page_metafields
        .get("components")
        .and_then(|components| components.get("productSpecs"))
        .and_then(|specs| specs.get("items"))
    else {
        return Ok(vec![]);
    };

    list_at(items, "components.productSpecs.items")?
        .iter()
        .map(Specification::from_entry)
        .collect()
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscountValue {
    /// Signed percentage points as supplied by the page: `-25.0` means 25% off.
    Percentage(f64),
    FixedAmount(Money),
}

impl Discount {
    fn from_entry(entry: &Value) -> Result<Discount, NormalizeError> {
        let missing = |name: &str| NormalizeError::MissingDiscountField {
            field: name.to_string(),
            entry: entry.clone(),
        };

        let string = |name: &str| {
            entry
                .get(name)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| missing(name))
        };

        let quoted_number = |name: &str| {
            let raw = string(name)?;

            number(&raw, name)
        };

        let instant = |name: &str| {
            let raw = string(name)?;

            DateTime::parse_from_rfc3339(&raw)
                .map_err(|source| NormalizeError::NotAnInstant {
                    field: name.to_string(),
                    value: raw,
                    source,
                })
                .map(|instant| instant.with_timezone(&Utc))
        };

        let currency = string("currency")?;

        let value = match string("value_type")?.as_str() {
            "percentage" => DiscountValue::Percentage(quoted_number("value")?),
            "fixed_amount" => {
                DiscountValue::FixedAmount(money(quoted_number("value")?, &currency, "value")?)
            }
            other => {
                return Err(NormalizeError::UnknownDiscountValueType {
                    value_type: other.to_string(),
                    entry: entry.clone(),
                });
            }
        };

        let price_amount = entry
            .get("variant_price4wscode")
            .and_then(Value::as_f64)
            .ok_or_else(|| missing("variant_price4wscode"))?;

        let variant_shopify_id = entry
            .get("variant_shopify_id")
            .and_then(Value::as_i64)
            .ok_or_else(|| missing("variant_shopify_id"))?;

        let amazon_deal = entry
            .get("amazon_deal")
            .and_then(Value::as_bool)
            .ok_or_else(|| missing("amazon_deal"))?;

        let ends_at = match entry.get("ends_at") {
            Some(Value::Null) | None => None,
            Some(_) => Some(instant("ends_at")?),
        };

        Ok(Discount {
            code: string("title")?,
            kind: string("discount_type")?,
            value,
            price: money(price_amount, &currency, "variant_price4wscode")?,
            saving: money(quoted_number("fixed_value")?, &currency, "fixed_value")?,
            starts_at: instant("starts_at")?,
            ends_at,
            variant_shopify_id,
            amazon_deal,
        })
    }
}

pub(super) fn discounts(variant_metafields: &Value) -> Result<Vec<Discount>, NormalizeError> {
    let Some(entries) = variant_metafields.get("discountCodeApp") else {
        return Ok(vec![]);
    };

    list_at(entries, "discountCodeApp")?
        .iter()
        .map(Discount::from_entry)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberPrice {
    pub participates: bool,
    pub discount_subject: String,
    /// `None` where the record names no member-only amount.
    pub target_price: Option<Money>,
}

impl MemberPrice {
    fn from_entry(entry: &Value, currency: &str) -> Result<MemberPrice, NormalizeError> {
        let participates = entry
            .get("participate")
            .and_then(Value::as_bool)
            .ok_or_else(|| missing_member_price_field(entry, "participate"))?;

        let discount_subject = entry
            .get("discount_subject")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| missing_member_price_field(entry, "discount_subject"))?;

        Ok(MemberPrice {
            participates,
            discount_subject,
            target_price: target_price(entry, currency)?,
        })
    }
}

fn target_price(entry: &Value, currency: &str) -> Result<Option<Money>, NormalizeError> {
    let (price_type, price_value) = match (entry.get("price_type"), entry.get("price_value")) {
        (None, None) => return Ok(None),
        (Some(price_type), Some(price_value)) => (price_type, price_value),
        _ => {
            return Err(NormalizeError::UnpairedMemberPrice {
                entry: entry.clone(),
            });
        }
    };

    let price_type = price_type
        .as_str()
        .ok_or_else(|| missing_member_price_field(entry, "price_type"))?;

    if price_type != "target_price" {
        return Err(NormalizeError::UnknownMemberPriceType {
            price_type: price_type.to_string(),
            entry: entry.clone(),
        });
    }

    let raw = price_value
        .as_str()
        .ok_or_else(|| missing_member_price_field(entry, "price_value"))?;

    money(
        number(raw, "price_value")?,
        currency,
        "memberPrice.price_value",
    )
    .map(Some)
}

fn missing_member_price_field(entry: &Value, field: &str) -> NormalizeError {
    NormalizeError::MissingMemberPriceField {
        field: field.to_string(),
        entry: entry.clone(),
    }
}

// The amount carries no currency of its own, so it takes the one the variant's prices take.
pub(super) fn member_price(
    variant_metafields: &Value,
    currency: &str,
) -> Result<Option<MemberPrice>, NormalizeError> {
    variant_metafields
        .get("memberPrice")
        .map(|entry| MemberPrice::from_entry(entry, currency))
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyParameter {
    pub name: String,
    pub value: KeyParameterValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyParameterValue {
    Text(String),
    /// Per-port or per-mode figures, e.g. `USB-C` / `5V⎓3A / 9V⎓3A`.
    Labelled(Vec<LabelledValue>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelledValue {
    pub name: String,
    pub label: String,
}

impl KeyParameter {
    fn from_row(row: &Value) -> Result<KeyParameter, NormalizeError> {
        let name = string_field(row, "key", "key_parameter")?;

        let value = match (row.get("value"), row.get("valueList")) {
            (Some(_), None) => {
                KeyParameterValue::Text(string_field(row, "value", "key_parameter")?)
            }
            (None, Some(list)) => KeyParameterValue::Labelled(
                list_at(list, "key_parameter.list[].valueList")?
                    .iter()
                    .map(LabelledValue::from_entry)
                    .collect::<Result<_, _>>()?,
            ),
            _ => {
                return Err(NormalizeError::KeyParameterWithoutValue { row: row.clone() });
            }
        };

        Ok(KeyParameter { name, value })
    }
}

impl LabelledValue {
    fn from_entry(entry: &Value) -> Result<LabelledValue, NormalizeError> {
        Ok(LabelledValue {
            name: string_field(entry, "name", "valueList")?,
            label: string_field(entry, "label", "valueList")?,
        })
    }
}

pub(super) fn key_parameters(
    variant_metafields: &Value,
) -> Result<Vec<KeyParameter>, NormalizeError> {
    let Some(list) = variant_metafields
        .get("infos")
        .and_then(|infos| infos.get("key_parameter"))
        .and_then(|parameters| parameters.get("list"))
    else {
        return Ok(vec![]);
    };

    list_at(list, "infos.key_parameter.list")?
        .iter()
        .map(KeyParameter::from_row)
        .collect()
}

fn list_at<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, NormalizeError> {
    value.as_array().ok_or_else(|| NormalizeError::NotAList {
        path: path.to_string(),
    })
}

fn string_field(entry: &Value, name: &str, block: &str) -> Result<String, NormalizeError> {
    entry
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| NormalizeError::EntryFieldNotAString {
            block: block.to_string(),
            field: name.to_string(),
            entry: entry.clone(),
        })
}

fn number(raw: &str, field: &str) -> Result<f64, NormalizeError> {
    raw.parse::<f64>()
        .map_err(|source| NormalizeError::NotANumber {
            field: field.to_string(),
            value: raw.to_string(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::json;

    fn usd(amount: i64) -> Money {
        Money::from_raw(amount as f64 / 100.0, "USD").unwrap()
    }

    fn discount_entry() -> Value {
        json!({
            "title": "WS7DV2UNKYPL", "starts_at": "2026-08-31T07:00:00.000Z",
            "ends_at": "2026-09-14T06:59:59.000Z", "variant_shopify_id": 46383111110806i64,
            "value_type": "fixed_amount", "value": "-50.01", "currency": "USD",
            "fixed_value": "50.01", "value_style": "$50", "variant_price4wscode": 179.98,
            "discount_type": "DEAL", "amazon_deal": false, "cdi_id": 177315,
            "cdi_updated_at": "2026-08-31T17:18:36.662Z"
        })
    }

    #[test]
    fn a_fixed_amount_code_crosses_as_money_in_the_entrys_own_currency() {
        let discounts = discounts(&json!({"discountCodeApp": [discount_entry()]})).unwrap();

        assert_eq!(discounts.len(), 1);
        assert_eq!(discounts[0].code, "WS7DV2UNKYPL");
        assert_eq!(discounts[0].kind, "DEAL");
        assert_eq!(discounts[0].value, DiscountValue::FixedAmount(usd(-5001)));
        assert_eq!(discounts[0].price, usd(17998));
        assert_eq!(discounts[0].saving, usd(5001));
        assert_eq!(discounts[0].variant_shopify_id, 46383111110806);
    }

    #[test]
    fn a_percentage_code_keeps_its_signed_points() {
        let mut entry = discount_entry();
        entry["value_type"] = json!("percentage");
        entry["value"] = json!("-15");

        let discount = Discount::from_entry(&entry).unwrap();

        assert_eq!(discount.value, DiscountValue::Percentage(-15.0));
    }

    #[test]
    fn a_code_without_an_end_is_open_ended_rather_than_malformed() {
        let mut entry = discount_entry();
        entry.as_object_mut().unwrap().remove("ends_at");

        assert_eq!(Discount::from_entry(&entry).unwrap().ends_at, None);
    }

    #[test]
    fn an_unknown_value_type_fails_rather_than_guessing_the_unit() {
        let mut entry = discount_entry();
        entry["value_type"] = json!("buy_x_get_y");

        let error = Discount::from_entry(&entry).unwrap_err();

        assert!(matches!(
            error,
            NormalizeError::UnknownDiscountValueType { value_type, .. } if value_type == "buy_x_get_y"
        ));
    }

    #[test]
    fn a_variant_rendering_no_codes_has_none() {
        assert_eq!(discounts(&json!({})).unwrap(), vec![]);
        assert_eq!(discounts(&json!({"discountCodeApp": []})).unwrap(), vec![]);
    }

    #[test]
    fn a_member_record_without_an_amount_keeps_its_flags() {
        let member_price = member_price(
            &json!({"memberPrice": {"discount_subject": "discounted_price", "participate": true}}),
            "USD",
        )
        .unwrap();

        assert_eq!(
            member_price,
            Some(MemberPrice {
                participates: true,
                discount_subject: "discounted_price".to_string(),
                target_price: None,
            })
        );
    }

    #[test]
    fn a_target_price_crosses_in_the_variants_currency() {
        let member_price = member_price(
            &json!({"memberPrice": {
                "discount_subject": "discounted_price", "participate": true,
                "price_type": "target_price", "price_value": "175.99"
            }}),
            "USD",
        )
        .unwrap()
        .unwrap();

        assert_eq!(member_price.target_price, Some(usd(17599)));
    }

    #[test]
    fn an_unknown_member_price_type_fails() {
        let error = member_price(
            &json!({"memberPrice": {
                "discount_subject": "discounted_price", "participate": true,
                "price_type": "percentage_off", "price_value": "10"
            }}),
            "USD",
        )
        .unwrap_err();

        assert!(matches!(
            error,
            NormalizeError::UnknownMemberPriceType { price_type, .. } if price_type == "percentage_off"
        ));
    }

    #[test]
    fn a_member_price_type_without_its_value_fails() {
        for half in [
            json!({"price_type": "target_price"}),
            json!({"price_value": "175.99"}),
        ] {
            let mut entry = json!({"discount_subject": "discounted_price", "participate": true});
            entry
                .as_object_mut()
                .unwrap()
                .extend(half.as_object().unwrap().clone());

            let error = member_price(&json!({ "memberPrice": entry }), "USD").unwrap_err();

            assert!(
                matches!(error, NormalizeError::UnpairedMemberPrice { .. }),
                "{half}"
            );
        }
    }

    #[test]
    fn a_variant_without_a_member_record_has_none() {
        assert_eq!(member_price(&json!({}), "USD").unwrap(), None);
    }

    #[test]
    fn a_key_parameter_is_either_one_text_or_a_labelled_list() {
        let parameters = key_parameters(&json!({"infos": {"key_parameter": {"list": [
            {"key": "Total Capacity", "value": "26,250mAh"},
            {"key": "Total Input", "valueList": [
                {"name": "USB-C", "label": "5V⎓3A / 9V⎓3A"},
                {"name": "USB-C", "label": "28V⎓5.36A (150W Max)"}
            ]}
        ]}}}))
        .unwrap();

        assert_eq!(
            parameters,
            vec![
                KeyParameter {
                    name: "Total Capacity".to_string(),
                    value: KeyParameterValue::Text("26,250mAh".to_string()),
                },
                KeyParameter {
                    name: "Total Input".to_string(),
                    value: KeyParameterValue::Labelled(vec![
                        LabelledValue {
                            name: "USB-C".to_string(),
                            label: "5V⎓3A / 9V⎓3A".to_string(),
                        },
                        LabelledValue {
                            name: "USB-C".to_string(),
                            label: "28V⎓5.36A (150W Max)".to_string(),
                        },
                    ]),
                },
            ]
        );
    }

    #[test]
    fn a_key_parameter_row_with_neither_or_both_values_fails() {
        for row in [
            json!({"key": "Total Capacity"}),
            json!({"key": "Total Capacity", "value": "1", "valueList": []}),
        ] {
            let error =
                key_parameters(&json!({"infos": {"key_parameter": {"list": [row]}}})).unwrap_err();

            assert!(
                matches!(error, NormalizeError::KeyParameterWithoutValue { .. }),
                "{error}"
            );
        }
    }

    #[test]
    fn a_variant_whose_infos_carry_no_key_parameters_has_none() {
        assert_eq!(key_parameters(&json!({})).unwrap(), vec![]);
        assert_eq!(
            key_parameters(&json!({"infos": {"name": "Bundle kit"}})).unwrap(),
            vec![]
        );
    }

    #[test]
    fn specifications_are_read_from_the_page_and_key_features_from_the_product() {
        let page = json!({"components": {"productSpecs": {
            "title": "Specs",
            "items": [{"name": "Total Capacity", "presentation": "10,000mAh"}]
        }}});
        let product = json!({"productInfos": {"key_features": {"list": [
            {"key": "300W Total Output", "value": "Keeps two laptops running."}
        ]}}});

        assert_eq!(
            specifications(&page).unwrap(),
            vec![Specification {
                name: "Total Capacity".to_string(),
                value: "10,000mAh".to_string(),
            }]
        );
        assert_eq!(
            key_features(&product).unwrap(),
            vec![KeyFeature {
                title: "300W Total Output".to_string(),
                body: "Keeps two laptops running.".to_string(),
            }]
        );
        assert_eq!(specifications(&product).unwrap(), vec![]);
        assert_eq!(key_features(&page).unwrap(), vec![]);
    }

    #[test]
    fn a_specification_row_that_is_not_two_strings_fails() {
        let page = json!({"components": {"productSpecs": {"items": [{"name": "Weight"}]}}});

        let error = specifications(&page).unwrap_err();

        assert!(matches!(
            error,
            NormalizeError::EntryFieldNotAString { field, .. } if field == "presentation"
        ));
    }
}
