//! Storage migrations run in isolated schemas, including populated rollbacks.
use super::test_db::{Database, json_query};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use serde_json::{Value, json};
use uuid::Uuid;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
const TABLES: [(&str, &str, Option<&str>); 7] = [
    (
        "sitemaps_raw",
        "raw_sitemap_documents_data",
        Some("body_size"),
    ),
    (
        "sitemaps_processed",
        "processed_sitemaps_data",
        Some("document_size"),
    ),
    ("sitemaps_consolidated", "consolidated_sitemaps_data", None),
    ("offer_details_raw", "raw_offers_data", Some("body_size")),
    ("offer_details_extracted", "extracted_offers_data", None),
    ("offer_details_normalized", "normalized_offers_data", None),
    (
        "offer_details_structuralized",
        "structuralized_offers_data",
        None,
    ),
];

fn snapshot(conn: &mut PgConnection) -> Value {
    let mut result = json!({});
    for (table, payload, _) in TABLES {
        for name in [table, payload] {
            result[name] = json_query(
                conn,
                &format!(
                    "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id),'[]') AS value FROM {name} t"
                ),
            );
        }
    }
    result
}

fn seed_legacy(conn: &mut PgConnection) {
    // Include whitespace, escapes and multibyte characters inside strings, as
    // well as every JSON container/scalar kind accepted by the offer stages.
    let documents = [
        json!({"a b": ["ž 🔎  spaced : ,", "quote\" and \\ slash\n\t", {"empty": {}}], "n": 12.5}),
        json!([]),
        json!({}),
        json!(null),
        json!(true),
        json!(42),
        json!(" ž  🔎 "),
    ];
    for (table, payload, size) in TABLES {
        for document in &documents {
            let id = Uuid::now_v7();
            let (columns, values, payload_column, data) = match table {
                "sitemaps_raw" => (
                    "retrieval_id,url,body_size",
                    "gen_random_uuid(),'https://shop.test/sitemap',2147483647",
                    "body",
                    json!("ž 🔎"),
                ),
                "sitemaps_processed" => (
                    "retrieval_id,retailer_code,document_size,url_count",
                    "gen_random_uuid(),'ankerau',0,0",
                    "document",
                    document.clone(),
                ),
                "sitemaps_consolidated" => (
                    "stats,\"references\",storage",
                    "'{}','{}','{\"content_size\":123,\"extra\":true}'",
                    "content",
                    document.clone(),
                ),
                "offer_details_raw" => (
                    "offer_id,url,body_size,retailer_code",
                    "gen_random_uuid(),'https://shop.test/a',7,'ankerau'",
                    "body",
                    json!("ž 🔎"),
                ),
                _ => (
                    "offer_id",
                    "gen_random_uuid()",
                    "document",
                    document.clone(),
                ),
            };
            diesel::sql_query(format!(
                "INSERT INTO {table} (id,{columns}) VALUES ($1,{values})"
            ))
            .bind::<SqlUuid, _>(id)
            .execute(conn)
            .unwrap();
            let expression = if size == Some("body_size") {
                "$2 #>> '{}'"
            } else {
                "$2"
            };
            diesel::sql_query(format!(
                "INSERT INTO {payload} (id,{payload_column}) VALUES ($1,{expression})"
            ))
            .bind::<SqlUuid, _>(id)
            .bind::<Jsonb, _>(data)
            .execute(conn)
            .unwrap();
        }
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_storage_preserves_rows_and_payloads_through_up_down_up() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap();
    seed_legacy(&mut conn);
    let before = snapshot(&mut conn);
    conn.run_next_migration(MIGRATIONS).unwrap();
    let after = snapshot(&mut conn);
    for (table, payload, size) in TABLES {
        assert_eq!(after[payload], before[payload]);
        for (old, new) in before[table]
            .as_array()
            .unwrap()
            .iter()
            .zip(after[table].as_array().unwrap())
        {
            let mut expected = old.clone();
            if let Some(column) = size {
                let size = expected.as_object_mut().unwrap().remove(column).unwrap();
                expected["storage"] = json!({"content_size": size});
            } else if table != "sitemaps_consolidated" {
                let data = before[payload]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|d| d["id"] == old["id"])
                    .unwrap();
                expected["storage"] = json!({"content_size": data["document"].to_string().len()});
            }
            assert_eq!(*new, expected, "{table}");
            assert!(new.get("body_size").is_none());
            assert!(new.get("document_size").is_none());
        }
        assert!(
            conn.batch_execute(&format!("UPDATE {table} SET storage=NULL"))
                .is_err()
        );
    }
    conn.revert_last_migration(MIGRATIONS).unwrap();
    assert_eq!(snapshot(&mut conn), before);
    conn.run_next_migration(MIGRATIONS).unwrap();
    assert_eq!(snapshot(&mut conn), after);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_storage_upgrade_rejects_missing_payloads_atomically() {
    for payload in [
        "extracted_offers_data",
        "normalized_offers_data",
        "structuralized_offers_data",
    ] {
        let db = Database::new();
        let mut conn = db.pool.get().unwrap();
        conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
        conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
        conn.revert_last_migration(MIGRATIONS).unwrap();
        seed_legacy(&mut conn);
        conn.batch_execute(&format!("DELETE FROM {payload}"))
            .unwrap();
        let before = snapshot(&mut conn);
        assert!(conn.run_next_migration(MIGRATIONS).is_err());
        assert_eq!(snapshot(&mut conn), before);
        assert_eq!(
            conn.applied_migrations().unwrap()[0].to_string(),
            "20261015000009"
        );
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_storage_downgrade_rejects_invalid_sizes_atomically() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap();
    seed_legacy(&mut conn);
    conn.run_next_migration(MIGRATIONS).unwrap();
    for (table, _, size) in TABLES {
        if size.is_none() {
            continue;
        }
        let valid = json_query(
            &mut conn,
            &format!("SELECT storage AS value FROM {table} LIMIT 1"),
        );
        for bad in [
            json!({}),
            json!(null),
            json!({"content_size": null}),
            json!({"content_size": "7"}),
            json!({"content_size": 1.5}),
            json!({"content_size": 2147483648_i64}),
        ] {
            diesel::sql_query(format!("UPDATE {table} SET storage=$1"))
                .bind::<Jsonb, _>(bad)
                .execute(&mut conn)
                .unwrap();
            let before = snapshot(&mut conn);
            assert!(conn.revert_last_migration(MIGRATIONS).is_err(), "{table}");
            assert_eq!(snapshot(&mut conn), before);
            assert_eq!(
                conn.applied_migrations().unwrap()[0].to_string(),
                "20261015000010"
            );
        }
        diesel::sql_query(format!("UPDATE {table} SET storage=$1"))
            .bind::<Jsonb, _>(valid)
            .execute(&mut conn)
            .unwrap();
    }
    conn.revert_last_migration(MIGRATIONS).unwrap();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_typed_storage_round_trips_unsigned_sizes() {
    use super::db::{Json, Storage};
    #[derive(Debug, QueryableByName)]
    struct Row {
        #[diesel(sql_type = Jsonb)]
        value: Json<Storage>,
    }
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    for content_size in [0, 7, u64::MAX] {
        let input = Json(Storage { content_size });
        let output: Row = diesel::sql_query("SELECT $1 AS value")
            .bind::<Jsonb, _>(&input)
            .get_result(&mut conn)
            .unwrap();
        assert_eq!(output.value, input);
    }
    assert!(matches!(
        diesel::sql_query("SELECT NULL::jsonb AS value").get_result::<Row>(&mut conn),
        Err(diesel::result::Error::DeserializationError(_))
    ));
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_raw_object_cutover_removes_only_raw_sitemaps_and_guards_rollback() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Remaining object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Storage JSON.
    seed_legacy(&mut conn);
    conn.run_next_migration(MIGRATIONS).unwrap();
    let before = snapshot(&mut conn);
    conn.run_next_migration(MIGRATIONS).unwrap();
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_raw"
        ),
        json!(0)
    );
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(to_regclass('raw_sitemap_documents_data') IS NULL) AS value"
        ),
        json!(true)
    );
    for (table, payload, _) in TABLES {
        if table == "sitemaps_raw" {
            continue;
        }
        for name in [table, payload] {
            assert_eq!(
                json_query(
                    &mut conn,
                    &format!("SELECT jsonb_agg(to_jsonb(t) ORDER BY id) AS value FROM {name} t")
                ),
                before[name]
            );
        }
    }
    conn.revert_last_migration(MIGRATIONS).unwrap();
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM raw_sitemap_documents_data"
        ),
        json!(0)
    );
    conn.run_next_migration(MIGRATIONS).unwrap();
    conn.batch_execute("INSERT INTO sitemaps_raw (id,retrieval_id,url,storage) VALUES (gen_random_uuid(),gen_random_uuid(),'https://shop.test',jsonb_build_object('content_size',7,'object_storage_id',gen_random_uuid()))").unwrap();
    let row = json_query(&mut conn, "SELECT to_jsonb(r) AS value FROM sitemaps_raw r");
    assert!(conn.revert_last_migration(MIGRATIONS).is_err());
    assert_eq!(
        conn.applied_migrations().unwrap()[0].to_string(),
        "20261015000011"
    );
    assert_eq!(
        json_query(&mut conn, "SELECT to_jsonb(r) AS value FROM sitemaps_raw r"),
        row
    );
    conn.batch_execute("DELETE FROM sitemaps_raw").unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_remaining_objects_cutover_preserves_raw_sitemaps_and_bus_history() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    for _ in 0..3 {
        conn.revert_last_migration(MIGRATIONS).unwrap();
    }
    seed_legacy(&mut conn);
    conn.run_next_migration(MIGRATIONS).unwrap(); // Storage metadata.
    conn.run_next_migration(MIGRATIONS).unwrap(); // Raw sitemap objects.
    conn.batch_execute("INSERT INTO sitemaps_raw (id,retrieval_id,url,storage) VALUES (gen_random_uuid(),gen_random_uuid(),'https://shop.test/raw',jsonb_build_object('content_size',7,'object_storage_id',gen_random_uuid()));
        INSERT INTO inbox_entries(id,payload,meta) VALUES(gen_random_uuid(),'test','{}');
        INSERT INTO outbox_entries(id,payload,meta) VALUES(gen_random_uuid(),'test','{}');
        INSERT INTO command_entries(id,command_type,payload) VALUES(gen_random_uuid(),'Test','test');
        INSERT INTO event_entries(id,event_type,payload) VALUES(gen_random_uuid(),'Test','test');").unwrap();
    let retained = [
        "sitemaps_raw",
        "inbox_entries",
        "outbox_entries",
        "command_entries",
        "event_entries",
    ];
    let snapshot = |conn: &mut PgConnection| {
        retained.map(|table| {
            json_query(
                conn,
                &format!("SELECT jsonb_agg(to_jsonb(t) ORDER BY id) AS value FROM {table} t"),
            )
        })
    };
    let before = snapshot(&mut conn);
    conn.run_next_migration(MIGRATIONS).unwrap();
    assert_eq!(snapshot(&mut conn), before);
    for (table, payload, _) in TABLES.into_iter().skip(1) {
        assert_eq!(
            json_query(
                &mut conn,
                &format!("SELECT to_jsonb(count(*)) AS value FROM {table}")
            ),
            json!(0)
        );
        assert_eq!(
            json_query(
                &mut conn,
                &format!("SELECT to_jsonb(to_regclass('{payload}') IS NULL) AS value")
            ),
            json!(true)
        );
    }
    conn.revert_last_migration(MIGRATIONS).unwrap();
    for (_, payload, _) in TABLES.into_iter().skip(1) {
        assert_eq!(
            json_query(
                &mut conn,
                &format!("SELECT to_jsonb(count(*)) AS value FROM {payload}")
            ),
            json!(0)
        );
    }
    conn.run_next_migration(MIGRATIONS).unwrap();
    for (table, _, _) in TABLES.into_iter().skip(1) {
        let columns = match table {
            "sitemaps_processed" => "retrieval_id,retailer_code,url_count,storage",
            "sitemaps_consolidated" => "stats,\"references\",storage",
            "offer_details_raw" => "offer_id,url,retailer_code,storage",
            _ => "offer_id,storage",
        };
        let values = match table {
            "sitemaps_processed" => "gen_random_uuid(),'ankerau',0,'{}'",
            "sitemaps_consolidated" => "'{}','{}','{}'",
            "offer_details_raw" => "gen_random_uuid(),'https://shop.test','ankerau','{}'",
            _ => "gen_random_uuid(),'{}'",
        };
        conn.batch_execute(&format!(
            "INSERT INTO {table}(id,{columns}) VALUES(gen_random_uuid(),{values})"
        ))
        .unwrap();
        assert!(conn.revert_last_migration(MIGRATIONS).is_err());
        assert_eq!(
            conn.applied_migrations().unwrap()[0].to_string(),
            "20261016000012"
        );
        assert_eq!(
            json_query(
                &mut conn,
                &format!("SELECT to_jsonb(count(*)) AS value FROM {table}")
            ),
            json!(1)
        );
        conn.batch_execute(&format!("DELETE FROM {table}")).unwrap();
    }
    assert_eq!(snapshot(&mut conn), before);
    conn.revert_last_migration(MIGRATIONS).unwrap();
}
