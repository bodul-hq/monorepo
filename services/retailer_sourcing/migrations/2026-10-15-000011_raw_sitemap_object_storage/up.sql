-- Fresh-data-only cutover: old raw sitemap bodies and metadata are discarded.
-- Processed/consolidated sitemaps and offer data are intentionally retained.
DELETE FROM sitemaps_raw;
DROP TABLE raw_sitemap_documents_data;
