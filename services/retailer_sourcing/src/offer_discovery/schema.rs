// @generated automatically by Diesel CLI.

diesel::table! {
    raw_offers (id) {
        id -> Uuid,
        offer_id -> Uuid,
        url -> Text,
        body -> Text,
        body_size -> Int4,
        fetched_at -> Timestamptz,
    }
}
