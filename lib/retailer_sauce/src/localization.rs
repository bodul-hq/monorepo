//! Selected facts from the downloaded page, independent of extraction diagnostics.
use everyday::prelude::CountryCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TargetMarket {
    Country { code: CountryCode },
    Region { label: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "WireLocalization")]
pub struct Localization {
    pub target_market: Option<TargetMarket>,
    pub language: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireLocalization {
    target_market: Option<TargetMarket>,
    language: Option<String>,
}
impl TryFrom<WireLocalization> for Localization {
    type Error = String;
    fn try_from(value: WireLocalization) -> Result<Self, Self::Error> {
        let result = Self {
            target_market: value.target_market,
            language: value.language,
        };
        result.guard()?;
        Ok(result)
    }
}
impl Localization {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(TargetMarket::Region { label }) = &self.target_market
            && (label.trim().is_empty() || label.len() > 128 || label.chars().any(char::is_control))
        {
            return Err(
                "market region must be a nonblank label of at most 128 bytes without controls"
                    .into(),
            );
        }
        if let Some(language) = &self.language {
            validate_language(language)?;
        }
        Ok(())
    }
}

/// Supported syntax is ICU's Unicode BCP47 locale grammar. Preserve the source spelling.
/// Unsupported grandfathered/extlang forms remain unknown during extraction.
pub fn validate_language(value: &str) -> Result<(), String> {
    if value.len() > 128 || value.contains('_') || value.parse::<icu_locale_core::Locale>().is_err()
    {
        return Err("unsupported or malformed language tag".into());
    }
    Ok(())
}

/// Missing imports mean no observation. An explicit null object is invalid.
pub fn observation<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Localization>, D::Error> {
    Localization::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observations_preserve_full_tags_and_validate_market_scope() {
        for value in [
            serde_json::json!({"target_market":{"kind":"country","code":"CA"},"language":"en-AU"}),
            serde_json::json!({"target_market":{"kind":"region","label":"Europe"},"language":null}),
            serde_json::json!({}),
        ] {
            let parsed: Localization = serde_json::from_value(value).unwrap();
            parsed.guard().unwrap();
        }
        for value in [
            serde_json::json!(null),
            serde_json::json!({"language":"fr_FR"}),
            serde_json::json!({"target_market":{"kind":"country","code":"EU"}}),
            serde_json::json!({"target_market":{"kind":"region","label":"\0"}}),
        ] {
            assert!(serde_json::from_value::<Localization>(value).is_err());
        }
    }
}
