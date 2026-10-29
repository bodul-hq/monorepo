//! Selected facts from the downloaded page, independent of extraction diagnostics.
use everyday::prelude::CountryCode;
use serde::{Deserialize, Serialize};

/// Stable identifiers for supported regional markets (not country codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionCode {
    #[serde(rename = "eu")]
    EuropeanUnion,
}
impl RegionCode {
    pub fn as_str(self) -> &'static str {
        "eu"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetMarket {
    Country {
        code: CountryCode,
    },
    Region {
        code: RegionCode,
    },
    /// Retains historical labels verbatim until an explicit mapping is known.
    #[serde(rename = "region")]
    LegacyRegion {
        label: String,
    },
}
impl<'de> Deserialize<'de> for TargetMarket {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Current {
            Country { code: CountryCode },
            Region { code: RegionCode },
        }
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Legacy {
            Region { label: String },
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Current(Current),
            Legacy(Legacy),
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Current(Current::Country { code }) => Self::Country { code },
            Wire::Current(Current::Region { code }) => Self::Region { code },
            Wire::Legacy(Legacy::Region { label }) => Self::LegacyRegion { label },
        })
    }
}
impl TargetMarket {
    /// Canonicalize only known evidence; never guess a code for another label.
    pub fn canonicalized(&self) -> Self {
        match self {
            Self::LegacyRegion { label } if label == "European Union" => Self::Region {
                code: RegionCode::EuropeanUnion,
            },
            _ => self.clone(),
        }
    }
    /// Used only by explicitly versioned historical export regeneration.
    pub fn legacy(&self) -> Self {
        match self {
            Self::Region {
                code: RegionCode::EuropeanUnion,
            } => Self::LegacyRegion {
                label: "European Union".into(),
            },
            _ => self.clone(),
        }
    }
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
    /// New producer boundaries require an agreed code; historical readers stay lossless.
    pub fn for_write(self) -> Result<Self, String> {
        let value = self.canonicalized();
        value.guard()?;
        if matches!(value.target_market, Some(TargetMarket::LegacyRegion { .. })) {
            return Err("unmapped regional market: an explicit code mapping is required".into());
        }
        Ok(value)
    }
    pub fn canonicalized(mut self) -> Self {
        self.target_market = self.target_market.map(|market| market.canonicalized());
        self
    }
    pub fn legacy(mut self) -> Self {
        self.target_market = self.target_market.map(|market| market.legacy());
        self
    }
    pub fn guard(&self) -> Result<(), String> {
        if let Some(TargetMarket::LegacyRegion { label }) = &self.target_market
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
    parse_language(value).map(|_| ())
}

fn parse_language(value: &str) -> Result<icu_locale_core::Locale, String> {
    if value.len() > 128 || value.contains('_') {
        return Err("unsupported or malformed language tag".into());
    }
    value
        .parse()
        .map_err(|_| "unsupported or malformed language tag".into())
}

/// Interpretation for a new extraction only. Historical observations retain their spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocale {
    pub canonical: String,
    pub language: String,
    pub region: Option<String>,
    pub country: Option<CountryCode>,
}

pub fn source_locale(value: &str) -> Result<SourceLocale, String> {
    let locale = parse_language(value)?;
    let region = locale.id.region.map(|region| region.to_string());
    let country = region.as_deref().and_then(|region| region.parse().ok());
    Ok(SourceLocale {
        canonical: locale.to_string(),
        language: locale.id.language.to_string(),
        region,
        country,
    })
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

#[cfg(test)]
mod source_locale_tests {
    use super::*;

    #[test]
    fn parses_full_locale_without_likely_subtags_or_extension_overrides() {
        for (raw, language, country) in [
            ("it-IT", "it", Some("IT")),
            ("IT-it", "it", Some("IT")),
            ("zh-Hant-TW", "zh", Some("TW")),
            ("sl-SI-rozaj", "sl", Some("SI")),
            ("en-UK", "en", Some("GB")),
            ("und-IT", "und", Some("IT")),
            ("it-IT-x-test", "it", Some("IT")),
            ("en-US-u-rg-itzzzz", "en", Some("US")),
            ("it", "it", None),
            ("zh-Hant", "zh", None),
            ("es-419", "es", None),
            ("en-001", "en", None),
            ("en-EU", "en", None),
            ("en-ZZ", "en", None),
            ("en-u-rg-itzzzz", "en", None),
        ] {
            let parsed = source_locale(raw).unwrap();
            assert_eq!(parsed.language, language, "{raw}");
            assert_eq!(
                parsed
                    .country
                    .map(|code| code.as_str().to_owned())
                    .as_deref(),
                country,
                "{raw}"
            );
        }
        assert_eq!(
            source_locale("IT-it").unwrap(),
            source_locale("it-IT").unwrap()
        );
        assert_ne!(
            source_locale("en-US").unwrap(),
            source_locale("en-GB").unwrap()
        );
        for raw in [
            "",
            " ",
            "it_IT",
            "it-IT\0",
            "en-\nUS",
            "i-klingon",
            "zh-cmn",
            &"a".repeat(129),
        ] {
            assert!(source_locale(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn deserialization_never_reinterprets_historical_observations() {
        let old = serde_json::json!({"target_market":null,"language":"IT-it"});
        let value: Localization = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), old);
    }
}

#[cfg(test)]
mod region_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn code_and_legacy_contracts_are_distinct_and_round_trip() {
        let old = json!({"kind":"region","label":"European Union"});
        let market: TargetMarket = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(serde_json::to_value(&market).unwrap(), old);
        let current = market.canonicalized();
        assert_eq!(
            serde_json::to_value(&current).unwrap(),
            json!({"kind":"region","code":"eu"})
        );
        assert_eq!(current.legacy(), market);
        let europe: TargetMarket =
            serde_json::from_value(json!({"kind":"region","label":"Europe"})).unwrap();
        assert_eq!(europe.canonicalized(), europe);
        let unsupported = Localization {
            target_market: Some(europe),
            language: None,
        };
        assert!(unsupported.guard().is_ok());
        assert!(unsupported.for_write().is_err());
        for value in [
            json!({"kind":"region","code":"EU"}),
            json!({"kind":"region","code":"unknown"}),
            json!({"kind":"region","code":""}),
            json!({"kind":"region","code":"eu","label":"European Union"}),
            json!({"kind":"country","code":"EU"}),
        ] {
            assert!(serde_json::from_value::<TargetMarket>(value).is_err());
        }
    }
}
