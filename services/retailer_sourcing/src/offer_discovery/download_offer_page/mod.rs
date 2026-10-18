//! Downloads an offer's page and stores it, every time it is asked to.

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
    #[test]
    #[ignore = "requires DATABASE_URL_TEST, OBJECT_STORAGE_URL and OBJECT_STORAGE_TOKEN; run make test-object-storage"]
    fn live_round_trip() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let objects =
            crate::platform::object_storage::ObjectStorage::from_env(runtime.handle().clone())
                .unwrap();
        super::postgres_tests::verify_round_trip(
            &crate::platform::test_db::Database::new(),
            objects,
        );
    }
}
