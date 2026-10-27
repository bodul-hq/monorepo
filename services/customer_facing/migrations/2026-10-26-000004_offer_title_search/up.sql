DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_extension e JOIN pg_namespace n ON n.oid = e.extnamespace
        WHERE e.extname = 'pg_trgm' AND n.nspname = 'customer_facing_extensions'
    ) THEN
        RAISE EXCEPTION 'Provision pg_trgm in customer_facing_extensions before migrating customer-facing';
    END IF;
END $$;

CREATE INDEX offers_title_search_idx ON offers
USING gin (lower(title) customer_facing_extensions.gin_trgm_ops);
