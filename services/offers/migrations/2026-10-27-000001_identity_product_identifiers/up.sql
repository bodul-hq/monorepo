-- Stop event runners while deploying this projection extension. Diesel applies
-- the validation, backfill and verification in one transaction.
LOCK TABLE read_offers IN SHARE MODE;
LOCK TABLE read_offer_identities IN ACCESS EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM read_offer_identities i JOIN read_offers o USING (id)
        WHERE i.retailer_code IS DISTINCT FROM o.retailer_code
           OR i.url IS DISTINCT FROM o.url
    ) THEN
        RAISE EXCEPTION 'Offer identity backfill found conflicting retailer/URL values for the same offer ID';
    END IF;
    IF EXISTS (
        SELECT 1 FROM read_offer_identities i JOIN read_offers o USING (id, retailer_code, url)
        WHERE o.gtin = '' OR o.sku = '' OR o.shopify_product_gid = ''
           OR o.shopify_product_variation_gid = '' OR o.shopify_handle = ''
    ) THEN
        RAISE EXCEPTION 'Offer identity backfill found empty product identifiers; repair the source projection before migrating';
    END IF;
END $$;

ALTER TABLE read_offer_identities
    ADD COLUMN gtin TEXT,
    ADD COLUMN sku TEXT,
    ADD COLUMN shopify_product_gid TEXT,
    ADD COLUMN shopify_product_variation_gid TEXT,
    ADD COLUMN shopify_handle TEXT;

UPDATE read_offer_identities i SET
    gtin = o.gtin,
    sku = o.sku,
    shopify_product_gid = o.shopify_product_gid,
    shopify_product_variation_gid = o.shopify_product_variation_gid,
    shopify_handle = o.shopify_handle
FROM read_offers o
WHERE i.id = o.id AND i.retailer_code = o.retailer_code AND i.url = o.url;

DO $$
DECLARE unmatched BIGINT;
BEGIN
    IF EXISTS (
        SELECT 1 FROM read_offer_identities i JOIN read_offers o USING (id, retailer_code, url)
        WHERE i.gtin IS DISTINCT FROM o.gtin
           OR i.sku IS DISTINCT FROM o.sku
           OR i.shopify_product_gid IS DISTINCT FROM o.shopify_product_gid
           OR i.shopify_product_variation_gid IS DISTINCT FROM o.shopify_product_variation_gid
           OR i.shopify_handle IS DISTINCT FROM o.shopify_handle
    ) THEN
        RAISE EXCEPTION 'Offer identity identifier backfill verification failed';
    END IF;
    SELECT count(*) INTO unmatched FROM read_offer_identities i
    WHERE NOT EXISTS (SELECT 1 FROM read_offers o WHERE o.id = i.id);
    RAISE NOTICE 'Offer identity identifier backfill: % identities have no read_offers row and retain null identifiers', unmatched;
END $$;
