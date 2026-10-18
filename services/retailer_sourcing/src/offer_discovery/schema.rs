// @generated automatically by Diesel CLI.

diesel::table! {
    offer_details_extracted (id) {
        id -> Uuid,
        offer_id -> Uuid,
        extracted_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::table! {
    offer_details_normalized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        normalized_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::table! {
    offer_details_raw (id) {
        id -> Uuid,
        offer_id -> Uuid,
        url -> Text,
        fetched_at -> Timestamptz,
        retailer_code -> Text,
        storage -> Jsonb,
    }
}

diesel::table! {
    offer_details_structuralized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        structuralized_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    offer_details_extracted,
    offer_details_normalized,
    offer_details_raw,
    offer_details_structuralized,
);
