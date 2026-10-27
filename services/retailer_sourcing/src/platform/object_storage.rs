//! Object storage client for blocking workers and command-line exports.
use object_storage_client::{Client, CreateRecord, Transport, Uuid};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::runtime::Handle;

pub const RAW_SITEMAPS_BUCKET: &str = "sitemaps_raw";
pub const PROCESSED_SITEMAPS_BUCKET: &str = "sitemaps_processed";
pub const CONSOLIDATED_SITEMAPS_BUCKET: &str = "sitemaps_consolidated";
pub const RAW_OFFERS_BUCKET: &str = "offer_details_raw";
pub const EXTRACTED_OFFERS_BUCKET: &str = "offer_details_extracted";
pub const NORMALIZED_OFFERS_BUCKET: &str = "offer_details_normalized";
pub const STRUCTURALIZED_OFFERS_BUCKET: &str = "offer_details_structuralized";
pub const OFFER_EXPORT_BUCKET: &str = "offer_export";

/// An entity whose object upload and local write have both succeeded.
#[derive(Debug, Clone)]
pub struct Stored<T> {
    pub entity: T,
    pub object_storage_id: Uuid,
}

/// Offers keep a stable row identity while each replacement uploads a new object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OfferStorageMetadata {
    pub content_size: u64,
    #[schema(value_type = String, format = "uuid")]
    pub object_storage_id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub external_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectStorageMetadata {
    pub content_size: u64,
    pub object_storage_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Configuration(&'static str),
    #[error("object storage: {0}")]
    Client(#[from] object_storage_client::Error),
    #[error("object storage record does not match its local reference")]
    InvalidRecord,
}

#[derive(Clone)]
pub struct ObjectStorage {
    client: Client,
    runtime: Handle,
}

impl ObjectStorage {
    pub fn from_env(runtime: Handle) -> Result<Self, Error> {
        let url = std::env::var("OBJECT_STORAGE_URL")
            .map_err(|_| Error::Configuration("OBJECT_STORAGE_URL is required"))?;
        let token = std::env::var("OBJECT_STORAGE_TOKEN")
            .map_err(|_| Error::Configuration("OBJECT_STORAGE_TOKEN is required"))?;
        Self::new(&url, &token, runtime)
    }

    pub fn new(url: &str, token: &str, runtime: Handle) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(object_storage_client::Error::from)?;
        let client = Client::with_http_client(http, url, token, Transport::Rest)?;
        Ok(Self { client, runtime })
    }

    /// Call only from a blocking worker, never an async runtime task.
    pub fn create(&self, bucket: &str, external_id: Uuid, body: &str) -> Result<Uuid, Error> {
        let input = CreateRecord {
            idempotency_key: external_id,
            payload: body.to_owned(),
            metadata: serde_json::Value::Object(Default::default()),
        };
        let id = self.runtime.block_on(self.client.create(bucket, &input))?;
        if id.get_version_num() != 7 || id.get_variant() != uuid::Variant::RFC4122 {
            return Err(Error::InvalidRecord);
        }
        Ok(id)
    }

    pub fn create_offer(&self, bucket: &str, body: &str) -> Result<OfferStorageMetadata, Error> {
        let external_id = Uuid::now_v7();
        let object_storage_id = self.create(bucket, external_id, body)?;
        Ok(OfferStorageMetadata {
            content_size: body.len() as u64,
            object_storage_id,
            external_id,
        })
    }

    pub fn get_offer(&self, bucket: &str, storage: &OfferStorageMetadata) -> Result<String, Error> {
        self.get(
            bucket,
            storage.external_id,
            &ObjectStorageMetadata {
                content_size: storage.content_size,
                object_storage_id: storage.object_storage_id,
            },
        )
    }

    /// Load and validate a payload without holding a database connection.
    pub fn get(
        &self,
        bucket: &str,
        external_id: Uuid,
        storage: &ObjectStorageMetadata,
    ) -> Result<String, Error> {
        let record = self
            .runtime
            .block_on(self.client.get(bucket, storage.object_storage_id))?;
        if record.id != storage.object_storage_id
            || record.bucket != bucket
            || record.idempotency_key != external_id
            || record.payload.len() as u64 != storage.content_size
        {
            return Err(Error::InvalidRecord);
        }
        Ok(record.payload)
    }
}
