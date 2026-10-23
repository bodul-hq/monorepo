//! Event metadata for storing additional context with events.

use std::collections::HashMap;

/// Metadata stores key-value pairs of additional context for events.
///
/// This is commonly used to store information like:
/// - Correlation IDs for tracing
/// - Causation IDs linking events
/// - User IDs who triggered the event
/// - Timestamps
/// - Any other contextual information
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    data: HashMap<String, String>,
}

impl Metadata {
    /// Creates a new empty Metadata instance.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Creates Metadata from an existing HashMap.
    pub fn from_map(data: HashMap<String, String>) -> Self {
        Self { data }
    }

    /// Inserts a key-value pair into the metadata.
    ///
    /// If the key already exists, the value is updated and the old value is returned.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.data.insert(key.into(), value.into())
    }

    /// Gets a value by key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Removes a key-value pair and returns the value if it existed.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    /// Returns true if the metadata contains the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns the number of key-value pairs.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the metadata is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns an iterator over the key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.data.iter()
    }

    /// Consumes the Metadata and returns the underlying HashMap.
    pub fn into_map(self) -> HashMap<String, String> {
        self.data
    }

    /// Returns a reference to the underlying HashMap.
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.data
    }
}

impl From<HashMap<String, String>> for Metadata {
    fn from(data: HashMap<String, String>) -> Self {
        Self::from_map(data)
    }
}

impl From<Metadata> for HashMap<String, String> {
    fn from(metadata: Metadata) -> Self {
        metadata.into_map()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_metadata() {
        let metadata = Metadata::new();
        assert!(metadata.is_empty());
        assert_eq!(metadata.len(), 0);
    }

    #[test]
    fn insert_adds_key_value() {
        let mut metadata = Metadata::new();
        metadata.insert("correlation_id", "abc-123");

        assert_eq!(metadata.get("correlation_id"), Some(&"abc-123".to_string()));
        assert_eq!(metadata.len(), 1);
    }

    #[test]
    fn insert_updates_existing_key() {
        let mut metadata = Metadata::new();
        metadata.insert("key", "value1");
        let old = metadata.insert("key", "value2");

        assert_eq!(old, Some("value1".to_string()));
        assert_eq!(metadata.get("key"), Some(&"value2".to_string()));
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let metadata = Metadata::new();
        assert_eq!(metadata.get("missing"), None);
    }

    #[test]
    fn remove_returns_value_and_removes_key() {
        let mut metadata = Metadata::new();
        metadata.insert("key", "value");

        let removed = metadata.remove("key");
        assert_eq!(removed, Some("value".to_string()));
        assert!(!metadata.contains_key("key"));
    }

    #[test]
    fn contains_key_returns_correct_result() {
        let mut metadata = Metadata::new();
        metadata.insert("exists", "value");

        assert!(metadata.contains_key("exists"));
        assert!(!metadata.contains_key("missing"));
    }

    #[test]
    fn from_map_creates_metadata() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());

        let metadata = Metadata::from_map(map);

        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn into_map_consumes_metadata() {
        let mut metadata = Metadata::new();
        metadata.insert("key", "value");

        let map = metadata.into_map();

        assert_eq!(map.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn as_map_returns_reference() {
        let mut metadata = Metadata::new();
        metadata.insert("key", "value");

        let map = metadata.as_map();

        assert_eq!(map.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn from_hashmap_trait() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());

        let metadata: Metadata = map.into();

        assert_eq!(metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn into_hashmap_trait() {
        let mut metadata = Metadata::new();
        metadata.insert("key", "value");

        let map: HashMap<String, String> = metadata.into();

        assert_eq!(map.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn default_creates_empty_metadata() {
        let metadata = Metadata::default();
        assert!(metadata.is_empty());
    }

    #[test]
    fn iter_returns_all_pairs() {
        let mut metadata = Metadata::new();
        metadata.insert("key1", "value1");
        metadata.insert("key2", "value2");

        let pairs: Vec<_> = metadata.iter().collect();

        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn equality_works() {
        let mut m1 = Metadata::new();
        m1.insert("key", "value");

        let mut m2 = Metadata::new();
        m2.insert("key", "value");

        assert_eq!(m1, m2);
    }

    #[test]
    fn clone_works() {
        let mut original = Metadata::new();
        original.insert("key", "value");

        let cloned = original.clone();

        assert_eq!(original, cloned);
    }
}
