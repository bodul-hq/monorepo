-- Restore the original columns and constraints; invalid values or dangling
-- references fail the migration transaction without discarding data.
ALTER TABLE consolidated_sitemaps
    ADD COLUMN processed_sitemap_id UUID,
    ADD COLUMN retrieval_id UUID,
    ADD COLUMN retailer_code TEXT,
    ADD COLUMN content_size INT;
UPDATE consolidated_sitemaps SET
    processed_sitemap_id = ("references" ->> 'processed_sitemap_id')::UUID,
    retrieval_id = ("references" ->> 'retrieval_id')::UUID,
    retailer_code = "references" ->> 'retailer_code',
    content_size = (storage ->> 'content_size')::INTEGER;
ALTER TABLE consolidated_sitemaps
    ALTER COLUMN processed_sitemap_id SET NOT NULL,
    ALTER COLUMN retrieval_id SET NOT NULL,
    ALTER COLUMN retailer_code SET NOT NULL,
    ALTER COLUMN content_size SET NOT NULL,
    ADD CONSTRAINT consolidated_sitemaps_processed_sitemap_id_key UNIQUE (processed_sitemap_id),
    ADD CONSTRAINT consolidated_sitemaps_processed_sitemap_id_fkey
        FOREIGN KEY (processed_sitemap_id) REFERENCES processed_sitemaps(id) ON DELETE CASCADE;
DROP INDEX consolidated_sitemaps_processed_reference_key;
ALTER TABLE consolidated_sitemaps
    DROP COLUMN "references",
    DROP COLUMN storage;
