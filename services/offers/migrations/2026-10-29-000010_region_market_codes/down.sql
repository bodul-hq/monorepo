DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM read_offers WHERE target_market->>'kind' = 'region'
        AND target_market ? 'code' AND target_market <> '{"kind":"region","code":"eu"}'::jsonb) THEN
        RAISE EXCEPTION 'Unsupported regional code: cannot downgrade without an explicit mapping';
    END IF;
END $$;
UPDATE read_offers SET target_market = '{"kind":"region","label":"European Union"}'::jsonb
WHERE target_market = '{"kind":"region","code":"eu"}'::jsonb;
