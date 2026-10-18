//! PostgreSQL representations and conversions, private to this slice.

use crate::platform::db::Json;
use crate::platform::object_storage::ObjectStorageMetadata;
use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::super::schema::{sitemaps_consolidated, sitemaps_processed};
use super::super::entity::{ConsolidatedSitemap, ProcessedSitemap};
use super::StoreError;

#[derive(Queryable, Selectable)]
#[diesel(table_name = sitemaps_processed)]
pub(super) struct ProcessedSitemapPgRecord {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: String,
    pub storage: Json<ObjectStorageMetadata>,
}

impl ProcessedSitemapPgRecord {
    pub fn into_entity(self, body: &str) -> Result<ProcessedSitemap, StoreError> {
        Ok(ProcessedSitemap {
            id: self.id,
            retrieval_id: self.retrieval_id,
            object_storage_id: self.storage.0.object_storage_id,
            retailer_code: RetailerCode::try_from(self.retailer_code.as_str())?,
            document: serde_json::from_str(body)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct References {
    pub processed_sitemap_id: Uuid,
    pub retrieval_id: Uuid,
    #[serde(with = "retailer_code")]
    pub retailer_code: RetailerCode,
}

// Preserve the existing lowercase database representation of retailer codes.
mod retailer_code {
    use bodul_shared::retailer::RetailerCode;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(code: &RetailerCode, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&code.as_str())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<RetailerCode, D::Error> {
        let code = String::deserialize(deserializer)?;
        RetailerCode::try_from(code.as_str()).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Stats {
    pub product_count: u32,
    pub catalog_count: u32,
    pub content_count: u32,
    pub not_interested_count: u32,
    pub unknown_count: u32,
}

#[derive(Insertable)]
#[diesel(table_name = sitemaps_consolidated)]
pub(super) struct ConsolidatedSitemapPgRecord {
    pub id: Uuid,
    pub references: Json<References>,
    pub stats: Json<Stats>,
    pub storage: Json<ObjectStorageMetadata>,
    pub consolidated_at: DateTime<Utc>,
}

pub(super) fn validate(entity: &ConsolidatedSitemap, body: &str) -> Result<(), StoreError> {
    i32::try_from(body.len()).map_err(|_| {
        StoreError::Unexpected("consolidated sitemap content is too large for storage".into())
    })?;
    stats(entity)?;
    Ok(())
}

fn stats(entity: &ConsolidatedSitemap) -> Result<Stats, StoreError> {
    Ok(Stats {
        product_count: stored_count(entity.product_count())?,
        catalog_count: stored_count(entity.catalog_count())?,
        content_count: stored_count(entity.content_count())?,
        not_interested_count: stored_count(entity.not_interested_count())?,
        unknown_count: stored_count(entity.unknown_count())?,
    })
}

impl ConsolidatedSitemapPgRecord {
    pub fn new(
        entity: &ConsolidatedSitemap,
        storage: ObjectStorageMetadata,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            id: entity.id,
            references: Json(References {
                processed_sitemap_id: entity.processed_sitemap_id,
                retrieval_id: entity.retrieval_id,
                retailer_code: entity.retailer_code,
            }),
            stats: Json(stats(entity)?),
            storage: Json(storage),
            consolidated_at: Utc::now(),
        })
    }
}

fn stored_count(count: usize) -> Result<u32, StoreError> {
    i32::try_from(count).map(|count| count as u32).map_err(|_| {
        StoreError::Unexpected(format!(
            "consolidated sitemap count is too large for storage: {count}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_counts_representable_in_the_previous_schema() {
        assert_eq!(stored_count(0).unwrap(), 0);
        assert_eq!(stored_count(i32::MAX as usize).unwrap(), i32::MAX as u32);
        assert!(stored_count(i32::MAX as usize + 1).is_err());
    }
    #[test]
    fn rejects_invalid_stored_documents_and_retailer_codes() {
        let record = |code: &str| ProcessedSitemapPgRecord {
            id: Uuid::now_v7(),
            retrieval_id: Uuid::now_v7(),
            retailer_code: code.into(),
            storage: Json(ObjectStorageMetadata {
                content_size: 4,
                object_storage_id: Uuid::now_v7(),
            }),
        };
        assert!(matches!(
            record("invalid").into_entity("null"),
            Err(StoreError::InvalidRetailerCode(_))
        ));
        assert!(matches!(
            record("ugreenca").into_entity("null"),
            Err(StoreError::Serialization(_))
        ));
    }
}

#[cfg(test)]
mod postgres_tests {
    use super::*;
    use crate::platform::test_db::Database;
    use diesel::sql_types::Jsonb;
    use serde_json::json;

    #[derive(Debug, QueryableByName)]
    struct Metadata {
        #[diesel(sql_type = Jsonb)]
        references: Json<References>,
        #[diesel(sql_type = Jsonb)]
        stats: Json<Stats>,
        #[diesel(sql_type = Jsonb)]
        storage: Json<ObjectStorageMetadata>,
    }

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn metadata_round_trips_and_rejects_invalid_fields() {
        let db = Database::new();
        let mut conn = db.pool.get().unwrap();
        let references = Json(References {
            processed_sitemap_id: Uuid::now_v7(),
            retrieval_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
        });
        let stats = Json(Stats {
            product_count: 1,
            catalog_count: 2,
            content_count: 3,
            not_interested_count: 4,
            unknown_count: 5,
        });
        let storage = Json(ObjectStorageMetadata {
            content_size: 123,
            object_storage_id: Uuid::now_v7(),
        });
        let query = r#"SELECT $1 AS "references", $2 AS stats, $3 AS storage"#;
        let output: Metadata = diesel::sql_query(query)
            .bind::<Jsonb, _>(&references)
            .bind::<Jsonb, _>(&stats)
            .bind::<Jsonb, _>(&storage)
            .get_result(&mut conn)
            .unwrap();
        assert_eq!(output.references, references);
        assert_eq!(output.stats, stats);
        assert_eq!(output.storage, storage);

        // Exercise the actual PostgreSQL deserializer, not only Serde in memory.
        let valid_references = serde_json::to_value(&references.0).unwrap();
        let valid_stats = serde_json::to_value(&stats.0).unwrap();
        for (field, invalid) in [
            ("processed_sitemap_id", json!("not-a-uuid")),
            ("retrieval_id", json!(42)),
            ("retailer_code", json!("invalid")),
            ("retailer_code", json!("UgreenCa")),
            ("extra", json!(true)),
        ] {
            let mut bad = valid_references.clone();
            bad[field] = invalid;
            assert!(matches!(
                diesel::sql_query(query)
                    .bind::<Jsonb, _>(bad)
                    .bind::<Jsonb, _>(&stats)
                    .bind::<Jsonb, _>(&storage)
                    .get_result::<Metadata>(&mut conn),
                Err(diesel::result::Error::DeserializationError(_))
            ));
        }
        for field in ["processed_sitemap_id", "retrieval_id", "retailer_code"] {
            let mut bad = valid_references.clone();
            bad.as_object_mut().unwrap().remove(field);
            assert!(
                diesel::sql_query(query)
                    .bind::<Jsonb, _>(bad)
                    .bind::<Jsonb, _>(&stats)
                    .bind::<Jsonb, _>(&storage)
                    .get_result::<Metadata>(&mut conn)
                    .is_err()
            );
        }
        for field in [
            "product_count",
            "catalog_count",
            "content_count",
            "not_interested_count",
            "unknown_count",
        ] {
            for invalid in [
                json!(null),
                json!(-1),
                json!(1.5),
                json!("2"),
                json!(4294967296_u64),
            ] {
                let mut bad = valid_stats.clone();
                bad[field] = invalid;
                assert!(matches!(
                    diesel::sql_query(query)
                        .bind::<Jsonb, _>(&references)
                        .bind::<Jsonb, _>(bad)
                        .bind::<Jsonb, _>(&storage)
                        .get_result::<Metadata>(&mut conn),
                    Err(diesel::result::Error::DeserializationError(_))
                ));
            }
            let mut bad = valid_stats.clone();
            bad.as_object_mut().unwrap().remove(field);
            assert!(
                diesel::sql_query(query)
                    .bind::<Jsonb, _>(&references)
                    .bind::<Jsonb, _>(bad)
                    .bind::<Jsonb, _>(&storage)
                    .get_result::<Metadata>(&mut conn)
                    .is_err()
            );
        }
    }
}
