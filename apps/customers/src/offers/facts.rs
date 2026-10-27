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

/// During rolling deployment old API responses omit content or use one description object.
pub(super) fn normalize_content(value: &mut serde_json::Value) -> Result<(), String> {
    use retailer_sauce::offer_content;
    let title = value
        .get("title")
        .and_then(serde_json::Value::as_str)
        .ok_or("title is required")?
        .to_owned();
    let language = match value
        .get("localization")
        .and_then(|value| value.get("language"))
    {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(value)) => Some(value.clone()),
        Some(_) => return Err("language must be a string or null".into()),
    };
    let mut content = match value.get("content") {
        Some(value) if value.is_array() => {
            serde_json::from_value::<Vec<offer_content::Content>>(value.clone())
                .map_err(|error| error.to_string())?
        }
        value => {
            let description = match value {
                None => None,
                Some(value) if value.is_object() => {
                    serde_json::from_value::<Content>(value.clone())
                        .map_err(|error| error.to_string())?
                        .description
                }
                Some(_) => return Err("content must be an object or array".into()),
            };
            offer_content::singleton(title, language.clone(), description)
        }
    };
    offer_content::guard(&content, &language)?;
    offer_content::sort(&mut content);
    value["content"] = serde_json::to_value(content).expect("serializable content");
    Ok(())
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
