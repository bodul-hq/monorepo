-- Pause projection writers. Validate the entire inventory before rewriting any row.
DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM read_offers WHERE target_market->>'kind' = 'region'
        AND target_market <> '{"kind":"region","label":"European Union"}'::jsonb
        AND target_market <> '{"kind":"region","code":"eu"}'::jsonb) THEN
        RAISE EXCEPTION 'Unmapped regional market: agree an explicit code mapping before migration';
    END IF;
END $$;
UPDATE read_offers SET target_market = '{"kind":"region","code":"eu"}'::jsonb
WHERE target_market = '{"kind":"region","label":"European Union"}'::jsonb;
