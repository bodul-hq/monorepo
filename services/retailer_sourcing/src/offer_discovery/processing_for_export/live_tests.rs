use super::postgres_tests::verify_round_trip;
use crate::platform::object_storage::ObjectStorage;
use crate::platform::test_db::Database;

#[test]
#[ignore = "requires DATABASE_URL_TEST, OBJECT_STORAGE_URL and OBJECT_STORAGE_TOKEN; run make test-object-storage"]
fn live_export_snapshot_round_trip() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let objects = ObjectStorage::from_env(runtime.handle().clone()).unwrap();
    verify_round_trip(&Database::new(), objects);
}
