// @generated automatically by Diesel CLI.

use diesel::allow_tables_to_appear_in_same_query;
use diesel::table;

table! {
    event_store (id) {
        id -> Int8,
        #[max_length = 255]
        agg_id -> Varchar,
        sequence -> Int8,
        #[max_length = 255]
        payload_type -> Varchar,
        payload -> Jsonb,
        metadata -> Jsonb,
        recorded_on -> Timestamptz,
    }
}

table! {
    inbox_records (id) {
        id -> Uuid,
        payload -> Text,
        meta -> Jsonb,
        status -> Int4,
        scheduled_at -> Timestamptz,
        attempts -> Int4,
        received_at -> Timestamptz,
        updated_at -> Timestamptz,
        processed_at -> Nullable<Timestamptz>,
    }
}

table! {
    outbox_records (id) {
        id -> Uuid,
        payload -> Text,
        meta -> Jsonb,
        status -> Int4,
        scheduled_at -> Timestamptz,
        attempts -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        published_at -> Nullable<Timestamptz>,
    }
}


allow_tables_to_appear_in_same_query!(
    event_store,
    inbox_records,
    outbox_records,
);
