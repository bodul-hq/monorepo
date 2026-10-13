-- Offer discovery's tables. An offer is the offer_id handed out when its
-- product URL was discovered; its progress is logged, not stored.

-- The downloaded page of an offer.
CREATE TABLE raw_offers (
    id UUID PRIMARY KEY,
    offer_id UUID NOT NULL UNIQUE,
    url TEXT NOT NULL,
    body TEXT NOT NULL,
    body_size INT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- Finds a page of the same URL downloaded recently, to skip downloading it again.
CREATE INDEX idx_raw_offers_url_fetched_at ON raw_offers (url, fetched_at);
