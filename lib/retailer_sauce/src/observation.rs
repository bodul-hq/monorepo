//! A missing legacy field is different from a fresh observation of an unknown value.

use html_extractor::serde_json::Value;
use html_extractor::serde_json::from_value;
use serde::de::DeserializeOwned;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Observation<T> {
    #[default]
    Missing,
    Observed(Option<T>),
}

impl<T> Observation<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Missing => None,
            Self::Observed(value) => value.as_ref(),
        }
    }

    pub fn into_value(self) -> Option<T> {
        match self {
            Self::Missing => None,
            Self::Observed(value) => value,
        }
    }

    pub fn resolve(self, previous: Option<T>) -> Option<T> {
        match self {
            Self::Missing => previous,
            Self::Observed(value) => value,
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Observation<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(Self::Observed)
    }
}

impl<T: Serialize> Serialize for Observation<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Missing => serializer.serialize_none(),
            Self::Observed(value) => value.serialize(serializer),
        }
    }
}

/// Unlike an ordinary Option field, use of this decoder requires the JSON key.
pub fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

/// JSON structs in the public fact contract must be objects, not Serde sequences.
pub fn object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;
    if !value.is_object() {
        return Err(DeError::custom("expected a JSON object"));
    }
    from_value(value).map_err(DeError::custom)
}

pub fn nullable_object<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    if !value.is_object() {
        return Err(DeError::custom("expected a JSON object or null"));
    }
    from_value(value).map(Some).map_err(DeError::custom)
}

pub fn object_observation<'de, D, T>(deserializer: D) -> Result<Observation<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    nullable_object(deserializer).map(Observation::Observed)
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;
    use serde_json::to_string;

    use super::*;
    #[derive(Serialize, Deserialize)]
    struct Example {
        #[serde(default, skip_serializing_if = "Observation::is_missing")]
        field: Observation<String>,
    }
    #[test]
    fn missing_null_and_value_survive_round_trip() {
        for (wire, expected) in [
            ("{}", Observation::Missing),
            (r#"{"field":null}"#, Observation::Observed(None)),
            (
                r#"{"field":"value"}"#,
                Observation::Observed(Some("value".into())),
            ),
        ] {
            let value: Example = from_str(wire).unwrap();
            assert_eq!(value.field, expected);
            assert_eq!(to_string(&value).unwrap(), wire);
        }
    }
}
