//! Parses a retrieval's stored sitemap files into one sitemap tree and stores
//! it.

pub mod prelude;

mod command;
mod entity;
mod event;
mod handler;
#[cfg(test)]
mod postgres_tests;
mod reacts_to;
mod store;

#[cfg(test)]
mod live_tests {
    use super::postgres_tests::verify_round_trip;
    use crate::platform::object_storage::ObjectStorage;
    use crate::platform::test_db::Database;
    use tokio::runtime::Runtime;

    #[test]
    #[ignore = "requires DATABASE_URL_TEST, OBJECT_STORAGE_URL and OBJECT_STORAGE_TOKEN; run make test-object-storage"]
    fn live_round_trip() {
        let runtime = Runtime::new().unwrap();
        let objects = ObjectStorage::from_env(runtime.handle().clone()).unwrap();
        verify_round_trip(&Database::new(), objects);
    }
}
