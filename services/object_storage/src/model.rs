use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::{Uuid, Variant, Version};

pub const BUCKETS: &[&str] = &[
    "sitemap_raw",
    "sitemaps_raw",
    "sitemaps_processed",
    "sitemaps_consolidated",
    "sitemap_processed",
    "sitemap_consolidated",
    "offer_details_raw",
    "offer_details_extracted",
    "offer_details_normalized",
    "offer_details_structuralized",
    "offer_export",
];

pub fn bucket(value: &str) -> Result<(), Error> {
    if BUCKETS.contains(&value) {
        Ok(())
    } else {
        Err(Error::Input("Unknown bucket."))
    }
}

pub fn uuid7(value: &str) -> Result<Uuid, Error> {
    Uuid::parse_str(value)
        .ok()
        .filter(|id| {
            id.get_version() == Some(Version::SortRand) && id.get_variant() == Variant::RFC4122
        })
        .ok_or(Error::Input(
            "IDs and idempotency keys must be UUIDv7 strings.",
        ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub idempotency_key: String,
    pub payload: String,
    pub metadata: Value,
}

impl Create {
    pub fn validate(&self, name: &str) -> Result<Uuid, Error> {
        bucket(name)?;
        let key = uuid7(&self.idempotency_key)?;
        if self.payload.contains('\0') || self.metadata.is_null() || !nul_free_json(&self.metadata)
        {
            return Err(Error::Input(
                "Payload and non-null metadata must contain values supported by PostgreSQL.",
            ));
        }
        Ok(key)
    }
}

// Reject NUL characters before any database work. PostgreSQL performs numeric
// range validation itself, without converting numbers through floating point.
fn nul_free_json(value: &Value) -> bool {
    match value {
        Value::String(s) => !s.contains('\0'),
        Value::Array(values) => values.iter().all(nul_free_json),
        Value::Object(values) => values
            .iter()
            .all(|(k, v)| !k.contains('\0') && nul_free_json(v)),
        _ => true,
    }
}

#[derive(Debug, Serialize, diesel::QueryableByName)]
pub struct Record {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub bucket: String,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub idempotency_key: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub payload: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub metadata: Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct Created {
    pub id: Uuid,
    #[serde(skip)]
    pub fresh: bool,
}
