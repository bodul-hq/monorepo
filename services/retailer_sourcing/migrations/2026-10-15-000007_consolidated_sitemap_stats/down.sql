-- Restore the original columns without discarding stored counts or payloads.
ALTER TABLE consolidated_sitemaps
    ADD COLUMN product_count INT,
    ADD COLUMN catalog_count INT,
    ADD COLUMN content_count INT,
    ADD COLUMN not_interested_count INT,
    ADD COLUMN unknown_count INT;
UPDATE consolidated_sitemaps SET
    product_count = (stats ->> 'product_count')::INTEGER,
    catalog_count = (stats ->> 'catalog_count')::INTEGER,
    content_count = (stats ->> 'content_count')::INTEGER,
    not_interested_count = (stats ->> 'not_interested_count')::INTEGER,
    unknown_count = (stats ->> 'unknown_count')::INTEGER;
ALTER TABLE consolidated_sitemaps
    ALTER COLUMN product_count SET NOT NULL,
    ALTER COLUMN catalog_count SET NOT NULL,
    ALTER COLUMN content_count SET NOT NULL,
    ALTER COLUMN not_interested_count SET NOT NULL,
    ALTER COLUMN unknown_count SET NOT NULL,
    DROP COLUMN stats;
