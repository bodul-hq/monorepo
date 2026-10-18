-- Restore the previous integer columns. Missing, non-integer, or out-of-range
-- values fail the transactional downgrade without losing any records.
ALTER TABLE sitemaps_raw ADD COLUMN body_size INT;
UPDATE sitemaps_raw SET body_size = CASE
    WHEN jsonb_typeof(storage -> 'content_size') = 'number'
        AND (storage ->> 'content_size') ~ '^-?[0-9]+$'
    THEN (storage ->> 'content_size')::INTEGER END;
ALTER TABLE sitemaps_raw ALTER COLUMN body_size SET NOT NULL, DROP COLUMN storage;

ALTER TABLE sitemaps_processed ADD COLUMN document_size INT;
UPDATE sitemaps_processed SET document_size = CASE
    WHEN jsonb_typeof(storage -> 'content_size') = 'number'
        AND (storage ->> 'content_size') ~ '^-?[0-9]+$'
    THEN (storage ->> 'content_size')::INTEGER END;
ALTER TABLE sitemaps_processed ALTER COLUMN document_size SET NOT NULL, DROP COLUMN storage;

ALTER TABLE offer_details_raw ADD COLUMN body_size INT;
UPDATE offer_details_raw SET body_size = CASE
    WHEN jsonb_typeof(storage -> 'content_size') = 'number'
        AND (storage ->> 'content_size') ~ '^-?[0-9]+$'
    THEN (storage ->> 'content_size')::INTEGER END;
ALTER TABLE offer_details_raw ALTER COLUMN body_size SET NOT NULL, DROP COLUMN storage;

ALTER TABLE offer_details_extracted DROP COLUMN storage;
ALTER TABLE offer_details_normalized DROP COLUMN storage;
ALTER TABLE offer_details_structuralized DROP COLUMN storage;
