-- Pause projection writers while this transactional backfill runs.
ALTER TABLE offers ADD COLUMN content JSONB;
UPDATE offers SET content = jsonb_build_array(jsonb_build_object(
    'language', language, 'title', title, 'description', description
));
ALTER TABLE offers ALTER COLUMN content SET NOT NULL;
ALTER TABLE offers ADD CONSTRAINT offers_content_nonempty_array CHECK (
    CASE WHEN jsonb_typeof(content) = 'array' THEN jsonb_array_length(content) > 0 ELSE FALSE END
);
