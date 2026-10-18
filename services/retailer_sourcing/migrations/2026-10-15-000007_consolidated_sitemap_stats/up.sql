-- Keep domain counts together in the PostgreSQL representation.
ALTER TABLE consolidated_sitemaps ADD COLUMN stats JSONB;
UPDATE consolidated_sitemaps SET stats = jsonb_build_object(
    'product_count', product_count,
    'catalog_count', catalog_count,
    'content_count', content_count,
    'not_interested_count', not_interested_count,
    'unknown_count', unknown_count
);
ALTER TABLE consolidated_sitemaps ALTER COLUMN stats SET NOT NULL;
ALTER TABLE consolidated_sitemaps
    DROP COLUMN product_count,
    DROP COLUMN catalog_count,
    DROP COLUMN content_count,
    DROP COLUMN not_interested_count,
    DROP COLUMN unknown_count;
