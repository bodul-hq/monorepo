-- The offer data read out of each downloaded page, one row per offer at each
-- stage. A later run replaces an offer's row. Each document is kept in the
-- stage's _data table, keyed by the row's id, so rows can be read and listed
-- without loading it.

-- The page extracted into JSON with the retailer's architecture.
CREATE TABLE extracted_offers (
    id UUID PRIMARY KEY,
    offer_id UUID NOT NULL UNIQUE,
    extracted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE extracted_offers_data (
    id UUID PRIMARY KEY REFERENCES extracted_offers(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);

-- The extracted JSON normalized by the retailer's normalizer.
CREATE TABLE normalized_offers (
    id UUID PRIMARY KEY,
    offer_id UUID NOT NULL UNIQUE,
    normalized_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE normalized_offers_data (
    id UUID PRIMARY KEY REFERENCES normalized_offers(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);

-- The normalized document structuralized by the retailer's structuralizer.
CREATE TABLE structuralized_offers (
    id UUID PRIMARY KEY,
    offer_id UUID NOT NULL UNIQUE,
    structuralized_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE structuralized_offers_data (
    id UUID PRIMARY KEY REFERENCES structuralized_offers(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);
