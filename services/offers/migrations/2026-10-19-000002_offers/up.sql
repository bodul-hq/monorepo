CREATE TABLE offer_identities (
    id UUID PRIMARY KEY,
    retailer_code TEXT NOT NULL,
    url TEXT COLLATE "C" NOT NULL CHECK (octet_length(url) <= 2048),
    UNIQUE (retailer_code, url)
);

CREATE TABLE event_store (
    id BIGSERIAL PRIMARY KEY,
    agg_id VARCHAR(255) NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence > 0),
    payload_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL,
    recorded_on TIMESTAMPTZ NOT NULL,
    UNIQUE (agg_id, sequence)
);

CREATE TABLE read_offers (
    id UUID PRIMARY KEY,
    retailer_code TEXT NOT NULL,
    url TEXT COLLATE "C" NOT NULL CHECK (octet_length(url) <= 2048),
    title TEXT NOT NULL,
    price_amount BIGINT NOT NULL,
    currency TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (retailer_code, url)
);
