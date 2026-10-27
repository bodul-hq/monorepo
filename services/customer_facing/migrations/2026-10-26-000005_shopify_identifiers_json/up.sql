-- Diesel runs this entire migration in one transaction. Do not run old binaries
-- concurrently: both old readers and writers reference the removed columns.
LOCK TABLE offers IN ACCESS EXCLUSIVE MODE;

DO $$
DECLARE invalid_ids TEXT;
BEGIN
    SELECT string_agg(id::text, ', ' ORDER BY id) INTO invalid_ids FROM offers
    WHERE shopify_product_gid = '' OR shopify_product_variation_gid = '' OR shopify_handle = '';
    IF invalid_ids IS NOT NULL THEN
        RAISE EXCEPTION 'Invalid empty Shopify identifiers; resolve rows before migrating: %', invalid_ids;
    END IF;
END $$;

ALTER TABLE offers ADD COLUMN external_identifiers JSONB NOT NULL DEFAULT
    '{"shopify_product_gid":null,"shopify_product_variation_gid":null,"shopify_handle":null}'::jsonb;

UPDATE offers SET external_identifiers = jsonb_build_object(
    'shopify_product_gid', shopify_product_gid,
    'shopify_product_variation_gid', shopify_product_variation_gid,
    'shopify_handle', shopify_handle
);

ALTER TABLE offers ADD CONSTRAINT offers_external_identifiers_shape CHECK ((
    jsonb_typeof(external_identifiers) = 'object'
    AND external_identifiers ?& ARRAY['shopify_product_gid','shopify_product_variation_gid','shopify_handle']
    AND external_identifiers - ARRAY['shopify_product_gid','shopify_product_variation_gid','shopify_handle'] = '{}'::jsonb
    AND jsonb_typeof(external_identifiers -> 'shopify_product_gid') IN ('string','null')
    AND jsonb_typeof(external_identifiers -> 'shopify_product_variation_gid') IN ('string','null')
    AND jsonb_typeof(external_identifiers -> 'shopify_handle') IN ('string','null')
) IS TRUE);

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM offers WHERE
        (external_identifiers ->> 'shopify_product_gid') IS DISTINCT FROM shopify_product_gid
        OR (external_identifiers ->> 'shopify_product_variation_gid') IS DISTINCT FROM shopify_product_variation_gid
        OR (external_identifiers ->> 'shopify_handle') IS DISTINCT FROM shopify_handle
    ) THEN
        RAISE EXCEPTION 'Shopify identifier backfill verification failed';
    END IF;
END $$;

ALTER TABLE offers DROP COLUMN shopify_product_gid,
    DROP COLUMN shopify_product_variation_gid, DROP COLUMN shopify_handle;
