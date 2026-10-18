-- Recreate empty legacy payload tables only; deleted bodies cannot be restored.
DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM sitemaps_processed)
        OR EXISTS (SELECT 1 FROM sitemaps_consolidated)
        OR EXISTS (SELECT 1 FROM offer_details_raw)
        OR EXISTS (SELECT 1 FROM offer_details_extracted)
        OR EXISTS (SELECT 1 FROM offer_details_normalized)
        OR EXISTS (SELECT 1 FROM offer_details_structuralized) THEN
        RAISE EXCEPTION 'Remove new pipeline rows before reverting the object-storage cutover';
    END IF;
END $$;
CREATE TABLE processed_sitemaps_data (
    id UUID PRIMARY KEY REFERENCES sitemaps_processed(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);
CREATE TABLE consolidated_sitemaps_data (
    id UUID PRIMARY KEY REFERENCES sitemaps_consolidated(id) ON DELETE CASCADE,
    content JSONB NOT NULL
);
CREATE TABLE raw_offers_data (
    id UUID PRIMARY KEY REFERENCES offer_details_raw(id) ON DELETE CASCADE,
    body TEXT NOT NULL
);
CREATE TABLE extracted_offers_data (
    id UUID PRIMARY KEY REFERENCES offer_details_extracted(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);
CREATE TABLE normalized_offers_data (
    id UUID PRIMARY KEY REFERENCES offer_details_normalized(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);
CREATE TABLE structuralized_offers_data (
    id UUID PRIMARY KEY REFERENCES offer_details_structuralized(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);
