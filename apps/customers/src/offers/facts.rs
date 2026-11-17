use retailer_sauce::observation::required_nullable as retailer_sauce_observation_required_nullable;
use retailer_sauce::offer_content::Content as OfferContentContent;
use retailer_sauce::offer_content::guard as offer_content_guard;
use retailer_sauce::{
    availability::Availability, item_condition::ItemCondition, structuralized::content::Content,
};
use serde::Serialize;
use serde::{Deserialize, Deserializer, de::Error};
use serde_json::Value;
use serde_json::from_value as decode_json_value;
use serde_json::to_value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedContent {
    #[serde(deserialize_with = "retailer_sauce_observation_required_nullable")]
    pub language: Option<String>,
    pub title: String,
    #[serde(deserialize_with = "retailer_sauce_observation_required_nullable")]
    pub description: Option<String>,
    pub url: String,
}

pub(super) fn guard_content(
    content: &[LocalizedContent],
    language: &Option<String>,
) -> Result<(), String> {
    let legacy: Vec<_> = content
        .iter()
        .map(|entry| OfferContentContent {
            language: entry.language.clone(),
            title: entry.title.clone(),
            description: entry.description.clone(),
        })
        .collect();
    offer_content_guard(&legacy, language)?;
    if content
        .iter()
        .any(|entry| entry.url.len() > 2048 || entry.url.contains('\0'))
    {
        return Err("invalid content URL".into());
    }
    Ok(())
}

#[derive(Deserialize)]
struct Status {
    kind: String,
    raw: Option<String>,
}

fn read_status<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Status>, D::Error> {
    Option::<Value>::deserialize(deserializer)?
        .map(|value| {
            if !value.is_object() {
                return Err(D::Error::custom("status must be an object or null"));
            }
            decode_json_value(value).map_err(D::Error::custom)
        })
        .transpose()
}

/// During rolling deployment old API responses omit content or use one description object.
pub(super) fn normalize_content(value: &mut Value) -> Result<(), String> {
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .ok_or("title is required")?
        .to_owned();
    let language = match value
        .get("localization")
        .and_then(|value| value.get("language"))
    {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => return Err("language must be a string or null".into()),
    };
    let url = value
        .get("url")
        .and_then(Value::as_str)
        .ok_or("url is required")?
        .to_owned();
    let mut content: Vec<LocalizedContent> = match value.get("content") {
        Some(value) if value.is_array() => {
            let mut entries = value.clone();
            for entry in entries.as_array_mut().expect("array") {
                if let Some(object) = entry.as_object_mut() {
                    object
                        .entry("url")
                        .or_insert_with(|| Value::String(url.clone()));
                }
            }
            decode_json_value(entries).map_err(|error| error.to_string())?
        }
        value => {
            let description = match value {
                None => None,
                Some(value) if value.is_object() => {
                    decode_json_value::<Content>(value.clone())
                        .map_err(|error| error.to_string())?
                        .description
                }
                Some(_) => return Err("content must be an object or array".into()),
            };
            vec![LocalizedContent {
                title,
                language: language.clone(),
                description,
                url,
            }]
        }
    };
    guard_content(&content, &language)?;
    content.sort_by(|a, b| a.language.cmp(&b.language));
    value["content"] = to_value(content).expect("serializable content");
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
