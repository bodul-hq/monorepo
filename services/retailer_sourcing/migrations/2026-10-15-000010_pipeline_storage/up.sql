-- Preserve existing size values; only their location and name change.
ALTER TABLE sitemaps_raw ADD COLUMN storage JSONB;
UPDATE sitemaps_raw SET storage = jsonb_build_object('content_size', body_size);
ALTER TABLE sitemaps_raw ALTER COLUMN storage SET NOT NULL, DROP COLUMN body_size;

ALTER TABLE sitemaps_processed ADD COLUMN storage JSONB;
UPDATE sitemaps_processed SET storage = jsonb_build_object('content_size', document_size);
ALTER TABLE sitemaps_processed ALTER COLUMN storage SET NOT NULL, DROP COLUMN document_size;

ALTER TABLE offer_details_raw ADD COLUMN storage JSONB;
UPDATE offer_details_raw SET storage = jsonb_build_object('content_size', body_size);
ALTER TABLE offer_details_raw ALTER COLUMN storage SET NOT NULL, DROP COLUMN body_size;

-- These stages did not previously record sizes. Measure compact UTF-8 JSON,
-- retaining whitespace and escapes inside strings. A missing payload leaves
-- storage NULL, so the migration fails atomically instead of inventing a size.
ALTER TABLE offer_details_extracted ADD COLUMN storage JSONB;
UPDATE offer_details_extracted AS record
SET storage = jsonb_build_object('content_size', octet_length(
    regexp_replace(data.document::text, '("(?:[^"\\]|\\.)*")|\s+', '\1', 'g')
))
FROM extracted_offers_data AS data WHERE data.id = record.id;
ALTER TABLE offer_details_extracted ALTER COLUMN storage SET NOT NULL;

ALTER TABLE offer_details_normalized ADD COLUMN storage JSONB;
UPDATE offer_details_normalized AS record
SET storage = jsonb_build_object('content_size', octet_length(
    regexp_replace(data.document::text, '("(?:[^"\\]|\\.)*")|\s+', '\1', 'g')
))
FROM normalized_offers_data AS data WHERE data.id = record.id;
ALTER TABLE offer_details_normalized ALTER COLUMN storage SET NOT NULL;

ALTER TABLE offer_details_structuralized ADD COLUMN storage JSONB;
UPDATE offer_details_structuralized AS record
SET storage = jsonb_build_object('content_size', octet_length(
    regexp_replace(data.document::text, '("(?:[^"\\]|\\.)*")|\s+', '\1', 'g')
))
FROM structuralized_offers_data AS data WHERE data.id = record.id;
ALTER TABLE offer_details_structuralized ALTER COLUMN storage SET NOT NULL;

-- sitemaps_consolidated already has storage.content_size.
