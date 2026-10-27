ALTER TABLE offers ADD COLUMN was_price_amount BIGINT,
    ADD COLUMN was_price_currency TEXT,
    ADD CONSTRAINT offers_was_price_pair CHECK (
        (was_price_amount IS NULL) = (was_price_currency IS NULL)
    );
