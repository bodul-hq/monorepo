use retailer_sauce::{
    availability::Availability, item_condition::ItemCondition, structuralized::content::Content,
};
use serde::{Deserialize, Deserializer, de::Error};

#[derive(Deserialize)]
struct Status {
    kind: String,
    raw: Option<String>,
}

fn read_status<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Status>, D::Error> {
    Option::<serde_json::Value>::deserialize(deserializer)?
        .map(|value| {
            if !value.is_object() {
                return Err(D::Error::custom("status must be an object or null"));
            }
            serde_json::from_value(value).map_err(D::Error::custom)
        })
        .transpose()
}

pub(super) fn read_content<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Content, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    if !value.is_object() {
        return Err(D::Error::custom("content must be an object"));
    }
    let content: Content = serde_json::from_value(value).map_err(D::Error::custom)?;
    content.guard().map_err(D::Error::custom)?;
    Ok(content)
}

pub(super) fn read_availability<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Availability>, D::Error> {
    read_status(deserializer)?
        .map(|value| {
            let availability = match (value.kind.as_str(), value.raw) {
                ("available", None) => Availability::Available,
                ("unavailable", None) => Availability::Unavailable,
                ("in_stock", None) => Availability::InStock,
                ("out_of_stock", None) => Availability::OutOfStock,
                ("sold_out", None) => Availability::SoldOut,
                ("other", Some(raw)) => Availability::Other(raw),
                _ => return Err(D::Error::custom("invalid availability")),
            };
            availability.guard().map_err(D::Error::custom)?;
            Ok(availability)
        })
        .transpose()
}

pub(super) fn read_condition<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ItemCondition>, D::Error> {
    read_status(deserializer)?
        .map(|value| {
            let condition = match (value.kind.as_str(), value.raw) {
                ("new", None) => ItemCondition::New,
                ("other", Some(raw)) => ItemCondition::Other(raw),
                _ => return Err(D::Error::custom("invalid item condition")),
            };
            condition.guard().map_err(D::Error::custom)?;
            Ok(condition)
        })
        .transpose()
}
