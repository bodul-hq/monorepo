//! Database record representation of Metadata for Diesel PostgreSQL storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use testimony::prelude::Metadata;

/// Database record for storing Metadata as JSON.
///
/// This struct is designed to be stored in PostgreSQL using Diesel,
/// with the data serialized as a JSON string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataRecord {
    /// JSON-serialized key-value pairs
    pub data_json: String,
}

impl MetadataRecord {
    /// Creates a new MetadataRecord with the given JSON data.
    pub fn new(data_json: String) -> Self {
        Self { data_json }
    }

    /// Creates an empty MetadataRecord.
    pub fn empty() -> Self {
        Self {
            data_json: "{}".to_string(),
        }
    }

    /// Returns the JSON data as a string reference.
    pub fn as_json(&self) -> &str {
        &self.data_json
    }
}

impl From<Metadata> for MetadataRecord {
    fn from(metadata: Metadata) -> Self {
        let data_json =
            serde_json::to_string(metadata.as_map()).unwrap_or_else(|_| "{}".to_string());
        Self { data_json }
    }
}

impl From<MetadataRecord> for Metadata {
    fn from(record: MetadataRecord) -> Self {
        let map: HashMap<String, String> =
            serde_json::from_str(&record.data_json).unwrap_or_default();
        Metadata::from_map(map)
    }
}

impl From<&Metadata> for MetadataRecord {
    fn from(metadata: &Metadata) -> Self {
        let data_json =
            serde_json::to_string(metadata.as_map()).unwrap_or_else(|_| "{}".to_string());
        Self { data_json }
    }
}

impl Default for MetadataRecord {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_record_with_json() {
        let record = MetadataRecord::new(r#"{"key":"value"}"#.to_string());
        assert_eq!(record.as_json(), r#"{"key":"value"}"#);
    }

    #[test]
    fn empty_creates_empty_json_object() {
        let record = MetadataRecord::empty();
        assert_eq!(record.as_json(), "{}");
    }

    #[test]
    fn from_metadata_serializes_to_json() {
        let mut metadata = Metadata::new();
        metadata.insert("correlation_id", "abc-123");
        metadata.insert("user_id", "user-456");

        let record: MetadataRecord = metadata.into();

        // Parse the JSON to verify it contains the expected keys
        let parsed: HashMap<String, String> = serde_json::from_str(record.as_json()).unwrap();
        assert_eq!(parsed.get("correlation_id"), Some(&"abc-123".to_string()));
        assert_eq!(parsed.get("user_id"), Some(&"user-456".to_string()));
    }

    #[test]
    fn from_metadata_ref_serializes_to_json() {
        let mut metadata = Metadata::new();
        metadata.insert("key", "value");

        let record: MetadataRecord = (&metadata).into();

        let parsed: HashMap<String, String> = serde_json::from_str(record.as_json()).unwrap();
        assert_eq!(parsed.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn into_metadata_deserializes_from_json() {
        let record = MetadataRecord::new(r#"{"key1":"value1","key2":"value2"}"#.to_string());

        let metadata: Metadata = record.into();

        assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn roundtrip_preserves_data() {
        let mut original = Metadata::new();
        original.insert("correlation_id", "corr-123");
        original.insert("causation_id", "cause-456");
        original.insert("user_id", "user-789");

        let record: MetadataRecord = original.clone().into();
        let restored: Metadata = record.into();

        assert_eq!(
            original.get("correlation_id"),
            restored.get("correlation_id")
        );
        assert_eq!(original.get("causation_id"), restored.get("causation_id"));
        assert_eq!(original.get("user_id"), restored.get("user_id"));
    }

    #[test]
    fn empty_metadata_roundtrip() {
        let original = Metadata::new();

        let record: MetadataRecord = original.clone().into();
        let restored: Metadata = record.into();

        assert!(restored.is_empty());
    }

    #[test]
    fn invalid_json_returns_empty_metadata() {
        let record = MetadataRecord::new("invalid json".to_string());

        let metadata: Metadata = record.into();

        assert!(metadata.is_empty());
    }

    #[test]
    fn default_creates_empty_record() {
        let record = MetadataRecord::default();
        assert_eq!(record.as_json(), "{}");
    }

    #[test]
    fn serde_serialization_works() {
        let record = MetadataRecord::new(r#"{"key":"value"}"#.to_string());

        let serialized = serde_json::to_string(&record).unwrap();
        let deserialized: MetadataRecord = serde_json::from_str(&serialized).unwrap();

        assert_eq!(record, deserialized);
    }

    #[test]
    fn clone_works() {
        let record = MetadataRecord::new(r#"{"key":"value"}"#.to_string());
        let cloned = record.clone();

        assert_eq!(record, cloned);
    }
}
