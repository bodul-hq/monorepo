use diesel::result::Error;
use diesel::sql_query;
use serde_json::Value;
use serde_json::json;
use std::env::var;

use diesel::{Connection, PgConnection, RunQueryDsl, connection::SimpleConnection};
#[derive(diesel::QueryableByName)]
struct Market {
    #[diesel(sql_type=diesel::sql_types::Jsonb)]
    value: Value,
}
#[test]
#[ignore = "requires DATABASE_URL_TEST"]
fn postgres_tests_region_migration_preserves_values_and_blocks_unknown_labels() {
    let mut connection = PgConnection::establish(&var("DATABASE_URL_TEST").unwrap()).unwrap();
    connection.batch_execute("CREATE TEMP TABLE offers (target_market jsonb); INSERT INTO offers VALUES (NULL), ('{\"kind\":\"country\",\"code\":\"CA\"}'), ('{\"kind\":\"region\",\"label\":\"European Union\"}');").unwrap();
    let up = include_str!("../migrations/2026-10-29-000010_region_market_codes/up.sql");
    let down = include_str!("../migrations/2026-10-29-000010_region_market_codes/down.sql");
    connection.batch_execute(up).unwrap();
    let value: Market = sql_query(
        "SELECT target_market AS value FROM offers WHERE target_market->>'kind'='region'",
    )
    .get_result(&mut connection)
    .unwrap();
    assert_eq!(value.value, json!({"kind":"region","code":"eu"}));
    connection.batch_execute(down).unwrap();
    connection
        .batch_execute("INSERT INTO offers VALUES ('{\"kind\":\"region\",\"label\":\"Europe\"}')")
        .unwrap();
    let result = connection.transaction::<(), Error, _>(|c| c.batch_execute(up));
    assert!(result.is_err());
    let value: Market = sql_query(
        "SELECT target_market AS value FROM offers WHERE target_market->>'label'='European Union'",
    )
    .get_result(&mut connection)
    .unwrap();
    assert_eq!(value.value["label"], "European Union");
}
