use bodul_shared::retailer::RetailerCode;
use money::Money;
use retailer_sauce::observation::Observation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::object_storage::{ObjectStorageMetadata, OfferStorageMetadata};

pub const SCHEMA_VERSION: u32 = 9;

pub fn legacy_schema_version() -> u32 {
    1
}

/// The complete immutable input, independent of the latest pipeline metadata row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSource {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub storage: OfferStorageMetadata,
}

/// Matches offers' ImportOfferDetails command; this is not its HTTP form DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferImport {
    #[serde(
        default,
        skip_serializing_if = "Observation::is_missing",
        deserialize_with = "retailer_sauce::observation::object_observation"
    )]
    pub was_price: Observation<Money>,
    #[serde(default, skip_serializing_if = "Observation::is_missing")]
    pub availability: Observation<retailer_sauce::availability::Availability>,
    #[serde(default, skip_serializing_if = "Observation::is_missing")]
    pub item_condition: Observation<retailer_sauce::item_condition::ItemCondition>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::structuralized::content::observation"
    )]
    pub content: Option<retailer_sauce::structuralized::content::Content>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gtin: Option<retailer_sauce::identifier::Gtin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sku: Option<retailer_sauce::identifier::Sku>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::identifier::external_object"
    )]
    pub external_identifiers: Option<retailer_sauce::identifier::ExternalIdentifiers>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::localization::observation"
    )]
    pub localization: Option<retailer_sauce::localization::Localization>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "image_array"
    )]
    pub images: Option<Vec<String>>,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
}

fn image_array<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    Vec::<String>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportEntry {
    /// Delivery identity, not the receiving service's offer identity.
    pub id: Uuid,
    pub variation_index: usize,
    pub payload: OfferImport,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    MissingPrice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportExclusion {
    pub variation_index: usize,
    pub url: String,
    pub reason: ExclusionReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferExport {
    pub id: Uuid,
    pub schema_version: u32,
    pub source: ExportSource,
    pub entries: Vec<ExportEntry>,
    pub exclusions: Vec<ExportExclusion>,
}

impl OfferExport {
    pub fn valid_schema(&self) -> bool {
        matches!(self.schema_version, 1..=9)
            && (self.schema_version >= 3 || self.source.storage.scraped_at.is_none())
            && self.entries.iter().all(|entry| {
                if self.schema_version >= 9
                    && entry
                        .payload
                        .localization
                        .as_ref()
                        .and_then(|value| value.target_market.as_ref())
                        .is_some_and(|market| {
                            matches!(
                                market,
                                retailer_sauce::localization::TargetMarket::LegacyRegion { .. }
                            )
                        })
                {
                    return false;
                }
                let payload = &entry.payload;
                if (self.schema_version < 6 && !payload.was_price.is_missing())
                    || (self.schema_version < 7
                        && (!payload.availability.is_missing()
                            || !payload.item_condition.is_missing()))
                    || (self.schema_version < 8 && payload.content.is_some())
                    || payload.was_price.value().is_some_and(|was| {
                        was.currency() != payload.price.currency()
                            || was.amount() <= payload.price.amount()
                    })
                    || payload
                        .availability
                        .value()
                        .is_some_and(|v| v.guard().is_err())
                    || payload
                        .item_condition
                        .value()
                        .is_some_and(|v| v.guard().is_err())
                    || payload.content.as_ref().is_some_and(|v| v.guard().is_err())
                {
                    return false;
                }

                if entry.payload.external_identifiers.is_some() != (self.schema_version >= 5)
                    || (self.schema_version < 5
                        && (entry.payload.gtin.is_some() || entry.payload.sku.is_some()))
                {
                    return false;
                }
                let identifiers = retailer_sauce::identifier::ProductIdentifiers {
                    gtin: entry.payload.gtin.clone(),
                    sku: entry.payload.sku.clone(),
                    external_identifiers: entry
                        .payload
                        .external_identifiers
                        .clone()
                        .unwrap_or_default(),
                };
                if identifiers.guard().is_err() {
                    return false;
                }
                if entry
                    .payload
                    .localization
                    .as_ref()
                    .is_some_and(|value| value.guard().is_err())
                {
                    return false;
                }
                if entry.payload.localization.is_some() != (self.schema_version >= 4) {
                    return false;
                }
                if entry.payload.scraped_at
                    != (if self.schema_version >= 3 {
                        self.source.storage.scraped_at
                    } else {
                        None
                    })
                {
                    return false;
                }
                if self.schema_version == 1 {
                    entry.payload.images.is_none()
                } else {
                    entry.payload.images.as_ref().is_some_and(|images| {
                        let (filtered, skipped) = shared::images::filter(images.clone());
                        skipped == 0 && &filtered == images
                    })
                }
            })
    }
}

/// Pending bytes survive crashes; completed snapshots are loaded from object storage.
#[derive(Debug, Clone)]
pub(super) struct PreparedExport {
    pub snapshot: OfferExport,
    pub body: String,
    pub storage: Option<ObjectStorageMetadata>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn payload_images_distinguish_omission_from_malformed_null() {
        let mut value: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        assert!(
            serde_json::from_value::<OfferImport>(value.clone())
                .unwrap()
                .images
                .is_none()
        );
        value["images"] = serde_json::json!([]);
        assert_eq!(
            serde_json::from_value::<OfferImport>(value.clone())
                .unwrap()
                .images,
            Some(vec![])
        );
        value["images"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<OfferImport>(value).is_err());
    }
}
