// @generated automatically by Diesel CLI.

diesel::table! {
    sitemaps_consolidated (id) {
        id -> Uuid,
        consolidated_at -> Timestamptz,
        stats -> Jsonb,
        references -> Jsonb,
        storage -> Jsonb,
    }
}

diesel::table! {
    sitemaps_processed (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        retailer_code -> Text,
        url_count -> Int4,
        processed_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::table! {
    sitemaps_raw (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        url -> Text,
        last_modified -> Nullable<Timestamptz>,
        fetched_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    sitemaps_consolidated,
    sitemaps_processed,
    sitemaps_raw,
);
