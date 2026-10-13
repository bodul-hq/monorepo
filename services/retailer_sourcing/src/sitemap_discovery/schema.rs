// @generated automatically by Diesel CLI.

diesel::table! {
    consolidated_sitemaps (id) {
        id -> Uuid,
        processed_sitemap_id -> Uuid,
        retrieval_id -> Uuid,
        retailer_code -> Text,
        content -> Jsonb,
        content_size -> Int4,
        product_count -> Int4,
        catalog_count -> Int4,
        content_count -> Int4,
        not_interested_count -> Int4,
        unknown_count -> Int4,
        consolidated_at -> Timestamptz,
    }
}

diesel::table! {
    processed_sitemaps (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        retailer_code -> Text,
        document -> Jsonb,
        document_size -> Int4,
        url_count -> Int4,
        processed_at -> Timestamptz,
    }
}

diesel::table! {
    raw_sitemap_documents (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        url -> Text,
        last_modified -> Nullable<Timestamptz>,
        body -> Text,
        body_size -> Int4,
        fetched_at -> Timestamptz,
    }
}

diesel::joinable!(consolidated_sitemaps -> processed_sitemaps (processed_sitemap_id));

diesel::allow_tables_to_appear_in_same_query!(
    consolidated_sitemaps,
    processed_sitemaps,
    raw_sitemap_documents,
);
