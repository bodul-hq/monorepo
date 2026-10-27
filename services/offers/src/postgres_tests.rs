use diesel::{Connection, PgConnection, connection::SimpleConnection, r2d2::ConnectionManager};
use diesel_migrations::MigrationHarness;
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};
use testimony::prelude::EventStore;
use testimony_infra::prelude::DbPool;
use uuid::Uuid;

use crate::{
    bootstrap::MIGRATIONS,
    infra::repository::OfferEventStore,
    offers::{event::RetailerOfferEvent, retailer_offer_id::RetailerOfferId},
};

pub(crate) struct TestSchema {
    connection: PgConnection,
    name: String,
}

impl Drop for TestSchema {
    fn drop(&mut self) {
        self.connection
            .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.name))
            .expect("drop isolated test schema");
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn tmp_import_stores_command_in_inbox_and_reports_storage_failure() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let input = json!({
        "retailer_code": "ankerau", "title": "USB charger", "price_amount": 1999,
        "currency": "AUD", "url": "https://example.com/charger?color=red"
    });
    let response = client
        .post("/api/tmp-import")
        .body_json(&input)
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    let body: Value = response.json().await.value().deserialize();
    let message_id = Uuid::parse_str(body["message_id"].as_str().unwrap()).unwrap();

    // The import is only stored: no command, event or offer exists yet.
    let stored = inbox_test_snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i");
    assert_eq!(stored["id"], message_id.to_string());
    assert_eq!(
        serde_json::from_str::<Value>(stored["payload"].as_str().unwrap()).unwrap(),
        json!({
            "retailer_code":"AnkerAu", "title":"USB charger",
            "price":{"amount":1999,"currency":"AUD"},
            "url":"https://example.com/charger?color=red"
        })
    );
    assert_eq!(
        stored["meta"],
        json!({
            "message_id":message_id,"correlation_id":null,"source":"offers-api",
            "routing_key":crate::offers::command::CREATE_FROM_OFFER_DETAILS_COMMAND
        })
    );
    let counts = || {
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('commands', (SELECT count(*) FROM command_entries), 'events', (SELECT count(*) FROM event_store)) AS data",
        )
    };
    assert_eq!(counts(), json!({"commands":0,"events":0}));

    // Running the inbox queues the command; running commands saves the offer.
    run_queue(&client, "inbox", 10, 1).await;
    assert_eq!(counts(), json!({"commands":1,"events":0}));
    run_queue(&client, "commands", 10, 1).await;
    let stored_id = inbox_test_snapshot(&pool, "SELECT to_jsonb(agg_id) AS data FROM event_store");
    let id = stored_id.as_str().unwrap();
    let events = OfferEventStore::new(pool.clone())
        .load(RetailerOfferId::new(id))
        .unwrap();
    assert_eq!(events.len(), 1);
    let RetailerOfferEvent::OfferCreated(event) = events[0].event() else {
        panic!("expected creation event");
    };
    assert_eq!(event.title, "USB charger");
    assert_eq!(event.url, input["url"].as_str().unwrap());
    assert_eq!(event.price, money::Money::new(1999, money::Currency::AUD));
    assert_eq!(
        event.retailer_code,
        bodul_shared::retailer::RetailerCode::from_str("ankerau").unwrap()
    );

    // Inbox storage failures hide database diagnostics.
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE inbox_entries")
        .unwrap();
    let response = client
        .post("/api/tmp-import")
        .body_json(&input)
        .send()
        .await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    response
        .assert_text("inbox message could not be stored")
        .await;
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn reporting_returns_schema_statistics_and_exact_queue_counts() {
    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = url::Url::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let client = TestClient::new(crate::app(pool.clone()));
    let token = format!("Bearer {}", crate::reporting::DEFAULT_REPORTING_TOKEN);
    let response = client
        .get("/api/reporting/database-tables")
        .header("authorization", &token)
        .send()
        .await;
    response.assert_status_is_ok();
    assert_eq!(response.0.headers()["cache-control"], "no-store");
    let report: Value = response.json().await.value().deserialize();
    assert_eq!(report["schema_name"], schema.name);
    assert_eq!(report["row_count_kind"], "estimated");
    let tables = report["tables"].as_array().unwrap();
    assert!(
        tables
            .iter()
            .any(|table| table["table_name"] == "event_store")
    );
    let names: Vec<_> = tables
        .iter()
        .map(|table| table["table_name"].as_str().unwrap())
        .collect();
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    for table in tables {
        assert!(table["row_count"].as_u64().is_some());
        assert_eq!(
            table["total_bytes"].as_u64().unwrap(),
            table["table_bytes"].as_u64().unwrap() + table["index_bytes"].as_u64().unwrap()
        );
    }
    let response = client
        .get("/api/reporting/entry-statuses")
        .header("authorization", &token)
        .send()
        .await;
    response.assert_status_is_ok();
    let empty: Value = response.json().await.value().deserialize();
    assert_eq!(empty["command_entries"], json!([]));
    assert_eq!(empty["event_entries"], json!([]));
    pool.get()
        .unwrap()
        .batch_execute(
            "INSERT INTO command_entries (id, command_type, payload, status) VALUES
         ('00000000-0000-0000-0000-000000000001','test','{}',0),
         ('00000000-0000-0000-0000-000000000002','test','{}',0),
         ('00000000-0000-0000-0000-000000000003','test','{}',99);
         INSERT INTO event_entries (id, event_type, payload, status) VALUES
         ('00000000-0000-0000-0000-000000000004','test','{}',5);",
        )
        .unwrap();
    let response = client
        .get("/api/reporting/entry-statuses")
        .header("authorization", &token)
        .send()
        .await;
    response.assert_status_is_ok();
    let report: Value = response.json().await.value().deserialize();
    assert_eq!(
        report["command_entries"],
        json!([
            {"status":0,"status_text":"received","count":2},
            {"status":99,"status_text":"unknown","count":1}
        ])
    );
    assert_eq!(
        report["event_entries"],
        json!([{"status":5,"status_text":"completed","count":1}])
    );
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE event_entries")
        .unwrap();
    client
        .get("/api/reporting/entry-statuses")
        .header("authorization", &token)
        .send()
        .await
        .assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    client
        .get("/api/reporting/database-tables")
        .header("authorization", &token)
        .send()
        .await
        .assert_status_is_ok();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn identifiers_insert_preserves_identity_and_rejects_duplicates() {
    use crate::infra::repository::PostgresRetailerOfferIdentifiersRepository;
    use crate::offers::handler::RetailerOfferIdentifiersRepository;
    use bodul_shared::retailer::RetailerCode;
    use diesel::{QueryableByName, RunQueryDsl, sql_types::Text};

    #[derive(QueryableByName)]
    struct Identity {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        retailer_code: String,
        #[diesel(sql_type = Text)]
        url: String,
    }

    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut database_url = url::Url::parse(&database_url).unwrap();
    database_url
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(
            database_url.to_string(),
        ))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let repository = PostgresRetailerOfferIdentifiersRepository::new(pool.clone());
    let id = RetailerOfferId::new(Uuid::new_v4().to_string());
    let retailer = RetailerCode::from_str("ankerau").unwrap();
    let url = "https://example.com/Charger?color=red#details";
    assert!(repository.find(retailer, url).unwrap().is_none());
    repository.insert(&id, retailer, url).unwrap();
    let found = repository.find(retailer, url).unwrap().unwrap();
    assert_eq!(found.id, id);
    assert_eq!(found.retailer_code, retailer);
    assert_eq!(found.url, url);
    assert!(
        repository
            .find(retailer, &url.to_lowercase())
            .unwrap()
            .is_none()
    );
    assert!(
        repository
            .find(RetailerCode::AnkerCom, url)
            .unwrap()
            .is_none()
    );
    assert!(
        repository
            .find(retailer, &format!("{url}/"))
            .unwrap()
            .is_none()
    );

    // Renaming in either direction must preserve existing identities.
    {
        let mut connection = pool.get().unwrap();
        // Revert later repairs too, so this still reaches the rename migration.
        while connection
            .revert_last_migration(MIGRATIONS)
            .unwrap()
            .to_string()
            != "20261019000003"
        {}
        let previous: Identity =
            diesel::sql_query("SELECT id::text, retailer_code, url FROM offer_identities")
                .get_result(&mut connection)
                .unwrap();
        assert_eq!(previous.id, id.value());
        assert_eq!(previous.retailer_code, "ankerau");
        assert_eq!(previous.url, url);
        connection.run_pending_migrations(MIGRATIONS).unwrap();
    }
    let row: Identity =
        diesel::sql_query("SELECT id::text, retailer_code, url FROM read_offer_identities")
            .get_result(&mut pool.get().unwrap())
            .unwrap();
    assert_eq!(row.id, id.value());
    assert_eq!(row.retailer_code, "ankerau");
    assert_eq!(row.url, url);
    assert!(
        repository
            .insert(&id, retailer, "https://example.com/other")
            .is_err()
    );
    assert!(
        repository
            .insert(
                &RetailerOfferId::new(Uuid::new_v4().to_string()),
                retailer,
                url
            )
            .is_err()
    );
    assert!(
        repository
            .insert(&RetailerOfferId::new("not-a-uuid"), retailer, url)
            .is_err()
    );
    repository
        .insert(
            &RetailerOfferId::new(Uuid::new_v4().to_string()),
            retailer,
            "https://example.com/charger?color=red#details",
        )
        .unwrap();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn read_repositories_insert_contracts() {
    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut database_url = url::Url::parse(&database_url).unwrap();
    database_url
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(
            database_url.to_string(),
        ))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    // Verify a fresh schema can roll back completely and be rebuilt.
    {
        let mut connection = pool.get().unwrap();
        connection.revert_all_migrations(MIGRATIONS).unwrap();
        assert!(connection.applied_migrations().unwrap().is_empty());
        use diesel::RunQueryDsl;
        #[derive(diesel::QueryableByName)]
        struct TableName {
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }
        let tables = |connection: &mut PgConnection| {
            diesel::sql_query(
                "SELECT tablename::text AS name FROM pg_tables WHERE schemaname = current_schema() AND tablename <> '__diesel_schema_migrations' ORDER BY tablename",
            ).load::<TableName>(connection).unwrap().into_iter().map(|row| row.name).collect::<Vec<_>>()
        };
        assert!(tables(&mut connection).is_empty());
        connection.run_pending_migrations(MIGRATIONS).unwrap();
        assert_eq!(
            tables(&mut connection),
            vec![
                "command_entries",
                "event_entries",
                "event_store",
                "inbox_entries",
                "outbox_entries",
                "read_offer_identities",
                "read_offers",
            ]
        );
    }
    crate::read_offer::repository::tests::assert_insert_contract(
        &crate::read_offer::repository::PostgresRetailerOfferRepository::new(pool.clone()),
    );
    crate::read_offers_identifiers::repository::tests::assert_insert_contract(
        &crate::read_offers_identifiers::repository::PostgresRetailerOfferIdentifiersRepository::new(pool.clone()),
    );
    #[derive(diesel::QueryableByName)]
    struct StoredOffer {
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        price_amount: i64,
        #[diesel(sql_type = diesel::sql_types::Text)]
        currency: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }
    use diesel::RunQueryDsl;
    let rows =
        diesel::sql_query("SELECT title, price_amount, currency, created_at FROM read_offers")
            .load::<StoredOffer>(&mut pool.get().unwrap())
            .unwrap();
    assert_eq!(rows.len(), 2);
    for row in rows {
        assert_eq!(row.title, "Charger");
        assert_eq!(row.price_amount, 1999);
        assert_eq!(row.currency, "AUD");
        assert_eq!(
            row.created_at,
            chrono::DateTime::from_timestamp(1_700_000_000, 123_000_000).unwrap()
        );
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn registered_projectors_create_and_update_read_models() {
    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut database_url = url::Url::parse(&database_url).unwrap();
    database_url
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(
            database_url.to_string(),
        ))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    use crate::offers::event::{OfferPriceChangedEvent, OfferTitleChangedEvent};
    use diesel::RunQueryDsl;
    use kernel::ApplicationEvent;
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".to_owned()));
    let response = client
        .post("/api/tmp-import")
        .body_json(&json!({
            "retailer_code": "ankerau", "title": "Charger", "price_amount": 1999,
            "currency": "AUD", "url": "https://example.com/Charger?q=red#details"
        }))
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    run_queue(&client, "inbox", 10, 1).await;
    run_queue(&client, "commands", 10, 1).await;
    run_queue(&client, "events", 10, 1).await;
    run_queue(&client, "events", 10, 0).await;
    #[derive(diesel::QueryableByName)]
    struct Projection {
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        data: Value,
    }
    let read = || {
        diesel::sql_query("SELECT to_jsonb(o) AS data FROM read_offers o JOIN read_offer_identities i USING (id, retailer_code, url)")
        .get_result::<Projection>(&mut pool.get().unwrap()).unwrap().data
    };
    let before = read();
    let id = RetailerOfferId::new(before["id"].as_str().unwrap());
    assert_eq!(before["id"], id.value());
    assert_eq!(before["title"], "Charger");
    assert_eq!(before["price_amount"], 1999);
    assert_eq!(before["currency"], "AUD");
    assert_eq!(before["retailer_code"], "ankerau");
    assert_eq!(before["url"], "https://example.com/Charger?q=red#details");
    let title = OfferTitleChangedEvent {
        id: id.clone(),
        title: "Updated charger".to_owned(),
    };
    let price = OfferPriceChangedEvent {
        id: id.clone(),
        price: money::Money::new(1299, money::Currency::EUR),
    };
    for (event_type, payload) in [
        (title.event_type(), serde_json::to_string(&title).unwrap()),
        (price.event_type(), serde_json::to_string(&price).unwrap()),
    ] {
        let event_id = Uuid::new_v4();
        diesel::sql_query(
            "INSERT INTO event_entries (id, event_type, payload, meta) VALUES ($1, $2, $3, $4)",
        )
        .bind::<diesel::sql_types::Uuid, _>(event_id)
        .bind::<diesel::sql_types::Text, _>(event_type)
        .bind::<diesel::sql_types::Text, _>(payload)
        .bind::<diesel::sql_types::Jsonb, _>(json!({
            "event_id": event_id,
            "correlation_id": Uuid::new_v4(),
            "causation_id": Uuid::new_v4(),
            "source": "offers-test"
        }))
        .execute(&mut pool.get().unwrap())
        .unwrap();
        run_queue(&client, "events", 10, 1).await;
    }
    let mut expected = before;
    expected["title"] = json!("Updated charger");
    expected["price_amount"] = json!(1299);
    expected["currency"] = json!("EUR");
    assert_eq!(read(), expected);
    // Registration covers all offer events, preserving their wire payloads and metadata.
    use kernel::EventSubscriberPort;
    let subscriber =
        crate::subscribers::customer_facing::CustomerFacingSubscriber::new(pool.clone());
    let events = diesel::sql_query("SELECT to_jsonb(e) AS data FROM event_entries e")
        .load::<Projection>(&mut pool.get().unwrap())
        .unwrap();
    assert_eq!(events.len(), 3);
    for event in events {
        let envelope = kernel::NewEventEnvelope {
            event_type: event.data["event_type"].as_str().unwrap().to_owned(),
            payload: event.data["payload"].as_str().unwrap().to_owned(),
            metadata: Some(serde_json::from_value(event.data["meta"].clone()).unwrap()),
        };
        let read_outbox = || {
            diesel::sql_query("SELECT to_jsonb(o) AS data FROM outbox_entries o WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(envelope.metadata.as_ref().unwrap().event_id)
                .get_result::<Projection>(&mut pool.get().unwrap())
                .unwrap()
                .data
        };
        let before = read_outbox();
        assert_eq!(before["payload"], event.data["payload"]);
        assert_eq!(before["meta"]["event_type"], event.data["event_type"]);
        assert_eq!(before["meta"]["routing_key"], "customer_facing");
        assert_eq!(before["meta"]["content_type"], "application/json");
        for key in ["event_id", "correlation_id", "causation_id", "source"] {
            assert_eq!(before["meta"][key], event.data["meta"][key]);
        }
        assert_eq!(before["status"], 0);
        subscriber.handle(&envelope).unwrap();
        assert_eq!(read_outbox(), before);
    }
    #[derive(diesel::QueryableByName)]
    struct OutboxCount {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }
    let count = diesel::sql_query("SELECT count(*) AS count FROM outbox_entries")
        .get_result::<OutboxCount>(&mut pool.get().unwrap())
        .unwrap()
        .count;
    assert_eq!(count, 3);
    use crate::read_offer::repository::RetailerOfferRepository;
    let repository = crate::read_offer::repository::PostgresRetailerOfferRepository::new(pool);
    let missing = RetailerOfferId::new(Uuid::new_v4().to_string());
    assert!(repository.update_title(&missing, "missing").is_err());
    assert!(repository.update_price(&missing, &price.price).is_err());
    kernel.shutdown();
}

pub(crate) async fn run_queue(
    client: &TestClient<poem::Route>,
    queue: &str,
    limit: usize,
    processed: usize,
) {
    let response = client
        .post(format!("/api/{queue}/run"))
        .body_json(&json!({"limit": limit}))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(json!({"processed": processed, "errors": []}))
        .await;
}

pub(crate) fn inbox_test_database() -> (TestSchema, DbPool) {
    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).unwrap(),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = url::Url::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    (schema, pool)
}

#[derive(diesel::QueryableByName)]
struct InboxTestSnapshot {
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    data: Value,
}

pub(crate) fn inbox_test_snapshot(pool: &DbPool, sql: &str) -> Value {
    use diesel::RunQueryDsl;
    diesel::sql_query(sql)
        .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
        .unwrap()
        .data
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn inbox_submission_preserves_payload_metadata_and_duplicate_rows() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let message_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let input = json!({
        "payload":{"nested":[null,true,42,"é",{"key":"value"}]},
        "routing_key":" unknown.command ",
        "message_id":message_id,
        "correlation_id":correlation_id,
        "source":"external-source"
    });
    let response = client.post("/api/inbox").body_json(&input).send().await;
    response.assert_status(StatusCode::ACCEPTED);
    response.assert_json(json!({"message_id":message_id})).await;
    let sql = "SELECT to_jsonb(i) AS data FROM inbox_entries i";
    let original = inbox_test_snapshot(&pool, sql);
    assert_eq!(original["id"], message_id.to_string());
    assert_eq!(
        serde_json::from_str::<Value>(original["payload"].as_str().unwrap()).unwrap(),
        input["payload"]
    );
    assert_eq!(
        original["meta"],
        json!({
            "message_id":message_id,"correlation_id":correlation_id,
            "source":"external-source","routing_key":" unknown.command "
        })
    );
    assert_eq!(original["status"], 0);
    assert_eq!(original["attempts"], 0);
    assert!(original["reservation_id"].is_null());
    assert!(original["processed_at"].is_null());

    // A changed envelope with the same ID is successful and changes no column.
    let response = client
        .post("/api/inbox")
        .body_json(&json!({
            "message_id":message_id,"payload":"replacement","routing_key":"different",
            "source":"different","correlation_id":Uuid::new_v4()
        }))
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    response.assert_json(json!({"message_id":message_id})).await;
    assert_eq!(inbox_test_snapshot(&pool, sql), original);

    // Rejected envelopes must not insert messages.
    for body in inbox_api::io::testing::invalid_submissions() {
        client
            .post("/api/inbox")
            .body_json(&body)
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
    assert_eq!(inbox_test_snapshot(&pool, sql), original);

    let mut ids = std::collections::HashSet::from([message_id]);
    for payload in [
        json!(0),
        json!([1, "two"]),
        json!("text"),
        json!(42),
        json!(true),
    ] {
        let response = client
            .post("/api/inbox")
            .body_json(&json!({"payload":payload,"routing_key":"unknown","source":" crawler "}))
            .send()
            .await;
        response.assert_status(StatusCode::ACCEPTED);
        let body: Value = response.json().await.value().deserialize();
        let id = Uuid::parse_str(body["message_id"].as_str().unwrap()).unwrap();
        assert_eq!(body.as_object().unwrap().len(), 1);
        assert!(!id.is_nil());
        assert!(ids.insert(id));
        use diesel::RunQueryDsl;
        let stored =
            diesel::sql_query("SELECT to_jsonb(i) AS data FROM inbox_entries i WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(id)
                .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
                .unwrap()
                .data;
        assert_eq!(
            serde_json::from_str::<Value>(stored["payload"].as_str().unwrap()).unwrap(),
            payload
        );
        assert_eq!(
            stored["meta"],
            json!({"message_id":id,"correlation_id":null,"source":" crawler ","routing_key":"unknown"})
        );
    }
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('inbox', (SELECT count(*) FROM inbox_entries), 'commands', (SELECT count(*) FROM command_entries), 'events', (SELECT count(*) FROM event_entries), 'offers', (SELECT count(*) FROM read_offers)) AS data"
        ),
        json!({"inbox":6,"commands":0,"events":0,"offers":0})
    );

    // An actual SQL failure must also hide database diagnostics.
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE inbox_entries")
        .unwrap();
    let response = client.post("/api/inbox").body_json(&input).send().await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    response
        .assert_text("inbox message could not be stored")
        .await;
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn inbox_submission_processes_offer_through_all_http_stages() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let command = crate::offers::command::ImportOfferDetailsCommand {
        retailer_code: bodul_shared::retailer::RetailerCode::AnkerAu,
        title: "USB charger".into(),
        price: money::Money::new(1999, money::Currency::AUD),
        url: "https://example.com/Charger?q=red#details".into(),
    };
    let response = client
        .post("/api/inbox")
        .body_json(&json!({
            "routing_key":crate::offers::command::CREATE_FROM_OFFER_DETAILS_COMMAND,
            "source":"crawler",
            "payload": {
                "retailer_code":"AnkerAu", "title":"USB charger",
                "price":{"amount":1999,"currency":"AUD"},
                "url":"https://example.com/Charger?q=red#details"
            }
        }))
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    let counts = || {
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('commands', (SELECT count(*) FROM command_entries), 'events', (SELECT count(*) FROM event_entries), 'offers', (SELECT count(*) FROM read_offers)) AS data",
        )
    };
    assert_eq!(counts(), json!({"commands":0,"events":0,"offers":0}));
    run_queue(&client, "inbox", 10, 1).await;
    assert_eq!(counts(), json!({"commands":1,"events":0,"offers":0}));
    run_queue(&client, "commands", 10, 1).await;
    assert_eq!(counts(), json!({"commands":1,"events":1,"offers":0}));
    run_queue(&client, "events", 10, 1).await;
    let projection = inbox_test_snapshot(
        &pool,
        "SELECT to_jsonb(o) AS data FROM read_offers o JOIN read_offer_identities i USING (id, retailer_code, url)",
    );
    Uuid::parse_str(projection["id"].as_str().unwrap()).unwrap();
    assert_eq!(projection["title"], command.title);
    assert_eq!(projection["retailer_code"], "ankerau");
    assert_eq!(projection["url"], command.url);
    assert_eq!(projection["price_amount"], 1999);
    assert_eq!(projection["currency"], "AUD");
    for queue in ["inbox", "commands", "events"] {
        run_queue(&client, queue, 10, 0).await;
    }
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn queue_endpoints_process_one_stage_and_preserve_unavailable_outbox() {
    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).unwrap(),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = url::Url::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    use diesel::{
        RunQueryDsl,
        sql_types::{Jsonb, Text, Uuid as SqlUuid},
    };
    for number in 0..3 {
        let command = crate::offers::command::ImportOfferDetailsCommand {
            retailer_code: bodul_shared::retailer::RetailerCode::AnkerAu,
            title: "Charger".into(),
            price: money::Money::new(1999, money::Currency::AUD),
            url: format!("https://example.com/charger/{number}"),
        };
        diesel::sql_query("INSERT INTO inbox_entries (id, payload, meta, scheduled_at) VALUES ($1, $2, $3, NOW() + $4 * INTERVAL '1 day')")
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<Text, _>(serde_json::to_string(&command).unwrap())
            .bind::<Jsonb, _>(json!({"routing_key": crate::offers::command::CREATE_FROM_OFFER_DETAILS_COMMAND}))
            .bind::<diesel::sql_types::Integer, _>(if number == 2 { 1 } else { 0 })
            .execute(&mut pool.get().unwrap()).unwrap();
    }
    // Each endpoint advances only its own queue, and respects the batch limit.
    run_queue(&client, "commands", 10, 0).await;
    run_queue(&client, "inbox", 1, 1).await;
    run_queue(&client, "events", 10, 0).await;
    run_queue(&client, "commands", 10, 1).await;
    run_queue(&client, "commands", 10, 0).await;
    run_queue(&client, "events", 10, 1).await;
    run_queue(&client, "inbox", 10, 1).await;
    run_queue(&client, "inbox", 10, 0).await;
    run_queue(&client, "commands", 1, 1).await;
    run_queue(&client, "events", 1, 1).await;
    run_queue(&client, "events", 10, 0).await;

    #[derive(diesel::QueryableByName)]
    struct Snapshot {
        #[diesel(sql_type = Jsonb)]
        data: Value,
    }
    let snapshot = |sql: &str| {
        diesel::sql_query(sql)
            .get_result::<Snapshot>(&mut pool.get().unwrap())
            .unwrap()
            .data
    };
    let projections = snapshot(
        "SELECT to_jsonb(count(*)) AS data FROM read_offers JOIN read_offer_identities USING (id, retailer_code, url)",
    );
    assert_eq!(projections, 2);

    pool.get()
        .unwrap()
        .batch_execute(
            "INSERT INTO inbox_entries (id, payload, meta) VALUES (gen_random_uuid(), '{}', '{}');
         INSERT INTO outbox_entries (id, payload, meta) VALUES (gen_random_uuid(), '{}', '{}');",
        )
        .unwrap();
    let response = client
        .post("/api/inbox/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    let report: Value = response.json().await.value().deserialize();
    assert_eq!(report["processed"], Value::Null);
    assert_eq!(report["errors"].as_array().unwrap().len(), 1);
    let failed = snapshot(
        "SELECT jsonb_build_object('attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM inbox_entries WHERE meta = '{}'::jsonb",
    );
    assert_eq!(
        failed,
        json!({"attempts":1, "reserved":false, "scheduled_later":true, "processed":false})
    );
    run_queue(&client, "inbox", 10, 0).await;

    let outbox_sql = "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o";
    let before = snapshot(outbox_sql);
    client
        .post("/api/outbox/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await
        .assert_status(StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(snapshot(outbox_sql), before);
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn outbox_delivery_completes_only_confirmed_items_and_retries_failures() {
    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).unwrap(),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = url::Url::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    use diesel::RunQueryDsl;
    use outbox::io::{OutboundMessageEnvelope, OutboxError, OutboxPublisherPort};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Publisher(AtomicBool);
    impl OutboxPublisherPort for Publisher {
        fn publish(&self, message: OutboundMessageEnvelope) -> Result<(), OutboxError> {
            assert_eq!(message.metadata.routing_key, "customer_facing");
            if self.0.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err(err_test_delivery())
            }
        }
    }
    fn err_test_delivery() -> OutboxError {
        OutboxError::Transport("test broker unavailable".into())
    }
    let publisher = Arc::new(Publisher(AtomicBool::new(false)));
    let (kernel, mut state) = crate::start_mulac(pool.clone()).unwrap();
    state.outbox = mulac_poem::io::OutboxAdapter::new(
        outbox_api::io::OutboxService::from_postgres(pool.clone(), publisher.clone()),
    );
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    use kernel::EventSubscriberPort;
    let subscriber =
        crate::subscribers::customer_facing::CustomerFacingSubscriber::new(pool.clone());
    for _ in 0..2 {
        subscriber
            .handle(&kernel::NewEventEnvelope {
                event_type: crate::offers::event::OFFER_CREATED_EVENT.into(),
                payload: "{}".into(),
                metadata: Some(kernel::NewEventMetadata {
                    event_id: Uuid::new_v4(),
                    correlation_id: None,
                    causation_id: None,
                    source: Some("offers-test".into()),
                }),
            })
            .unwrap();
    }
    // Existing rows used null IDs; the migration makes them consumable too.
    pool.get()
        .unwrap()
        .batch_execute(
            "UPDATE outbox_entries SET meta = jsonb_set(meta, '{message_id}', 'null'::jsonb)",
        )
        .unwrap();
    pool.get()
        .unwrap()
        .batch_execute(include_str!(
            "../migrations/2026-10-21-000001_outbox_message_ids/up.sql"
        ))
        .unwrap();
    let response = client
        .post("/api/outbox/run")
        .body_json(&json!({"limit":1}))
        .send()
        .await;
    response.assert_status_is_ok();
    let body: Value = response.json().await.value().deserialize();
    assert!(body["processed"].is_null());
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
    #[derive(diesel::QueryableByName)]
    struct Snapshot {
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        data: Value,
    }
    let failed: Snapshot = diesel::sql_query("SELECT jsonb_build_object('attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM outbox_entries WHERE attempts = 1").get_result(&mut pool.get().unwrap()).unwrap();
    assert_eq!(
        failed.data,
        json!({"attempts":1,"reserved":false,"scheduled_later":true,"processed":false})
    );
    publisher.0.store(true, Ordering::SeqCst);
    run_queue(&client, "outbox", 1, 1).await;
    run_queue(&client, "outbox", 10, 0).await;
    pool.get()
        .unwrap()
        .batch_execute("UPDATE outbox_entries SET scheduled_at = NOW() WHERE processed_at IS NULL")
        .unwrap();
    run_queue(&client, "outbox", 1, 1).await;
    run_queue(&client, "outbox", 10, 0).await;
    let completed: Snapshot = diesel::sql_query("SELECT to_jsonb(count(*)) AS data FROM outbox_entries WHERE processed_at IS NOT NULL AND reservation_id IS NULL").get_result(&mut pool.get().unwrap()).unwrap();
    assert_eq!(completed.data, 2);
    kernel.shutdown();
}

fn insert_command(pool: &DbPool, id: Uuid, command_type: &str, payload: &str) {
    use diesel::{
        RunQueryDsl,
        sql_types::{Text, Uuid as SqlUuid},
    };
    diesel::sql_query(
        "INSERT INTO command_entries (id, command_type, payload) VALUES ($1, $2, $3)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(command_type)
    .bind::<Text, _>(payload)
    .execute(&mut pool.get().unwrap())
    .unwrap();
}

fn command_snapshot(pool: &DbPool, id: Uuid) -> Value {
    use diesel::RunQueryDsl;
    diesel::sql_query(
        "SELECT jsonb_build_object('status', status, 'attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM command_entries WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
    .unwrap()
    .data
}

async fn run_commands_expecting_errors(client: &TestClient<poem::Route>, errors: Value) {
    let response = client
        .post("/api/commands/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(json!({"processed":null,"errors":errors}))
        .await;
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn commands_run_reschedules_failed_commands_until_dead() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let id = Uuid::new_v4();
    insert_command(&pool, id, "unregistered", "{}");
    let not_found = json!(["no handler registered for command type 'unregistered'"]);

    run_commands_expecting_errors(&client, not_found.clone()).await;
    assert_eq!(
        command_snapshot(&pool, id),
        json!({"status":4, "attempts":1, "reserved":false, "scheduled_later":true, "processed":false})
    );
    // The backoff keeps the failed command out of the next batch.
    run_queue(&client, "commands", 10, 0).await;

    for attempts in 2..=6 {
        pool.get()
            .unwrap()
            .batch_execute("UPDATE command_entries SET scheduled_at = NOW()")
            .unwrap();
        run_commands_expecting_errors(&client, not_found.clone()).await;
        let status = if attempts == 6 { 7 } else { 4 };
        assert_eq!(
            command_snapshot(&pool, id),
            json!({"status":status, "attempts":attempts, "reserved":false, "scheduled_later":true, "processed":false})
        );
    }
    // Dead commands are never reserved again.
    pool.get()
        .unwrap()
        .batch_execute("UPDATE command_entries SET scheduled_at = NOW()")
        .unwrap();
    run_queue(&client, "commands", 10, 0).await;
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM event_entries"
        ),
        0
    );
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn commands_run_keeps_successful_commands_of_a_failed_batch() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let command = crate::offers::command::ImportOfferDetailsCommand {
        retailer_code: bodul_shared::retailer::RetailerCode::AnkerAu,
        title: "Charger".into(),
        price: money::Money::new(1999, money::Currency::AUD),
        url: "https://example.com/charger".into(),
    };
    let valid = Uuid::new_v4();
    let unregistered = Uuid::new_v4();
    insert_command(
        &pool,
        valid,
        crate::offers::command::CREATE_FROM_OFFER_DETAILS_COMMAND,
        &serde_json::to_string(&command).unwrap(),
    );
    insert_command(&pool, unregistered, "unregistered", "{}");

    run_commands_expecting_errors(
        &client,
        json!(["no handler registered for command type 'unregistered'"]),
    )
    .await;
    assert_eq!(
        command_snapshot(&pool, valid),
        json!({"status":5, "attempts":1, "reserved":false, "scheduled_later":false, "processed":true})
    );
    assert_eq!(command_snapshot(&pool, unregistered)["status"], 4);
    // The completed command's event is recorded but not dispatched.
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('events', (SELECT count(*) FROM event_entries WHERE status = 0), 'offers', (SELECT count(*) FROM read_offers)) AS data"
        ),
        json!({"events":1,"offers":0})
    );
    kernel.shutdown();
}

fn event_snapshot(pool: &DbPool, id: Uuid) -> Value {
    use diesel::RunQueryDsl;
    diesel::sql_query(
        "SELECT jsonb_build_object('status', status, 'attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM event_entries WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
    .unwrap()
    .data
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn events_run_reschedules_subscriber_failures_and_keeps_completed_events() {
    use diesel::RunQueryDsl;
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    client
        .post("/api/tmp-import")
        .body_json(&json!({
            "retailer_code": "ankerau", "title": "Charger", "price_amount": 1999,
            "currency": "AUD", "url": "https://example.com/charger"
        }))
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
    run_queue(&client, "inbox", 10, 1).await;
    run_queue(&client, "commands", 10, 1).await;
    // CustomerFacingSubscriber rejects an OfferCreated event without metadata.
    pool.get()
        .unwrap()
        .batch_execute("UPDATE event_entries SET meta = NULL")
        .unwrap();
    let failing = inbox_test_snapshot(&pool, "SELECT to_jsonb(id) AS data FROM event_entries");
    let failing = Uuid::parse_str(failing.as_str().unwrap()).unwrap();
    // An event type without subscribers completes and is counted.
    let unsubscribed = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO event_entries (id, event_type, payload) VALUES ($1, 'unsubscribed', '{}')",
    )
    .bind::<diesel::sql_types::Uuid, _>(unsubscribed)
    .execute(&mut pool.get().unwrap())
    .unwrap();

    let response = client
        .post("/api/events/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(json!({"processed":null,"errors":[
            "subscriber execution error: event metadata is required for customer-facing outbox delivery"
        ]}))
        .await;
    assert_eq!(
        event_snapshot(&pool, failing),
        json!({"status":4, "attempts":1, "reserved":false, "scheduled_later":true, "processed":false})
    );
    assert_eq!(event_snapshot(&pool, unsubscribed)["status"], 5);
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM outbox_entries"
        ),
        0
    );
    // The backoff keeps the failed event out of the next batch.
    run_queue(&client, "events", 10, 0).await;
    kernel.shutdown();
}
