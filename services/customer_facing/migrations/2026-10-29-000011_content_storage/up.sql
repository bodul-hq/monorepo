-- Coordinate this migration with the new projection writer and reader.
DO $$ BEGIN
 IF EXISTS (SELECT 1 FROM offers o WHERE NOT EXISTS (
   SELECT 1 FROM pg_catalog.jsonb_array_elements(o.content) e
   WHERE (e->>'language') IS NOT DISTINCT FROM o.language
   AND e->>'title'=o.title AND (e->>'description') IS NOT DISTINCT FROM o.description
 )) THEN RAISE EXCEPTION 'selected content disagrees with scalar fields'; END IF;
END $$;
ALTER TABLE offers DROP CONSTRAINT offers_content_nonempty_array;
UPDATE offers SET content=pg_catalog.jsonb_build_object('selected_language', language,'entries',
 (SELECT pg_catalog.jsonb_agg(e || pg_catalog.jsonb_build_object('url',offers.url) ORDER BY position) FROM pg_catalog.jsonb_array_elements(content) WITH ORDINALITY AS elements(e, position)));
CREATE FUNCTION offer_selected_content(value jsonb) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE AS $$
 SELECT e FROM pg_catalog.jsonb_array_elements(value->'entries') e
 WHERE (e->'language') = (value->'selected_language') LIMIT 1
$$;
CREATE FUNCTION offer_content_valid(value jsonb) RETURNS boolean
LANGUAGE plpgsql IMMUTABLE STRICT AS $$
DECLARE e jsonb; seen jsonb := '[]'; selected boolean := false;
BEGIN
 IF pg_catalog.jsonb_typeof(value) <> 'object' OR NOT value ?& ARRAY['selected_language','entries'] THEN RETURN false; END IF;
 IF pg_catalog.jsonb_typeof(value->'entries') <> 'array' THEN RETURN false; END IF;
 IF pg_catalog.jsonb_array_length(value->'entries')=0 OR (value - ARRAY['selected_language','entries']) <> '{}'::jsonb THEN RETURN false; END IF;
 IF value->'selected_language' <> 'null'::jsonb AND pg_catalog.jsonb_typeof(value->'selected_language') <> 'string' THEN RETURN false; END IF;
 FOR e IN SELECT * FROM pg_catalog.jsonb_array_elements(value->'entries') LOOP
  IF pg_catalog.jsonb_typeof(e) <> 'object' OR NOT e ?& ARRAY['language','title','description','url']
  OR (e - ARRAY['language','title','description','url']) <> '{}'::jsonb
  OR pg_catalog.jsonb_typeof(e->'title') <> 'string' OR pg_catalog.btrim(e->>'title')=''
  OR pg_catalog.jsonb_typeof(e->'url') <> 'string' OR pg_catalog.octet_length(e->>'url')>2048
  OR (e->'language' <> 'null'::jsonb AND pg_catalog.jsonb_typeof(e->'language') <> 'string')
  OR (e->'description' <> 'null'::jsonb AND pg_catalog.jsonb_typeof(e->'description') <> 'string') THEN RETURN false; END IF;
  IF seen @> pg_catalog.jsonb_build_array(e->'language') THEN RETURN false; END IF;
  seen := seen || pg_catalog.jsonb_build_array(e->'language');
  selected := selected OR e->'language'=value->'selected_language';
 END LOOP;
 RETURN selected;
END $$;
ALTER TABLE offers ADD CONSTRAINT offers_content_valid CHECK (offer_content_valid(content));
ALTER TABLE offers DROP CONSTRAINT offers_retailer_code_url_key;
DROP INDEX offers_title_search_idx;
ALTER TABLE offers DROP COLUMN title, DROP COLUMN description, DROP COLUMN language, DROP COLUMN url;
CREATE UNIQUE INDEX offers_selected_url_idx ON offers (retailer_code, ((offer_selected_content(content)->>'url') COLLATE "C"));
CREATE INDEX offers_content_title_search_idx ON offers USING gin (pg_catalog.lower(offer_selected_content(content)->>'title') customer_facing_extensions.gin_trgm_ops);
CREATE INDEX offers_content_description_search_idx ON offers USING gin (pg_catalog.lower(offer_selected_content(content)->>'description') customer_facing_extensions.gin_trgm_ops);
