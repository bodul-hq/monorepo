use super::*;
use crate::identifier::Identifier;
use diesel::sql_types::{Jsonb, Nullable, Text, Uuid as SqlUuid};

fn offers_snapshot(pool: &DbPool) -> Value {
    snapshot(
        pool,
        "SELECT jsonb_agg(to_jsonb(o) - ARRAY['was_price_amount','was_price_currency','availability','item_condition','description'] ORDER BY id) AS data FROM offers o",
    )
}

fn auxiliary_snapshot(pool: &DbPool) -> Value {
    snapshot(pool, "SELECT jsonb_build_object(
        'images',(SELECT jsonb_agg(to_jsonb(i) ORDER BY position) FROM offer_images i),
        'receipts',(SELECT jsonb_agg(to_jsonb(r) ORDER BY message_id) FROM applied_offer_messages r),
        'commands',(SELECT jsonb_agg(to_jsonb(c) ORDER BY id) FROM command_entries c),
        'inbox',(SELECT jsonb_agg(to_jsonb(i) ORDER BY id) FROM inbox_entries i)
    ) AS data")
}

fn as_nested(mut rows: Value) -> Value {
    for row in rows.as_array_mut().unwrap() {
        let object = row.as_object_mut().unwrap();
        let mut external = serde_json::Map::new();
        for key in [
            "shopify_product_gid",
            "shopify_product_variation_gid",
            "shopify_handle",
        ] {
            external.insert(key.into(), object.remove(key).unwrap());
        }
        object.insert("external_identifiers".into(), Value::Object(external));
    }
    rows
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn migration_preserves_existing_values_and_rolls_back_latest_json_values() {
    let (_schema, pool) = database(true);
    revert_to(&pool, "20261026000004");
    for (id, product, variant, handle) in [
        (
            1,
            Some("gid://shopify/Product/0001"),
            Some("gid://shopify/ProductVariant/0002"),
            Some(" ž \\\" 001 "),
        ),
        (2, None, Some("00001"), None),
        (3, None, None, None),
    ] {
        diesel::sql_query("INSERT INTO offers(id,retailer_code,title,price_amount,currency,url,created_at,scraped_at,target_market,language,gtin,sku,shopify_product_gid,shopify_product_variation_gid,shopify_handle) VALUES ($1,'ankerau','Charger',1999,'AUD',$2,'2026-10-26T12:00:00Z','2026-10-26T12:01:00Z','{\"kind\":\"country\",\"code\":\"IT\"}','it','0012345678905','sku 001',$3,$4,$5)")
            .bind::<SqlUuid,_>(Uuid::from_u128(id)).bind::<Text,_>(format!("https://example.com/{id}"))
            .bind::<Nullable<Text>,_>(product).bind::<Nullable<Text>,_>(variant).bind::<Nullable<Text>,_>(handle)
            .execute(&mut pool.get().unwrap()).unwrap();
    }
    pool.get().unwrap().batch_execute("INSERT INTO offer_images(offer_id,position,image_url) VALUES ('00000000-0000-0000-0000-000000000001',0,'https://example.com/image'); INSERT INTO applied_offer_messages(message_id) VALUES ('00000000-0000-0000-0000-000000000011')").unwrap();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    for (kind, payload) in [
        (
            "OfferCreated",
            json!({"id":Uuid::from_u128(4),"retailer_code":"AnkerAu","title":"Legacy pending charger","price":{"amount":1999,"currency":"AUD"},"url":"https://example.com/4","created_at":"2026-10-26T12:00:00Z"}),
        ),
        (
            "OfferIdentifiersChanged",
            json!({"id":Uuid::from_u128(1),"gtin":null,"sku":"pending sku","external_identifiers":{"shopify_product_gid":null,"shopify_product_variation_gid":null,"shopify_handle":"pending update"}}),
        ),
    ] {
        client.post("/api/inbox").body_json(&json!({"message_id":Uuid::now_v7(),"source":"offers","routing_key":kind,"payload":payload})).send().await.assert_status(StatusCode::ACCEPTED);
    }
    assert_eq!(
        run(&client, "inbox", 100).await,
        json!({"processed":2,"errors":[]})
    );
    let before = offers_snapshot(&pool);
    let auxiliary = auxiliary_snapshot(&pool);
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(offers_snapshot(&pool), as_nested(before.clone()));
    assert_eq!(auxiliary_snapshot(&pool), auxiliary);
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='offers' AND column_name IN ('shopify_product_gid','shopify_product_variation_gid','shopify_handle')"
        ),
        0
    );
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM pg_indexes WHERE schemaname=current_schema() AND indexname='offers_title_search_idx'"
        ),
        1
    );
    let changed = json!({"shopify_product_gid":null,"shopify_product_variation_gid":"002","shopify_handle":"new ž \\\""});
    let identifier = Identifier(serde_json::from_value(changed.clone()).unwrap());
    diesel::sql_query("UPDATE offers SET external_identifiers=$1 WHERE id=$2")
        .bind::<Jsonb, _>(identifier)
        .bind::<SqlUuid, _>(Uuid::from_u128(1))
        .execute(&mut pool.get().unwrap())
        .unwrap();
    let after = offers_snapshot(&pool);
    revert_to(&pool, "20261026000004");
    let rolled_back = offers_snapshot(&pool);
    assert_eq!(rolled_back[0]["shopify_product_gid"], Value::Null);
    assert_eq!(rolled_back[0]["shopify_handle"], changed["shopify_handle"]);
    assert_eq!(as_nested(rolled_back), after);
    assert_eq!(auxiliary_snapshot(&pool), auxiliary);
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(offers_snapshot(&pool), after);
    assert_eq!(auxiliary_snapshot(&pool), auxiliary);
    assert_eq!(
        run(&client, "commands", 100).await,
        json!({"processed":2,"errors":[]})
    );
    let processed = offers_snapshot(&pool);
    assert_eq!(processed.as_array().unwrap().len(), 4);
    assert_eq!(
        processed[0]["external_identifiers"]["shopify_handle"],
        "pending update"
    );
    assert_eq!(
        processed[3]["external_identifiers"],
        json!({"shopify_product_gid":null,"shopify_product_variation_gid":null,"shopify_handle":null})
    );
    kernel.shutdown();
}

#[derive(diesel::QueryableByName)]
struct Typed {
    #[diesel(sql_type = Jsonb)]
    identifiers: Identifier,
}

#[test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
fn typed_jsonb_and_database_shape_reject_invalid_values() {
    let (_schema, pool) = database(true);
    let mut connection = pool.get().unwrap();
    connection.batch_execute("INSERT INTO offers(id,retailer_code,title,price_amount,currency,url,created_at) VALUES ('00000000-0000-0000-0000-000000000001','ankerau','Charger',0,'AUD','https://example.com/1',now())").unwrap();
    let empty = json!({"shopify_product_gid":null,"shopify_product_variation_gid":null,"shopify_handle":null});
    assert_eq!(offers_snapshot(&pool)[0]["external_identifiers"], empty);
    let full = json!({"shopify_product_gid":"001","shopify_product_variation_gid":"002","shopify_handle":" Ž\\\" "});
    for json in [empty.clone(), full] {
        let identifier = Identifier(serde_json::from_value(json.clone()).unwrap());
        let row = diesel::sql_query("SELECT $1 AS identifiers")
            .bind::<Jsonb, _>(&identifier)
            .get_result::<Typed>(&mut connection)
            .unwrap();
        assert_eq!(row.identifiers, identifier);
        assert_eq!(serde_json::to_value(row.identifiers.0).unwrap(), json);
    }
    let mut shapes = vec![
        Value::Null,
        json!([]),
        json!([null, null, null]),
        json!(1),
        json!("str"),
        json!({}),
    ];
    for key in [
        "shopify_product_gid",
        "shopify_product_variation_gid",
        "shopify_handle",
    ] {
        let mut missing = empty.clone();
        missing.as_object_mut().unwrap().remove(key);
        shapes.push(missing);
        for invalid in [json!(1), json!(true), json!([]), json!({})] {
            let mut shape = empty.clone();
            shape[key] = invalid;
            shapes.push(shape);
        }
    }
    let mut extra = empty.clone();
    extra["extra"] = Value::Null;
    shapes.push(extra);
    for value in shapes {
        assert!(
            diesel::sql_query("SELECT $1 AS identifiers")
                .bind::<Jsonb, _>(&value)
                .get_result::<Typed>(&mut connection)
                .is_err(),
            "{value}"
        );
        assert!(
            diesel::sql_query("UPDATE offers SET external_identifiers=$1")
                .bind::<Jsonb, _>(&value)
                .execute(&mut connection)
                .is_err(),
            "{value}"
        );
    }
    assert!(
        diesel::sql_query("UPDATE offers SET external_identifiers=NULL")
            .execute(&mut connection)
            .is_err()
    );
    assert!(
        diesel::sql_query("SELECT NULL::jsonb AS identifiers")
            .get_result::<Typed>(&mut connection)
            .is_err()
    );
    for value in ["", "a\0b"] {
        let invalid = Identifier(retailer_sauce::identifier::ExternalIdentifiers {
            shopify_handle: Some(retailer_sauce::identifier::ShopifyHandle {
                value: value.into(),
            }),
            ..Default::default()
        });
        assert!(
            diesel::sql_query("UPDATE offers SET external_identifiers=$1")
                .bind::<Jsonb, _>(invalid)
                .execute(&mut connection)
                .is_err()
        );
    }
    assert_eq!(offers_snapshot(&pool)[0]["external_identifiers"], empty);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
fn migration_failure_retains_old_columns_and_values() {
    let (_schema, pool) = database(true);
    revert_to(&pool, "20261026000004");
    pool.get().unwrap().batch_execute("INSERT INTO offers(id,retailer_code,title,price_amount,currency,url,created_at,shopify_handle) VALUES ('00000000-0000-0000-0000-000000000001','ankerau','Charger',0,'AUD','https://example.com/1',now(),'')").unwrap();
    let before = offers_snapshot(&pool);
    assert!(bootstrap::migrate(&pool).is_err());
    assert_eq!(offers_snapshot(&pool), before);
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='offers' AND column_name='external_identifiers'"
        ),
        0
    );
    pool.get()
        .unwrap()
        .batch_execute("UPDATE offers SET shopify_handle='  '")
        .unwrap();
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(
        offers_snapshot(&pool)[0]["external_identifiers"]["shopify_handle"],
        "  "
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn invalid_stored_identifier_errors_are_sanitized_in_browse_and_search() {
    let (_schema, pool) = database(true);
    // The SQL constraint checks shape, while Rust also validates string values.
    pool.get().unwrap().batch_execute("INSERT INTO offers(id,retailer_code,title,price_amount,currency,url,created_at,external_identifiers) VALUES ('00000000-0000-0000-0000-000000000001','ankerau','Charger',0,'AUD','https://example.com/1',now(),'{\"shopify_product_gid\":null,\"shopify_product_variation_gid\":null,\"shopify_handle\":\"\"}')").unwrap();
    let (kernel, state) = start_mulac(pool).unwrap();
    let client = TestClient::new(crate::app(state));
    for route in ["/api/offers", "/api/offers/search?q=charger"] {
        let response = client.get(route).send().await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        response.assert_text("offers could not be retrieved").await;
    }
    kernel.shutdown();
}
