DROP TABLE raw_sitemap_documents_data;
ALTER TABLE raw_sitemap_documents ADD COLUMN body TEXT NOT NULL;

DROP TABLE processed_sitemaps_data;
ALTER TABLE processed_sitemaps ADD COLUMN document JSONB NOT NULL;

DROP TABLE consolidated_sitemaps_data;
ALTER TABLE consolidated_sitemaps ADD COLUMN content JSONB NOT NULL;

DROP TABLE raw_offers_data;
ALTER TABLE raw_offers ADD COLUMN body TEXT NOT NULL;
