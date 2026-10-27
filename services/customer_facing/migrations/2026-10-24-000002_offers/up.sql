CREATE TABLE offers (
    id UUID PRIMARY KEY,
    retailer_code TEXT NOT NULL,
    url TEXT COLLATE "C" NOT NULL CHECK (octet_length(url) <= 2048),
    title TEXT NOT NULL,
    price_amount BIGINT NOT NULL,
    currency TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (retailer_code, url)
);

-- Committed atomically with the business change, never with queue completion.
CREATE TABLE applied_offer_messages (
    message_id UUID PRIMARY KEY
);
