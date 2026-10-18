-- Each downloaded page records its retailer, as the sitemap tables do.
ALTER TABLE raw_offers ADD COLUMN retailer_code TEXT NOT NULL;

-- Offer pages are always downloaded, so nothing looks up a recent page by URL.
DROP INDEX idx_raw_offers_url_fetched_at;
