use super::*;
use retailer_sauce::{availability::Availability, item_condition::ItemCondition};

#[derive(Clone, Copy, poem_openapi::Enum, async_graphql::Enum, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityKind {
    #[oai(rename = "available")]
    Available,
    #[oai(rename = "unavailable")]
    Unavailable,
    #[oai(rename = "in_stock")]
    InStock,
    #[oai(rename = "out_of_stock")]
    OutOfStock,
    #[oai(rename = "sold_out")]
    SoldOut,
    #[oai(rename = "other")]
    Other,
}

#[derive(Clone, Object, SimpleObject, serde::Serialize)]
pub struct OfferAvailability {
    pub kind: AvailabilityKind,
    pub raw: Option<String>,
}

impl OfferAvailability {
    pub(super) fn from_record(value: serde_json::Value) -> Result<Self, ()> {
        let value: Availability = serde_json::from_value(value).map_err(|_| ())?;
        value.guard().map_err(|_| ())?;
        let (kind, raw) = match value {
            Availability::Available => (AvailabilityKind::Available, None),
            Availability::Unavailable => (AvailabilityKind::Unavailable, None),
            Availability::InStock => (AvailabilityKind::InStock, None),
            Availability::OutOfStock => (AvailabilityKind::OutOfStock, None),
            Availability::SoldOut => (AvailabilityKind::SoldOut, None),
            Availability::Other(raw) => (AvailabilityKind::Other, Some(raw)),
        };
        Ok(Self { kind, raw })
    }
}

#[derive(Clone, Copy, poem_openapi::Enum, async_graphql::Enum, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemConditionKind {
    #[oai(rename = "new")]
    New,
    #[oai(rename = "other")]
    Other,
}

#[derive(Clone, Object, SimpleObject, serde::Serialize)]
pub struct OfferItemCondition {
    pub kind: ItemConditionKind,
    pub raw: Option<String>,
}

impl OfferItemCondition {
    pub(super) fn from_record(value: serde_json::Value) -> Result<Self, ()> {
        let value: ItemCondition = serde_json::from_value(value).map_err(|_| ())?;
        value.guard().map_err(|_| ())?;
        let (kind, raw) = match value {
            ItemCondition::New => (ItemConditionKind::New, None),
            ItemCondition::Other(raw) => (ItemConditionKind::Other, Some(raw)),
        };
        Ok(Self { kind, raw })
    }
}

#[derive(Clone, Default, Object, SimpleObject, serde::Serialize)]
pub struct OfferContent {
    pub description: Option<String>,
}

pub(super) fn was_price(
    amount: Option<i64>,
    currency: Option<String>,
    current_amount: i64,
    current_currency: &str,
) -> Result<Option<OfferPrice>, ()> {
    match (amount, currency) {
        (None, None) => Ok(None),
        (Some(amount), Some(currency)) => Ok((amount > current_amount
            && currency == current_currency)
            .then(|| OfferPrice {
                amount: amount.to_string(),
                currency,
            })),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn enums_preserve_unknown_values_and_prices_only_show_comparable_discounts() {
        for value in [
            "available",
            "unavailable",
            "in_stock",
            "out_of_stock",
            "sold_out",
        ] {
            let fact = OfferAvailability::from_record(json!(value)).unwrap();
            assert_eq!(
                serde_json::to_value(fact).unwrap(),
                json!({"kind":value,"raw":null})
            );
        }
        for raw in [
            " https://schema.org/PreOrder ž ",
            "http://schema.org/OutOfStock",
            "https://schema.org/InStock",
        ] {
            let fact = OfferAvailability::from_record(json!({"other":raw})).unwrap();
            assert_eq!(
                serde_json::to_value(fact).unwrap(),
                json!({"kind":"other","raw":raw})
            );
        }
        for raw in [
            " Refurbished ž ",
            "http://schema.org/NewCondition",
            "https://schema.org/NewCondition",
        ] {
            let fact = OfferItemCondition::from_record(json!({"other":raw})).unwrap();
            assert_eq!(
                serde_json::to_value(fact).unwrap(),
                json!({"kind":"other","raw":raw})
            );
        }
        assert_eq!(
            serde_json::to_value(OfferItemCondition::from_record(json!("new")).unwrap()).unwrap(),
            json!({"kind":"new","raw":null})
        );
        assert!(OfferAvailability::from_record(json!("guessed")).is_err());
        assert!(OfferItemCondition::from_record(json!({"other":"\0"})).is_err());
        assert!(
            was_price(Some(20), Some("USD".into()), 10, "USD")
                .unwrap()
                .is_some()
        );
        for (amount, currency) in [(10, "USD"), (9, "USD"), (20, "EUR")] {
            assert!(
                was_price(Some(amount), Some(currency.into()), 10, "USD")
                    .unwrap()
                    .is_none()
            );
        }
        assert!(was_price(None, Some("USD".into()), 10, "USD").is_err());
    }
}
