CREATE INDEX idx_raw_offers_url_fetched_at ON raw_offers (url, fetched_at);
ALTER TABLE raw_offers DROP COLUMN retailer_code;
