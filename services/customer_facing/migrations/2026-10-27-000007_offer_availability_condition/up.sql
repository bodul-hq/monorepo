ALTER TABLE offers ADD COLUMN availability JSONB,
    ADD COLUMN item_condition JSONB,
    ADD CONSTRAINT offers_availability_shape CHECK ((
        availability IS NULL
        OR availability IN ('"available"', '"unavailable"', '"in_stock"', '"out_of_stock"', '"sold_out"')
        OR (jsonb_typeof(availability) = 'object'
            AND availability ? 'other'
            AND availability - 'other' = '{}'::jsonb
            AND jsonb_typeof(availability -> 'other') = 'string')
    ) IS TRUE),
    ADD CONSTRAINT offers_item_condition_shape CHECK ((
        item_condition IS NULL OR item_condition = '"new"'
        OR (jsonb_typeof(item_condition) = 'object'
            AND item_condition ? 'other'
            AND item_condition - 'other' = '{}'::jsonb
            AND jsonb_typeof(item_condition -> 'other') = 'string')
    ) IS TRUE);
