//! The storefront keeps key features, specifications and discount codes in untyped metafield blobs
//! holding dozens of unrelated namespaces. Each fact is lifted by its path instead of typing the
//! whole blob, so a namespace nobody reads cannot fail a document.

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
    /// Text-only specification rows carry no separate label.
    pub name: Option<String>,
    pub value: String,
}

impl Specification {
    fn from_entry(entry: &Value) -> Result<Specification, NormalizeError> {
        Ok(Specification {
            name: specification_name(entry)?,
            value: string_field(entry, "presentation", "productSpecs")?,
        })
    }
}

// The CMS supports `name`, `type_name`, and unlabeled presentation-only rows.
// Preserve the full presentation instead of guessing a label from punctuation in the copy.
fn specification_name(entry: &Value) -> Result<Option<String>, NormalizeError> {
    match entry.get("name").or_else(|| entry.get("type_name")) {
        None => Ok(None),
        Some(Value::String(label)) => Ok(Some(label.clone())),
        Some(_) => Err(NormalizeError::SpecificationWithoutName {
            entry: entry.clone(),
        }),
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

    fn eur(amount: i64) -> Money {
        Money::from_raw(amount as f64 / 100.0, "EUR").unwrap()
    }

    fn discount_entry() -> Value {
        json!({
            "title": "WS24A121DED", "starts_at": "2026-09-06T22:00:00.000Z",
            "ends_at": "2026-09-28T21:59:59.000Z", "variant_shopify_id": 55534049001848i64,
            "value_type": "percentage", "value": "-25.0", "currency": "EUR",
            "fixed_value": "19.99", "value_style": "€25", "variant_price4wscode": 59.99,
            "discount_type": "WS24", "amazon_deal": false, "cdi_id": 147563,
            "cdi_updated_at": "2026-09-07T04:08:30.625Z"
        })
    }

    #[test]
    fn a_specification_labelled_type_name_crosses_under_that_label() {
        let specifications = specifications(&json!({"components": {"productSpecs": {"items": [
            {"name": "Size", "presentation": "63 x 35 x 35mm"},
            {"type_name": "Weight", "presentation": "30 g/1.1 oz"}
        ]}}}))
        .unwrap();

        assert_eq!(specifications[0].name.as_deref(), Some("Size"));
        assert_eq!(specifications[1].name.as_deref(), Some("Weight"));
        assert_eq!(specifications[1].value, "30 g/1.1 oz");
    }

    #[test]
    fn unlabeled_specifications_keep_the_complete_presentation() {
        let specifications = specifications(&json!({"components": {"productSpecs": {"items": [
            {"presentation": "Total Wattage: 20W "},
            {"presentation": "iPhone 13 / iPhone 13 Pro"}
        ]}}}))
        .unwrap();
        assert_eq!(specifications[0].name, None);
        assert_eq!(specifications[0].value, "Total Wattage: 20W ");
        assert_eq!(specifications[1].name, None);
        assert_eq!(specifications[1].value, "iPhone 13 / iPhone 13 Pro");
    }

    #[test]
    fn a_non_string_specification_label_is_still_an_error() {
        let error = specifications(&json!({"components": {"productSpecs": {"items": [
            {"name": 3, "presentation": "30 g"}
        ]}}}))
        .unwrap_err();
        assert!(matches!(
            error,
            NormalizeError::SpecificationWithoutName { .. }
        ));
    }

    #[test]
    fn a_percentage_code_keeps_its_signed_points_beside_the_price_and_saving_it_yields() {
        let discounts = discounts(&json!({"discountCodeApp": [discount_entry()]})).unwrap();

        assert_eq!(discounts.len(), 1);
        assert_eq!(discounts[0].code, "WS24A121DED");
        assert_eq!(discounts[0].kind, "WS24");
        assert_eq!(discounts[0].value, DiscountValue::Percentage(-25.0));
        assert_eq!(discounts[0].price, eur(5999));
        assert_eq!(discounts[0].saving, eur(1999));
        assert_eq!(discounts[0].variant_shopify_id, 55534049001848);
    }

    #[test]
    fn a_fixed_amount_code_crosses_as_money_in_the_entrys_own_currency() {
        let mut entry = discount_entry();
        entry["value_type"] = json!("fixed_amount");
        entry["value"] = json!("-20.0");

        let discount = Discount::from_entry(&entry).unwrap();

        assert_eq!(discount.value, DiscountValue::FixedAmount(eur(-2000)));
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
    fn specifications_are_read_from_the_page_and_key_features_from_the_product() {
        let page = json!({"components": {"productSpecs": {
            "title": "Specs",
            "items": [{"name": "Total Capacity", "presentation": "10,000mAh"}]
        }}});
        let product = json!({"productInfos": {"key_features": {"list": [
            {"key": "Ultra-Fast Charging", "value": "Charges a phone to 50% in 30 minutes."}
        ]}}});

        assert_eq!(
            specifications(&page).unwrap(),
            vec![Specification {
                name: Some("Total Capacity".to_string()),
                value: "10,000mAh".to_string(),
            }]
        );
        assert_eq!(
            key_features(&product).unwrap(),
            vec![KeyFeature {
                title: "Ultra-Fast Charging".to_string(),
                body: "Charges a phone to 50% in 30 minutes.".to_string(),
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
