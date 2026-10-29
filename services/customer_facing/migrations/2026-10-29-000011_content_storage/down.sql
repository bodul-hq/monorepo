DO $$ BEGIN
 IF EXISTS (SELECT 1 FROM offers o, jsonb_array_elements(o.content->'entries') e
 WHERE e->>'url' IS DISTINCT FROM offer_selected_content(o.content)->>'url') THEN
 RAISE EXCEPTION 'downgrade would lose localized URLs; restore from backup instead'; END IF;
END $$;
ALTER TABLE offers ADD COLUMN title TEXT, ADD COLUMN description TEXT, ADD COLUMN language TEXT, ADD COLUMN url TEXT COLLATE "C";
UPDATE offers SET title=offer_selected_content(content)->>'title',description=offer_selected_content(content)->>'description',language=content->>'selected_language',url=offer_selected_content(content)->>'url';
DROP INDEX offers_selected_url_idx;
DROP INDEX offers_content_title_search_idx;
DROP INDEX offers_content_description_search_idx;
ALTER TABLE offers DROP CONSTRAINT offers_content_valid;
UPDATE offers SET content=(SELECT jsonb_agg(e-'url' ORDER BY position) FROM jsonb_array_elements(content->'entries') WITH ORDINALITY AS elements(e, position));
DROP FUNCTION offer_content_valid(jsonb);
DROP FUNCTION offer_selected_content(jsonb);
ALTER TABLE offers ALTER COLUMN title SET NOT NULL, ALTER COLUMN url SET NOT NULL;
ALTER TABLE offers ADD CONSTRAINT offers_url_check CHECK (octet_length(url)<=2048), ADD CONSTRAINT offers_retailer_code_url_key UNIQUE (retailer_code,url), ADD CONSTRAINT offers_content_nonempty_array CHECK (CASE WHEN jsonb_typeof(content)='array' THEN jsonb_array_length(content)>0 ELSE false END);
CREATE INDEX offers_title_search_idx ON offers USING gin (lower(title) customer_facing_extensions.gin_trgm_ops);
