-- Product identifiers remain in read_offers and immutable event history.
ALTER TABLE read_offer_identities
    DROP COLUMN gtin,
    DROP COLUMN sku,
    DROP COLUMN shopify_product_gid,
    DROP COLUMN shopify_product_variation_gid,
    DROP COLUMN shopify_handle;
