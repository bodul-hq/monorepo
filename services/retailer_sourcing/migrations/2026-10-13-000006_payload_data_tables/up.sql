-- Each table's payload moves to a side table holding only the payload, keyed
-- by the row's id, so reading or listing rows doesn't load it.

ALTER TABLE raw_sitemap_documents DROP COLUMN body;

CREATE TABLE raw_sitemap_documents_data (
    id UUID PRIMARY KEY REFERENCES raw_sitemap_documents(id) ON DELETE CASCADE,
    body TEXT NOT NULL
);

ALTER TABLE processed_sitemaps DROP COLUMN document;

CREATE TABLE processed_sitemaps_data (
    id UUID PRIMARY KEY REFERENCES processed_sitemaps(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);

ALTER TABLE consolidated_sitemaps DROP COLUMN content;

CREATE TABLE consolidated_sitemaps_data (
    id UUID PRIMARY KEY REFERENCES consolidated_sitemaps(id) ON DELETE CASCADE,
    content JSONB NOT NULL
);

ALTER TABLE raw_offers DROP COLUMN body;

CREATE TABLE raw_offers_data (
    id UUID PRIMARY KEY REFERENCES raw_offers(id) ON DELETE CASCADE,
    body TEXT NOT NULL
);
