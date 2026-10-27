ALTER TABLE read_offers ADD COLUMN content JSONB;
UPDATE read_offers SET content = jsonb_build_array(jsonb_build_object(
    'language', language, 'title', title, 'description', description));
ALTER TABLE read_offers ALTER COLUMN content SET NOT NULL;
ALTER TABLE read_offers ADD CONSTRAINT read_offers_content_nonempty
    CHECK (jsonb_typeof(content) = 'array' AND jsonb_array_length(content) > 0);
