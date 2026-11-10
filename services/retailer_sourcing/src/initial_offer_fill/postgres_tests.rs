//! Durable end-to-end tests in an isolated schema, with local object-storage fixtures.
use super::{
    acquire_storefront_token as token, offer_details as details, product_catalog_ids as catalog,
    tests::{connection, product, token_value},
    transport::Http,
};
use crate::platform::{
    bus::{Bus, Event},
    state::AppState,
    test_db::{Database, json_query},
    test_objects::Server,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use diesel::{
    connection::SimpleConnection,
    prelude::*,
    sql_types::{Jsonb, Text, Uuid as SqlUuid},
};
use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontToken};
use serde_json::{Value, json};
use shared::retailer::RetailerCode;
use tower::ServiceExt;
use uuid::Uuid;

struct Documents;
impl token::handler::Fetch for Documents {
    fn configured_token(&self, _: &str) -> Result<Option<StorefrontToken>, String> {
        Ok(Some(token_value()))
    }
    fn get(&self, _: &str) -> Result<String, String> {
        Ok(r#"<script>storeTokens:{au:'synthetic-au-token',ca:'synthetic-ca-token',us:'public-us',uk:'public-uk',de:'public-de',fr:'public-fr','eu-en':'public-eu','eu-pl':'public-pl',nz:'public-nz',vn:'public-vn'}</script>"#.into())
    }
}
struct Source;
impl Http for Source {
    fn post(
        &self,
        url: &str,
        operation: &GraphqlRequest,
        _: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        match operation.operation_name.as_str() {
            "CatalogIdentifiers" => Ok(
                json!({"data":{"products":connection((1..=3).map(|i|json!({"cursor":format!("c{i}"),"node":{"id":format!("gid://shopify/Product/{i}")}})).collect(),false,Some("c3"))}}),
            ),
            "OfferDetails" => {
                let id = operation.variables["id"].as_str().unwrap();
                let retailer = RetailerCode::ALL
                    .into_iter()
                    .find(|r| {
                        retailer_data_extractor::initial_fill_config_for(*r)
                            .is_some_and(|c| c.graphql_url == url)
                    })
                    .unwrap();
                let mut value = product(id);
                value["priceRange"]["minVariantPrice"]["currencyCode"] = json!(
                    retailer_data_extractor::initial_fill_config_for(retailer)
                        .unwrap()
                        .currency
                );
                let currency = retailer_data_extractor::initial_fill_config_for(retailer)
                    .unwrap()
                    .currency;
                let value: Value =
                    serde_json::from_str(&value.to_string().replace("AUD", currency)).unwrap();
                Ok(json!({"data":{"product":value}}))
            }
            _ => Err("unexpected continuation".into()),
        }
    }
}
fn rows(db: &Database, table: &str) -> Value {
    json_query(
        &mut db.pool.get().unwrap(),
        &format!("SELECT coalesce(jsonb_agg(to_jsonb(t)), '[]') AS value FROM {table} t"),
    )
}
fn post(server: &Server, app: &axum::Router, path: &str, body: Value) -> (StatusCode, Value) {
    server.runtime.block_on(async {
        let response = app
            .clone()
            .oneshot(
                Request::post(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    })
}
fn run(server: &Server, app: &axum::Router, path: &str) {
    let (status, body) = post(server, app, path, json!({"limit":100}));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["errors"], json!([]), "{body}");
}
#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_initial_fill_reaches_terminal_stored_events_without_downstream_work() {
    run_initial_fill(RetailerCode::AnkerAu);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_canadian_initial_fill_reaches_terminal_stored_events_without_downstream_work() {
    run_initial_fill(RetailerCode::AnkerCa);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_remaining_anker_markets_preserve_terminal_event_contract() {
    for retailer in RetailerCode::ALL {
        if !matches!(retailer, RetailerCode::AnkerAu | RetailerCode::AnkerCa)
            && retailer_data_extractor::initial_fill_config_for(retailer).is_some()
        {
            run_initial_fill(retailer);
        }
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_bulk_initial_fill_validates_before_dispatch_and_preserves_run_identities() {
    let db = Database::new();
    let server = Server::new();
    let mut bus = Bus::default();
    bus.handle(token::handler::Handler(Documents));
    bus.handle(catalog::handler::Handler(Source));
    let guard = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = crate::app(AppState {
        bus: Some(bus.clone()),
        ..Default::default()
    });
    let (status, _) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers":["ankerau", "ankerkr"]}),
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(rows(&db, "command_entries").as_array().unwrap().is_empty());

    let (status, response) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers":["ankerau", "ankeritalycom", "ankerca", "ankerau"]}),
    );
    assert_eq!(status, StatusCode::ACCEPTED);
    let fills = response["fills"].as_array().unwrap();
    assert_eq!(
        fills
            .iter()
            .map(|f| f["retailer"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["ankerau", "ankeritalycom", "ankerca"]
    );
    assert_eq!(
        fills
            .iter()
            .map(|f| f["run_id"].as_str().unwrap())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3
    );
    let commands = rows(&db, "command_entries");
    let commands = commands.as_array().unwrap();
    assert_eq!(commands.len(), 3);
    for fill in fills {
        let command = commands
            .iter()
            .find(|c| c["meta"]["correlation_id"] == fill["run_id"])
            .unwrap();
        let payload: Value = serde_json::from_str(command["payload"].as_str().unwrap()).unwrap();
        assert_eq!(payload["run_id"], fill["run_id"]);
        assert_eq!(payload["retailer_code"], fill["retailer"]);
        let expected = "AcquireStorefrontToken";
        assert_eq!(command["command_type"], expected);
    }
    bus.shutdown();
}

fn run_initial_fill(retailer: RetailerCode) {
    let expected_commands = if retailer_data_extractor::initial_fill_config_for(retailer)
        .unwrap()
        .requires_token()
    {
        5
    } else {
        4
    };
    let db = Database::new();
    let server = Server::new();
    let mut bus = Bus::default();
    bus.handle(token::handler::Handler(Documents));
    bus.handle(catalog::handler::Handler(Source));
    bus.handle(details::handler::Handler {
        fetcher: Source,
        store: server.client(),
    });
    bus.on_many_keyed(catalog::reacts_to::acquired);
    bus.on_many_keyed(catalog::reacts_to::next_batch);
    bus.on_many_keyed(details::reacts_to::on);
    let guard = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = crate::app(AppState {
        bus: Some(bus.clone()),
        ..Default::default()
    });
    let (status, response) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers":[retailer.as_str()]}),
    );
    assert_eq!(status, StatusCode::ACCEPTED);
    let run_id = response["fills"][0]["run_id"].as_str().unwrap();
    for _ in 0..3 {
        run(&server, &app, "/api/commands/run");
        run(&server, &app, "/api/events/run");
    }
    let commands = rows(&db, "command_entries");
    assert_eq!(commands.as_array().unwrap().len(), expected_commands);
    assert!(
        commands
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] == 5)
    );
    assert!(
        commands
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["meta"]["correlation_id"] == run_id)
    );
    let events = rows(&db, "event_entries");
    let terminal: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "OfferDetailsFetched")
        .collect();
    assert_eq!(terminal.len(), 3);
    // Check the actual upload destination against object storage's existing
    // contract, independently of the handler's bucket constant.
    let data = server.data.lock().unwrap();
    assert_eq!(data.uploads.len(), 3);
    assert!(
        data.uploads
            .iter()
            .all(|(bucket, _)| bucket == "offer_details_raw")
    );
    drop(data);
    for row in terminal {
        assert_eq!(row["status"], 5);
        let event: details::event::OfferDetailsFetched =
            serde_json::from_str(row["payload"].as_str().unwrap()).unwrap();
        assert_eq!(event.retailer_code, retailer);
        let artifact = server
            .client()
            .get_offer(details::handler::BUCKET, &event.storage)
            .unwrap();
        let artifact: Value = serde_json::from_str(&artifact).unwrap();
        assert_eq!(artifact["schema_version"], 1);
        let config = retailer_data_extractor::initial_fill_config_for(retailer).unwrap();
        assert_eq!(
            artifact["pages"][0]["request"]["variables"]["country"],
            config.country
        );
        assert!(
            artifact["pages"][0]["request"]["query"]
                .as_str()
                .unwrap()
                .contains("language: $language")
        );
        assert_eq!(
            artifact["product"]["priceRange"]["minVariantPrice"]["currencyCode"],
            config.currency
        );
        assert_eq!(artifact["pages"].as_array().unwrap().len(), 1);
    }
    run(&server, &app, "/api/events/run");
    assert_eq!(
        rows(&db, "command_entries").as_array().unwrap().len(),
        expected_commands
    );
    assert!(rows(&db, "outbox_entries").as_array().unwrap().is_empty());
    bus.shutdown();
}
#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_partial_fanout_replay_deduplicates_command_rows() {
    let db = Database::new();
    let server = Server::new();
    let event = catalog::event::ProductCatalogIdsRetrieved {
        context: None,
        run_id: Uuid::now_v7(),
        retailer_code: RetailerCode::AnkerAu,
        storefront_token: Some(token_value()),
        after: None,
        product_ids: (1..=3)
            .map(|i| format!("gid://shopify/Product/{i}"))
            .collect(),
        next_cursor: None,
        retrieved_at: chrono::Utc::now(),
    };
    let failed_id = details::reacts_to::on(&event)[1].id;
    let mut conn = db.pool.get().unwrap();
    conn.batch_execute(&format!("CREATE FUNCTION fail_fanout() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.id = '{failed_id}'::uuid THEN RAISE EXCEPTION 'fixture failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER fail_fanout BEFORE INSERT ON command_entries FOR EACH ROW EXECUTE FUNCTION fail_fanout();")).unwrap();
    let event_id = Uuid::now_v7();
    diesel::sql_query("INSERT INTO event_entries (id,event_type,payload,meta,status) VALUES ($1,$2,$3,$4,0)")
        .bind::<SqlUuid,_>(event_id).bind::<Text,_>(catalog::event::ProductCatalogIdsRetrieved::NAME)
        .bind::<Text,_>(serde_json::to_string(&event).unwrap())
        .bind::<Jsonb,_>(json!({"event_id":event_id,"correlation_id":event.run_id,"causation_id":null,"source":"test"}))
        .execute(&mut conn).unwrap();
    drop(conn);
    let mut bus = Bus::default();
    bus.on_many_keyed(details::reacts_to::on);
    let guard = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = crate::app(AppState {
        bus: Some(bus.clone()),
        ..Default::default()
    });
    let (_, body) = post(&server, &app, "/api/events/run", json!({"limit":10}));
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 1);
    db.pool.get().unwrap().batch_execute("DROP TRIGGER fail_fanout ON command_entries; UPDATE event_entries SET scheduled_at=now() WHERE status=4").unwrap();
    run(&server, &app, "/api/events/run");
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 3);
    // Explicit replay of an already-processed batch must not insert more commands.
    db.pool
        .get()
        .unwrap()
        .batch_execute("UPDATE event_entries SET status=0, scheduled_at=now()")
        .unwrap();
    run(&server, &app, "/api/events/run");
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 3);
    bus.shutdown();
}
