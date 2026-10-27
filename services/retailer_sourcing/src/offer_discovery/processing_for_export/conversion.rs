use std::collections::HashSet;

use money::Money;
use serde::Deserialize;
use uuid::Uuid;

use super::entity::{
    ExclusionReason, ExportEntry, ExportExclusion, ExportSource, OfferExport, OfferImport,
    SCHEMA_VERSION,
};

#[derive(Deserialize)]
struct StructuralizedDocument {
    variations: Vec<Variation>,
}

#[derive(Deserialize)]
struct Variation {
    title: String,
    url: String,
    price: Option<Money>,
}

#[derive(Debug, thiserror::Error)]
#[error("offer export conversion failed: {0}")]
pub struct ConversionError(String);

pub(super) fn convert(source: ExportSource, body: &str) -> Result<OfferExport, ConversionError> {
    let input: StructuralizedDocument =
        serde_json::from_str(body).map_err(|error| ConversionError(error.to_string()))?;
    let mut entries = Vec::new();
    let mut exclusions = Vec::new();
    let mut urls = HashSet::new();
    for (variation_index, variation) in input.variations.into_iter().enumerate() {
        variation.guard(variation_index)?;
        if !urls.insert(variation.url.clone()) {
            return Err(ConversionError(format!(
                "variation {variation_index} repeats a retailer/URL identity"
            )));
        }
        let Some(price) = variation.price else {
            exclusions.push(ExportExclusion {
                variation_index,
                url: variation.url,
                reason: ExclusionReason::MissingPrice,
            });
            continue;
        };
        entries.push(ExportEntry {
            id: Uuid::now_v7(),
            variation_index,
            payload: OfferImport {
                retailer_code: source.retailer_code,
                title: variation.title,
                price,
                url: variation.url,
            },
        });
    }
    Ok(OfferExport {
        id: Uuid::now_v7(),
        schema_version: SCHEMA_VERSION,
        source,
        entries,
        exclusions,
    })
}

impl Variation {
    fn guard(&self, index: usize) -> Result<(), ConversionError> {
        if self.title.trim().is_empty() || self.title.contains('\0') {
            return Err(ConversionError(format!(
                "variation {index} requires a nonblank title without NUL"
            )));
        }
        if self.url.len() > 2048
            || self.url.trim() != self.url
            || self.url.chars().any(char::is_control)
            || !url::Url::parse(&self.url)
                .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.has_host())
        {
            return Err(ConversionError(format!(
                "variation {index} requires an absolute HTTP(S) URL of at most 2048 bytes without padding or controls"
            )));
        }
        if self.price.as_ref().is_some_and(|price| price.amount() < 0) {
            return Err(ConversionError(format!(
                "variation {index} requires a nonnegative price"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::platform::object_storage::OfferStorageMetadata;
    use bodul_shared::retailer::RetailerCode;
    use serde_json::{Value, json};

    pub(crate) const ANKER: &str =
        include_str!("../../../tests/fixtures/offer_export/ankerau.json");
    const UGREEN: &str = include_str!("../../../tests/fixtures/offer_export/ugreenca.json");
    const MINISFORUM: &str = include_str!("../../../tests/fixtures/offer_export/minisforumus.json");

    pub(crate) fn source() -> ExportSource {
        ExportSource {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://shop.example/Charger".into(),
            storage: OfferStorageMetadata {
                object_storage_id: Uuid::now_v7(),
                external_id: Uuid::now_v7(),
                content_size: ANKER.len() as u64,
            },
        }
    }

    #[test]
    fn preserves_priced_variations_and_matches_the_offers_contract_fixture() {
        let output = convert(source(), ANKER).unwrap();
        let expected: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&output.entries[0].payload).unwrap(),
            expected
        );
        assert_eq!(output.entries.len(), 2);
        assert_eq!(output.entries[1].payload.price.amount(), 0);
        assert_eq!(output.entries[1].variation_index, 1);
        assert_ne!(output.entries[0].id, output.entries[1].id);
        assert!(
            output
                .entries
                .iter()
                .all(|entry| entry.id.get_version_num() == 7)
        );
        assert!(output.exclusions.is_empty());
    }

    #[test]
    fn records_missing_price_and_keeps_unavailable_priced_variations() {
        let mut source = source();
        source.retailer_code = RetailerCode::UgreenCa;
        let output = convert(source, UGREEN).unwrap();
        assert_eq!(output.entries.len(), 1);
        assert_eq!(
            output.entries[0].payload.retailer_code,
            RetailerCode::UgreenCa
        );
        assert_eq!(
            output.exclusions,
            vec![ExportExclusion {
                variation_index: 1,
                url: "https://ca.example/products/hub?variant=2".into(),
                reason: ExclusionReason::MissingPrice,
            }]
        );
        let mut source = output.source;
        source.retailer_code = RetailerCode::MinisForumUs;
        let output = convert(source, MINISFORUM).unwrap();
        assert_eq!(output.entries[0].payload.price.amount(), 59900);
        assert_eq!(
            output.entries[0].payload.retailer_code,
            RetailerCode::MinisForumUs
        );
    }

    #[test]
    fn supports_empty_exports_and_missing_price_fields() {
        let output = convert(source(), r#"{"variations":[]}"#).unwrap();
        assert!(output.entries.is_empty());
        assert!(output.exclusions.is_empty());
        let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
        input["variations"][0]
            .as_object_mut()
            .unwrap()
            .remove("price");
        let output = convert(source(), &input.to_string()).unwrap();
        assert!(output.entries.is_empty());
        assert_eq!(output.exclusions.len(), 1);
    }

    #[test]
    fn preserves_exact_integer_money_including_zero_decimal_currencies() {
        for currency in ["AUD", "JPY", "KRW", "VND"] {
            for amount in [0, 1, i64::MAX] {
                let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
                input["variations"][0]["price"] = json!({"amount":amount,"currency":currency});
                let output = convert(source(), &input.to_string()).unwrap();
                assert_eq!(
                    serde_json::to_value(&output.entries[0].payload.price).unwrap(),
                    json!({"amount":amount,"currency":currency})
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_input_before_any_preparation() {
        for input in ["{}", "null", "[]", r#"{"variations":null}"#, "not JSON"] {
            assert!(convert(source(), input).is_err(), "{input}");
        }
        for (field, value) in [
            ("title", json!("  ")),
            ("title", json!("nul\u{0}")),
            ("title", json!(7)),
            ("url", json!("/relative")),
            ("url", json!("ftp://shop.example/product")),
            ("url", json!(" https://shop.example/product")),
            ("url", json!("https://shop.example/\nproduct")),
            (
                "url",
                json!(format!("https://shop.example/{}", "a".repeat(2048))),
            ),
            ("price", json!({"amount":-1,"currency":"AUD"})),
            ("price", json!({"amount":1.5,"currency":"AUD"})),
            ("price", json!({"amount":"1","currency":"AUD"})),
            ("price", json!({"amount":1,"currency":"BAD"})),
        ] {
            let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
            input["variations"][0][field] = value;
            assert!(convert(source(), &input.to_string()).is_err(), "{input}");
        }
        let mut input: Value = serde_json::from_str(ANKER).unwrap();
        input["variations"][1]["url"] = input["variations"][0]["url"].clone();
        assert!(
            convert(source(), &input.to_string())
                .unwrap_err()
                .to_string()
                .contains("repeats")
        );
    }
}
