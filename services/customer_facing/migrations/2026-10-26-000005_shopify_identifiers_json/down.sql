ALTER TABLE offers ADD COLUMN shopify_product_gid TEXT,
    ADD COLUMN shopify_product_variation_gid TEXT, ADD COLUMN shopify_handle TEXT;

UPDATE offers SET
    shopify_product_gid = external_identifiers ->> 'shopify_product_gid',
    shopify_product_variation_gid = external_identifiers ->> 'shopify_product_variation_gid',
    shopify_handle = external_identifiers ->> 'shopify_handle';

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM offers WHERE
        shopify_product_gid IS DISTINCT FROM (external_identifiers ->> 'shopify_product_gid')
        OR shopify_product_variation_gid IS DISTINCT FROM (external_identifiers ->> 'shopify_product_variation_gid')
        OR shopify_handle IS DISTINCT FROM (external_identifiers ->> 'shopify_handle')
    ) THEN
        RAISE EXCEPTION 'Shopify identifier rollback verification failed';
    END IF;
END $$;

ALTER TABLE offers DROP CONSTRAINT offers_external_identifiers_shape,
    DROP COLUMN external_identifiers;
