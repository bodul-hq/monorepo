use std::collections::HashSet;

use money::Money;
use serde::Deserialize;
use uuid::Uuid;

use super::entity::{
    ExclusionReason, ExportEntry, ExportExclusion, ExportSource, OfferExport, OfferImport,
};

#[derive(Deserialize)]
struct StructuralizedDocument {
    variations: Vec<Variation>,
}

#[derive(Deserialize)]
struct Variation {
    #[serde(default)]
    identifiers: VariationIdentifiers,
    #[serde(default)]
    localization: retailer_sauce::localization::Localization,
    title: String,
    url: String,
    price: Option<Money>,
    #[serde(default = "empty_images")]
    images: serde_json::Value,
}

#[derive(Default, Deserialize)]
struct VariationIdentifiers {
    gtin: Option<retailer_sauce::identifier::Gtin>,
    sku: Option<retailer_sauce::identifier::Sku>,
    #[serde(flatten)]
    external: retailer_sauce::identifier::ExternalIdentifiers,
}
impl VariationIdentifiers {
    fn into_identifiers(self) -> retailer_sauce::identifier::ProductIdentifiers {
        retailer_sauce::identifier::ProductIdentifiers {
            gtin: self.gtin,
            sku: self.sku,
            external_identifiers: self.external,
        }
    }
}

fn empty_images() -> serde_json::Value {
    serde_json::json!([])
}

#[derive(Debug, thiserror::Error)]
#[error("offer export conversion failed: {0}")]
pub struct ConversionError(String);

pub(super) fn convert(
    source: ExportSource,
    body: &str,
    schema_version: u32,
) -> Result<OfferExport, ConversionError> {
    if !matches!(schema_version, 1..=5) {
        return Err(ConversionError("unsupported schema version".into()));
    }
    if schema_version < 3 && source.storage.scraped_at.is_some() {
        return Err(ConversionError(
            "legacy export source cannot carry scrape time".into(),
        ));
    }
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
        let images = if schema_version >= 2 {
            let (images, skipped) = shared::images::filter(
                serde_json::from_value(variation.images)
                    .map_err(|error| ConversionError(error.to_string()))?,
            );
            tracing::info!(skipped_images = skipped, "filtered offer images");
            Some(images)
        } else {
            None
        };
        let identifiers = variation.identifiers.into_identifiers();
        identifiers.guard().map_err(ConversionError)?;
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
                gtin: if schema_version >= 5 {
                    identifiers.gtin
                } else {
                    None
                },
                sku: if schema_version >= 5 {
                    identifiers.sku
                } else {
                    None
                },
                external_identifiers: (schema_version >= 5)
                    .then_some(identifiers.external_identifiers),
                localization: (schema_version >= 4).then_some(variation.localization),
                scraped_at: if schema_version >= 3 {
                    source.storage.scraped_at
                } else {
                    None
                },
                images,
                retailer_code: source.retailer_code,
                title: variation.title,
                price,
                url: variation.url,
            },
        });
    }
    Ok(OfferExport {
        id: Uuid::now_v7(),
        schema_version,
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
    pub(crate) const UGREEN: &str =
        include_str!("../../../tests/fixtures/offer_export/ugreenca.json");
    pub(crate) const MINISFORUM: &str =
        include_str!("../../../tests/fixtures/offer_export/minisforumus.json");

    pub(crate) fn source() -> ExportSource {
        ExportSource {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://shop.example/Charger".into(),
            storage: OfferStorageMetadata {
                scraped_at: None,
                object_storage_id: Uuid::now_v7(),
                external_id: Uuid::now_v7(),
                content_size: ANKER.len() as u64,
            },
        }
    }

    #[test]
    fn preserves_priced_variations_and_matches_the_offers_contract_fixture() {
        let output = convert(source(), ANKER, 1).unwrap();
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
        let output = convert(source, UGREEN, 1).unwrap();
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
        let output = convert(source, MINISFORUM, 1).unwrap();
        assert_eq!(output.entries[0].payload.price.amount(), 59900);
        assert_eq!(
            output.entries[0].payload.retailer_code,
            RetailerCode::MinisForumUs
        );
    }

    #[test]
    fn supports_empty_exports_and_missing_price_fields() {
        let output = convert(source(), r#"{"variations":[]}"#, 1).unwrap();
        assert!(output.entries.is_empty());
        assert!(output.exclusions.is_empty());
        let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
        input["variations"][0]
            .as_object_mut()
            .unwrap()
            .remove("price");
        let output = convert(source(), &input.to_string(), 1).unwrap();
        assert!(output.entries.is_empty());
        assert_eq!(output.exclusions.len(), 1);
    }

    #[test]
    fn preserves_exact_integer_money_including_zero_decimal_currencies() {
        for currency in ["AUD", "JPY", "KRW", "VND"] {
            for amount in [0, 1, i64::MAX] {
                let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
                input["variations"][0]["price"] = json!({"amount":amount,"currency":currency});
                let output = convert(source(), &input.to_string(), 1).unwrap();
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
            assert!(convert(source(), input, 1).is_err(), "{input}");
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
            assert!(convert(source(), &input.to_string(), 1).is_err(), "{input}");
        }
        let mut input: Value = serde_json::from_str(ANKER).unwrap();
        input["variations"][1]["url"] = input["variations"][0]["url"].clone();
        assert!(
            convert(source(), &input.to_string(), 1)
                .unwrap_err()
                .to_string()
                .contains("repeats")
        );
    }

    #[test]
    fn v2_images_are_ordered_filtered_and_v1_shape_is_immutable() {
        let mut input: Value = serde_json::from_str(ANKER).unwrap();
        let a = "https://shop.example/A?q=1&x=2#front";
        let b = "http://shop.example/B";
        input["variations"][0]["images"] =
            json!([a, a, "/relative", b, "https://user@shop.example/a"]);
        let legacy = convert(source(), &input.to_string(), 1).unwrap();
        assert!(
            serde_json::to_value(&legacy.entries[0].payload)
                .unwrap()
                .get("images")
                .is_none()
        );
        let current = convert(source(), &input.to_string(), 2).unwrap();
        assert_eq!(current.entries[0].payload.images.as_ref().unwrap(), &[a, b]);
        assert_eq!(current.entries[1].payload.images, Some(vec![]));
        assert!(current.valid_schema());
        for images in [json!([]), json!(["/invalid", "javascript:alert(1)"])] {
            input["variations"][0]["images"] = images;
            assert_eq!(
                convert(source(), &input.to_string(), 2).unwrap().entries[0]
                    .payload
                    .images,
                Some(vec![])
            );
        }
        for images in [
            json!(null),
            json!("https://shop.example/A"),
            json!([1]),
            json!({}),
        ] {
            input["variations"][0]["images"] = images;
            assert!(convert(source(), &input.to_string(), 2).is_err());
            assert!(convert(source(), &input.to_string(), 1).is_ok());
        }
        input["variations"][0]
            .as_object_mut()
            .unwrap()
            .remove("images");
        assert_eq!(
            convert(source(), &input.to_string(), 2).unwrap().entries[0]
                .payload
                .images,
            Some(vec![])
        );
        assert!(convert(source(), ANKER, 99).is_err());
    }

    #[test]
    fn v2_matches_shared_import_fixture() {
        let output = convert(source(), ANKER, 2).unwrap();
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_payload_v2.json"
        ))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&output.entries[0].payload).unwrap(),
            fixture
        );
    }
}

#[cfg(test)]
mod scrape_tests {
    use super::*;
    #[test]
    fn v3_preserves_time_while_legacy_versions_preserve_their_shape() {
        let mut source = tests::source();
        for version in [1, 2] {
            let snapshot = convert(source.clone(), tests::ANKER, version).unwrap();
            let json = serde_json::to_value(&snapshot).unwrap();
            assert!(json["source"]["storage"].get("scraped_at").is_none());
            for entry in json["entries"].as_array().unwrap() {
                assert!(entry["payload"].get("scraped_at").is_none());
            }
            assert!(snapshot.valid_schema());
        }
        source.storage.scraped_at =
            Some(chrono::DateTime::from_timestamp_micros(1_700_000_000_123_456).unwrap());
        let snapshot = convert(source.clone(), tests::ANKER, 3).unwrap();
        assert!(snapshot.valid_schema());
        for entry in &snapshot.entries {
            assert_eq!(entry.payload.scraped_at, source.storage.scraped_at);
        }
        assert!(convert(source, tests::ANKER, 2).is_err());
        let mut invalid = snapshot.clone();
        invalid.entries[0].payload.scraped_at = None;
        assert!(!invalid.valid_schema());
    }
}

#[cfg(test)]
mod extended_contract_tests {
    use super::*;
    #[test]
    fn v5_keeps_exact_identifiers_and_legacy_exports_omit_new_fields() {
        let body = serde_json::json!({"variations":[{"title":" USB charger ž ","url":"https://shop.example/Charger?q=red&variant=1#details","price":{"amount":1999,"currency":"AUD"},"images":["https://example.com/a.png"],"localization":{"target_market":{"kind":"country","code":"AU"},"language":"en-AU"},"identifiers":{"gtin":"0012345678905","sku":"A1-BLACK","shopify_product_gid":"gid://shopify/Product/1","shopify_product_variation_gid":"gid://shopify/ProductVariant/2","shopify_handle":"usb-charger"}}]});
        let mut source = tests::source();
        source.storage.scraped_at = Some("2026-10-26T12:00:00.123456Z".parse().unwrap());
        let snapshot = convert(source.clone(), &body.to_string(), 5).unwrap();
        assert!(snapshot.valid_schema());
        assert_eq!(
            serde_json::to_value(&snapshot.entries[0].payload).unwrap(),
            serde_json::from_str::<serde_json::Value>(include_str!(
                "../../../tests/fixtures/offer_export/import_payload_v5.json"
            ))
            .unwrap()
        );
        for version in [1, 2, 3, 4] {
            let mut source = source.clone();
            if version < 3 {
                source.storage.scraped_at = None;
            }
            let snapshot = convert(source, &body.to_string(), version).unwrap();
            assert!(snapshot.valid_schema());
            let json = serde_json::to_value(&snapshot.entries[0].payload).unwrap();
            for field in ["gtin", "sku", "external_identifiers"] {
                assert!(json.get(field).is_none());
            }
        }
        for value in [
            serde_json::json!(null),
            serde_json::json!({"gtin":""}),
            serde_json::json!({"sku":123}),
            serde_json::json!({"shopify_handle":"\0"}),
        ] {
            let mut invalid = body.clone();
            invalid["variations"][0]["identifiers"] = value;
            assert!(convert(source.clone(), &invalid.to_string(), 5).is_err());
        }
        for fixture in [tests::ANKER, tests::UGREEN, tests::MINISFORUM] {
            let snapshot = convert(source.clone(), fixture, 5).unwrap();
            assert!(snapshot.valid_schema());
            assert!(
                snapshot
                    .entries
                    .iter()
                    .all(|entry| entry.payload.external_identifiers.is_some())
            );
        }
    }
}
