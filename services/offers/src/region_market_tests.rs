use diesel::{Connection, PgConnection, RunQueryDsl, connection::SimpleConnection};
#[derive(diesel::QueryableByName)]
struct Market {
    #[diesel(sql_type=diesel::sql_types::Jsonb)]
    value: serde_json::Value,
}
#[test]
#[ignore = "requires DATABASE_URL_TEST"]
fn postgres_tests_region_migration_preserves_values_and_blocks_unknown_labels() {
    let mut connection =
        PgConnection::establish(&std::env::var("DATABASE_URL_TEST").unwrap()).unwrap();
    connection.batch_execute("CREATE TEMP TABLE read_offers (target_market jsonb); INSERT INTO read_offers VALUES (NULL), ('{\"kind\":\"country\",\"code\":\"CA\"}'), ('{\"kind\":\"region\",\"label\":\"European Union\"}');").unwrap();
    let up = include_str!("../migrations/2026-10-29-000010_region_market_codes/up.sql");
    let down = include_str!("../migrations/2026-10-29-000010_region_market_codes/down.sql");
    connection.batch_execute(up).unwrap();
    let value: Market = diesel::sql_query(
        "SELECT target_market AS value FROM read_offers WHERE target_market->>'kind'='region'",
    )
    .get_result(&mut connection)
    .unwrap();
    assert_eq!(
        value.value,
        serde_json::json!({"kind":"region","code":"eu"})
    );
    connection.batch_execute(down).unwrap();
    connection
        .batch_execute(
            "INSERT INTO read_offers VALUES ('{\"kind\":\"region\",\"label\":\"Europe\"}')",
        )
        .unwrap();
    let result = connection.transaction::<(), diesel::result::Error, _>(|c| c.batch_execute(up));
    assert!(result.is_err());
    let value: Market = diesel::sql_query("SELECT target_market AS value FROM read_offers WHERE target_market->>'label'='European Union'").get_result(&mut connection).unwrap();
    assert_eq!(value.value["label"], "European Union");
}
